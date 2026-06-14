// Most code in here taken from https://github.com/console-rs/dialoguer
use console::{Key, Term};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::{io, ops::Rem};

use crate::theme::render::TermThemeRenderer;
use crate::theme::Theme;

pub type Result<T = ()> = std::result::Result<T, io::Error>;

#[derive(Clone)]
pub struct FuzzySelect<'a> {
    default: Option<usize>,
    items: Vec<String>,
    prompt: String,
    report: bool,
    clear: bool,
    highlight_matches: bool,
    enable_vim_mode: bool,
    max_length: Option<usize>,
    theme: &'a dyn Theme,
    initial_text: String,
}

#[allow(dead_code)]
impl<'a> FuzzySelect<'a> {
    pub fn with_theme(theme: &'a dyn Theme) -> Self {
        Self {
            default: None,
            items: vec![],
            prompt: "".into(),
            report: true,
            clear: true,
            highlight_matches: true,
            enable_vim_mode: false,
            max_length: None,
            theme,
            initial_text: "".into(),
        }
    }

    pub fn clear(mut self, val: bool) -> Self {
        self.clear = val;
        self
    }

    pub fn default(mut self, val: usize) -> Self {
        self.default = Some(val);
        self
    }

    pub fn item<T: ToString>(mut self, item: T) -> Self {
        self.items.push(item.to_string());
        self
    }

    pub fn items<T: ToString>(mut self, items: &[T]) -> Self {
        for item in items {
            self.items.push(item.to_string());
        }
        self
    }

    pub fn with_initial_text<S: Into<String>>(mut self, initial_text: S) -> Self {
        self.initial_text = initial_text.into();
        self
    }

    pub fn with_prompt<S: Into<String>>(mut self, prompt: S) -> Self {
        self.prompt = prompt.into();
        self
    }

    pub fn report(mut self, val: bool) -> Self {
        self.report = val;
        self
    }

    pub fn highlight_matches(mut self, val: bool) -> Self {
        self.highlight_matches = val;
        self
    }

    pub fn vim_mode(mut self, val: bool) -> Self {
        self.enable_vim_mode = val;
        self
    }

    pub fn max_length(mut self, rows: usize) -> Self {
        self.max_length = Some(rows);
        self
    }

    #[inline]
    pub fn interact(self) -> Result<usize> {
        self.interact_on(&Term::stderr())
    }

    #[inline]
    pub fn interact_opt(self) -> Result<Option<usize>> {
        self.interact_on_opt(&Term::stderr())
    }

    #[inline]
    pub fn interact_on(self, term: &Term) -> Result<usize> {
        self._interact_on(term, false)?
            .ok_or_else(|| io::Error::other("Quit not allowed in this case"))
    }

    #[inline]
    pub fn interact_on_opt(self, term: &Term) -> Result<Option<usize>> {
        self._interact_on(term, true)
    }

    fn _interact_on(self, term: &Term, allow_quit: bool) -> Result<Option<usize>> {
        let mut cursor = self.initial_text.chars().count();
        let mut search_term = self.initial_text.to_owned();

        let mut render = TermThemeRenderer::new(term, self.theme);
        let mut sel = self.default;

        let mut size_vec = Vec::new();
        for items in self.items.iter().as_slice() {
            let size = &items.len();
            size_vec.push(*size);
        }

        let matcher = SkimMatcherV2::default();

        let visible_term_rows = (term.size().0 as usize).max(3) - 2;
        let visible_term_rows = self
            .max_length
            .unwrap_or(visible_term_rows)
            .min(visible_term_rows);
        let mut starting_row = 0;

        term.hide_cursor()?;

        let mut vim_mode = false;

        loop {
            let mut byte_indices = search_term
                .char_indices()
                .map(|(index, _)| index)
                .collect::<Vec<_>>();

            byte_indices.push(search_term.len());

            render.clear()?;
            render.fuzzy_select_prompt(self.prompt.as_str(), &search_term, byte_indices[cursor])?;

            let mut filtered_list = self
                .items
                .iter()
                .map(|item| (item, matcher.fuzzy_match(item, &search_term)))
                .filter_map(|(item, score)| score.map(|s| (item, s)))
                .collect::<Vec<(&String, i64)>>();

            filtered_list.sort_unstable_by(|(_, s1), (_, s2)| s2.cmp(s1));

            for (idx, (item, _)) in filtered_list
                .iter()
                .enumerate()
                .skip(starting_row)
                .take(visible_term_rows)
            {
                render.fuzzy_select_prompt_item(
                    item,
                    Some(idx) == sel,
                    self.highlight_matches,
                    &matcher,
                    &search_term,
                )?;
            }
            term.flush()?;

            match (term.read_key()?, sel, vim_mode) {
                (Key::Escape, _, false) if self.enable_vim_mode => {
                    vim_mode = true;
                }
                (Key::Escape, _, false) | (Key::Char('q'), _, true) if allow_quit => {
                    if self.clear {
                        render.clear()?;
                        term.flush()?;
                    }
                    term.show_cursor()?;
                    return Ok(None);
                }
                (Key::Char('i' | 'a'), _, true) => {
                    vim_mode = false;
                }
                (Key::Char('\x10'), _, _)
                | (Key::ArrowUp | Key::BackTab, _, _)
                | (Key::Char('k'), _, true)
                    if !filtered_list.is_empty() =>
                {
                    if sel == Some(0) {
                        starting_row =
                            filtered_list.len().max(visible_term_rows) - visible_term_rows;
                    } else if sel == Some(starting_row) {
                        starting_row -= 1;
                    }
                    sel = match sel {
                        None => Some(filtered_list.len() - 1),
                        Some(sel) => Some(
                            ((sel as i64 - 1 + filtered_list.len() as i64)
                                % (filtered_list.len() as i64))
                                as usize,
                        ),
                    };
                    term.flush()?;
                }
                (Key::Char('\x0e'), _, _)
                | (Key::ArrowDown | Key::Tab, _, _)
                | (Key::Char('j'), _, true)
                    if !filtered_list.is_empty() =>
                {
                    sel = match sel {
                        None => Some(0),
                        Some(sel) => {
                            Some((sel as u64 + 1).rem(filtered_list.len() as u64) as usize)
                        }
                    };
                    if sel == Some(visible_term_rows + starting_row) {
                        starting_row += 1;
                    } else if sel == Some(0) {
                        starting_row = 0;
                    }
                    term.flush()?;
                }
                (Key::ArrowLeft, _, _) | (Key::Char('h'), _, true) if cursor > 0 => {
                    cursor -= 1;
                    term.flush()?;
                }
                (Key::ArrowRight, _, _) | (Key::Char('l'), _, true)
                    if cursor < byte_indices.len() - 1 =>
                {
                    cursor += 1;
                    term.flush()?;
                }
                (Key::Enter, Some(sel), _) if !filtered_list.is_empty() => {
                    if self.clear {
                        render.clear()?;
                    }

                    if self.report {
                        render
                            .input_prompt_selection(self.prompt.as_str(), filtered_list[sel].0)?;
                    }

                    let sel_string = filtered_list[sel].0;
                    let sel_string_pos_in_items =
                        self.items.iter().position(|item| item.eq(sel_string));

                    term.show_cursor()?;
                    return Ok(sel_string_pos_in_items);
                }
                (Key::Backspace, _, _) if cursor > 0 => {
                    cursor -= 1;
                    search_term.remove(byte_indices[cursor]);
                    term.flush()?;
                }
                (Key::Del, _, _) if cursor < byte_indices.len() - 1 => {
                    search_term.remove(byte_indices[cursor]);
                    term.flush()?;
                }
                (Key::Char(chr), _, _) if !chr.is_ascii_control() => {
                    search_term.insert(byte_indices[cursor], chr);
                    cursor += 1;
                    term.flush()?;
                    sel = Some(0);
                    starting_row = 0;
                }

                _ => {}
            }

            render.clear_preserve_prompt(&size_vec)?;
        }
    }
}
