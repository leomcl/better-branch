use console::{Key, Term};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::{collections::HashSet, io, ops::Rem};

use crate::theme::render::TermThemeRenderer;
use crate::theme::Theme;

pub type Result<T = ()> = std::result::Result<T, io::Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    VimEscape,
    Quit,
    VimInsert,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    ToggleSelection,
    SafeDelete,
    ForceDelete,
    ClearSearch,
    ClearSelections,
    Backspace,
    Delete,
    InsertChar(char),
    Checkout,
    None,
}

#[derive(Debug, Clone)]
pub struct DeleteResult {
    pub branches: Vec<String>,
    pub force: bool,
}

#[derive(Debug, Clone)]
pub enum MenuResult {
    Checkout(String),
    Delete(DeleteResult),
}

#[derive(Clone)]
pub struct Menu<'a> {
    default: Option<usize>,
    items: Vec<String>,
    prompt: String,
    report: bool,
    clear: bool,
    highlight_matches: bool,
    current_branch: Option<String>,
    enable_vim_mode: bool,
    max_length: Option<usize>,
    theme: &'a dyn Theme,
    initial_text: String,
}

#[allow(dead_code)]
impl<'a> Menu<'a> {
    pub fn with_theme(theme: &'a dyn Theme) -> Self {
        Self {
            default: None,
            items: vec![],
            prompt: "".into(),
            report: true,
            clear: true,
            highlight_matches: true,
            current_branch: None,
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

    pub fn current_branch(mut self, val: String) -> Self {
        self.current_branch = Some(val);
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
    pub fn interact(self) -> Result<Option<MenuResult>> {
        self._interact_on(&Term::stderr())
    }

    fn _interact_on(self, term: &Term) -> Result<Option<MenuResult>> {
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
        let mut selected: HashSet<String> = HashSet::new();

        loop {
            let mut byte_indices = search_term
                .char_indices()
                .map(|(index, _)| index)
                .collect::<Vec<_>>();

            byte_indices.push(search_term.len());

            render.clear()?;
            render.fuzzy_select_prompt(
                self.prompt.as_str(),
                &search_term,
                byte_indices[cursor],
                selected.len(),
            )?;

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
                let is_selected = selected.contains(*item);
                render.fuzzy_select_prompt_item(
                    item,
                    Some(idx) == sel,
                    is_selected,
                    self.current_branch
                        .as_ref()
                        .map(|b| b == *item)
                        .unwrap_or(false),
                    self.highlight_matches,
                    &matcher,
                    &search_term,
                )?;
            }
            term.flush()?;

            let key = term.read_key()?;
            match self.classify_action(
                &key,
                vim_mode,
                cursor,
                byte_indices.len(),
                filtered_list.is_empty(),
                &search_term,
                &selected,
            ) {
                Action::VimEscape => {
                    self.handle_vim_mode_escape(&mut vim_mode);
                }
                Action::Quit => {
                    return self.handle_quit(term, &mut render);
                }
                Action::VimInsert => {
                    self.handle_vim_mode_insert(&mut vim_mode);
                }
                Action::MoveUp => {
                    self.handle_move_up(
                        term,
                        filtered_list.len(),
                        visible_term_rows,
                        &mut sel,
                        &mut starting_row,
                    )?;
                }
                Action::MoveDown => {
                    self.handle_move_down(
                        term,
                        filtered_list.len(),
                        visible_term_rows,
                        &mut sel,
                        &mut starting_row,
                    )?;
                }
                Action::MoveLeft => {
                    self.handle_move_left(term, &mut cursor)?;
                }
                Action::MoveRight => {
                    self.handle_move_right(term, &mut cursor)?;
                }
                Action::ToggleSelection => {
                    if let Some(sel_idx) = sel {
                        if let Some((branch_name, _)) = filtered_list.get(sel_idx) {
                            let branch = branch_name.to_string();
                            if selected.contains(&branch) {
                                selected.remove(&branch);
                            } else {
                                selected.insert(branch);
                            }
                        }
                    }
                }
                Action::Checkout => {
                    if let Some(sel_idx) = sel {
                        if let Some((branch_name, _)) = filtered_list.get(sel_idx) {
                            if self.clear {
                                render.clear()?;
                                term.flush()?;
                            }
                            term.show_cursor()?;
                            return Ok(Some(MenuResult::Checkout(branch_name.to_string())));
                        }
                    }
                }
                Action::SafeDelete => {
                    if !selected.is_empty() {
                        let branches: Vec<String> = selected.iter().cloned().collect();
                        if self.confirm_delete(term, &mut render, &branches, false)? {
                            return Ok(Some(MenuResult::Delete(DeleteResult {
                                branches,
                                force: false,
                            })));
                        }
                    }
                }
                Action::ForceDelete => {
                    if !selected.is_empty() {
                        let branches: Vec<String> = selected.iter().cloned().collect();
                        if self.confirm_delete(term, &mut render, &branches, true)? {
                            return Ok(Some(MenuResult::Delete(DeleteResult {
                                branches,
                                force: true,
                            })));
                        }
                    }
                }
                Action::ClearSearch => {
                    search_term.clear();
                    cursor = 0;
                    sel = self.default;
                    starting_row = 0;
                }
                Action::ClearSelections => {
                    selected.clear();
                }
                Action::Backspace => {
                    self.handle_backspace(term, &mut cursor, &mut search_term, &byte_indices)?;
                }
                Action::Delete => {
                    self.handle_delete(term, cursor, &mut search_term, &byte_indices)?;
                }
                Action::InsertChar(chr) => {
                    self.handle_char(
                        term,
                        chr,
                        &mut cursor,
                        &mut search_term,
                        &byte_indices,
                        &mut sel,
                        &mut starting_row,
                    )?;
                }
                Action::None => {}
            }

            render.clear_preserve_prompt(&size_vec)?;
        }
    }

    fn confirm_delete(
        &self,
        term: &Term,
        render: &mut TermThemeRenderer,
        branches: &[String],
        force: bool,
    ) -> Result<bool> {
        let action = if force { "Force delete" } else { "Delete" };
        let count = branches.len();
        let prompt = format!(
            "{} {} selected branch{}? (y/N)",
            action,
            count,
            if count == 1 { "" } else { "s" }
        );

        // Hide the list temporarily and show confirmation
        render.clear()?;
        term.write_line(&prompt)?;
        term.flush()?;

        loop {
            let key = term.read_key()?;
            match key {
                Key::Char('y' | 'Y') => {
                    term.clear_last_lines(1)?;
                    return Ok(true);
                }
                Key::Enter | Key::Char('n' | 'N') | Key::Escape => {
                    term.clear_last_lines(1)?;
                    return Ok(false);
                }
                _ => {}
            }
        }
    }

    fn handle_vim_mode_escape(&self, vim_mode: &mut bool) {
        *vim_mode = true;
    }

    fn handle_quit(
        &self,
        term: &Term,
        render: &mut TermThemeRenderer,
    ) -> Result<Option<MenuResult>> {
        if self.clear {
            render.clear()?;
            term.flush()?;
        }
        term.show_cursor()?;
        Ok(None)
    }

    fn handle_vim_mode_insert(&self, vim_mode: &mut bool) {
        *vim_mode = false;
    }

    fn handle_move_up(
        &self,
        term: &Term,
        filtered_list_len: usize,
        visible_term_rows: usize,
        sel: &mut Option<usize>,
        starting_row: &mut usize,
    ) -> Result {
        if *sel == Some(0) {
            *starting_row = filtered_list_len.max(visible_term_rows) - visible_term_rows;
        } else if *sel == Some(*starting_row) {
            *starting_row -= 1;
        }
        *sel = match *sel {
            None => Some(filtered_list_len - 1),
            Some(s) => Some(
                ((s as i64 - 1 + filtered_list_len as i64) % (filtered_list_len as i64)) as usize,
            ),
        };
        term.flush()?;
        Ok(())
    }

    fn handle_move_down(
        &self,
        term: &Term,
        filtered_list_len: usize,
        visible_term_rows: usize,
        sel: &mut Option<usize>,
        starting_row: &mut usize,
    ) -> Result {
        *sel = match *sel {
            None => Some(0),
            Some(s) => Some((s as u64 + 1).rem(filtered_list_len as u64) as usize),
        };
        if *sel == Some(visible_term_rows + *starting_row) {
            *starting_row += 1;
        } else if *sel == Some(0) {
            *starting_row = 0;
        }
        term.flush()?;
        Ok(())
    }

    fn handle_move_left(&self, term: &Term, cursor: &mut usize) -> Result {
        *cursor -= 1;
        term.flush()?;
        Ok(())
    }

    fn handle_move_right(&self, term: &Term, cursor: &mut usize) -> Result {
        *cursor += 1;
        term.flush()?;
        Ok(())
    }

    fn handle_backspace(
        &self,
        term: &Term,
        cursor: &mut usize,
        search_term: &mut String,
        byte_indices: &[usize],
    ) -> Result {
        *cursor -= 1;
        search_term.remove(byte_indices[*cursor]);
        term.flush()?;
        Ok(())
    }

    fn handle_delete(
        &self,
        term: &Term,
        cursor: usize,
        search_term: &mut String,
        byte_indices: &[usize],
    ) -> Result {
        search_term.remove(byte_indices[cursor]);
        term.flush()?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn handle_char(
        &self,
        term: &Term,
        chr: char,
        cursor: &mut usize,
        search_term: &mut String,
        byte_indices: &[usize],
        sel: &mut Option<usize>,
        starting_row: &mut usize,
    ) -> Result {
        search_term.insert(byte_indices[*cursor], chr);
        *cursor += 1;
        term.flush()?;
        *sel = Some(0);
        *starting_row = 0;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn classify_action(
        &self,
        key: &Key,
        vim_mode: bool,
        cursor: usize,
        byte_indices_len: usize,
        filtered_list_is_empty: bool,
        search_term: &str,
        selected: &HashSet<String>,
    ) -> Action {
        match (key, vim_mode) {
            (Key::Escape, false) if self.enable_vim_mode => Action::VimEscape,

            (Key::Escape, false) if !search_term.is_empty() => Action::ClearSearch,
            (Key::Escape, false) if !selected.is_empty() => Action::ClearSelections,
            (Key::Escape, false) => Action::Quit,
            (Key::Char('q'), true) => Action::Quit,

            (Key::Char('i' | 'a'), true) => Action::VimInsert,

            (Key::Char('\x10'), _) if !filtered_list_is_empty => Action::MoveUp,
            (Key::ArrowUp | Key::BackTab, _) if !filtered_list_is_empty => Action::MoveUp,
            (Key::Char('k'), true) if !filtered_list_is_empty => Action::MoveUp,

            (Key::Char('\x0e'), _) if !filtered_list_is_empty => Action::MoveDown,
            (Key::ArrowDown | Key::Tab, _) if !filtered_list_is_empty => Action::MoveDown,
            (Key::Char('j'), true) if !filtered_list_is_empty => Action::MoveDown,

            (Key::ArrowLeft, _) if cursor > 0 => Action::MoveLeft,
            (Key::Char('h'), true) if cursor > 0 => Action::MoveLeft,

            (Key::ArrowRight, _) if cursor < byte_indices_len - 1 => Action::MoveRight,
            (Key::Char('l'), true) if cursor < byte_indices_len - 1 => Action::MoveRight,

            (Key::Char(' ') | Key::Char('\0'), _) if !filtered_list_is_empty => {
                Action::ToggleSelection
            }

            (Key::Enter, _) if !filtered_list_is_empty => Action::Checkout,

            (Key::Char('\x04'), _) if !selected.is_empty() => Action::SafeDelete,

            (Key::Char('\x06') | Key::Char('\x18'), _) if !selected.is_empty() => {
                Action::ForceDelete
            }

            (Key::Backspace, _) if cursor > 0 => Action::Backspace,
            (Key::Del, _) if cursor < byte_indices_len - 1 => Action::Delete,

            (Key::Char(chr), _) if !chr.is_ascii_control() => Action::InsertChar(*chr),

            _ => Action::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::colorful::ColorfulTheme;

    #[allow(clippy::too_many_arguments)]
    fn classify(
        enable_vim_mode: bool,
        key: Key,
        vim: bool,
        cursor: usize,
        byte_indices_len: usize,
        list_empty: bool,
        search_term: &str,
        selected: &HashSet<String>,
    ) -> Action {
        let theme = ColorfulTheme::default();
        let menu = Menu::with_theme(&theme).vim_mode(enable_vim_mode);
        menu.classify_action(
            &key,
            vim,
            cursor,
            byte_indices_len,
            list_empty,
            search_term,
            selected,
        )
    }

    const VIM_ENABLED: bool = true;
    const VIM_DISABLED: bool = false;

    #[test]
    fn escape_in_vim_mode_enters_normal_mode() {
        assert_eq!(
            classify(
                VIM_ENABLED,
                Key::Escape,
                false,
                0,
                1,
                false,
                "",
                &HashSet::new()
            ),
            Action::VimEscape
        );
    }

    #[test]
    fn escape_clears_search_first() {
        assert_eq!(
            classify(
                VIM_DISABLED,
                Key::Escape,
                false,
                3,
                4,
                false,
                "feat",
                &HashSet::new()
            ),
            Action::ClearSearch
        );
    }

    #[test]
    fn escape_clears_selections_second() {
        let selected = HashSet::from(["feature/foo".to_string()]);
        assert_eq!(
            classify(VIM_DISABLED, Key::Escape, false, 0, 1, false, "", &selected),
            Action::ClearSelections
        );
    }

    #[test]
    fn escape_quits_when_clean() {
        assert_eq!(
            classify(
                VIM_DISABLED,
                Key::Escape,
                false,
                0,
                1,
                false,
                "",
                &HashSet::new()
            ),
            Action::Quit
        );
    }

    #[test]
    fn vim_quit_key() {
        assert_eq!(
            classify(
                VIM_ENABLED,
                Key::Char('q'),
                true,
                0,
                1,
                false,
                "",
                &HashSet::new()
            ),
            Action::Quit
        );
    }

    #[test]
    fn vim_insert_keys() {
        for key in ['i', 'a'] {
            assert_eq!(
                classify(
                    VIM_ENABLED,
                    Key::Char(key),
                    true,
                    0,
                    1,
                    false,
                    "",
                    &HashSet::new()
                ),
                Action::VimInsert,
                "vim key {:?} should enter insert mode",
                key
            );
        }
    }

    #[test]
    fn vim_movement_keys() {
        assert_eq!(
            classify(
                VIM_ENABLED,
                Key::Char('k'),
                true,
                0,
                1,
                false,
                "",
                &HashSet::new()
            ),
            Action::MoveUp
        );
        assert_eq!(
            classify(
                VIM_ENABLED,
                Key::Char('j'),
                true,
                0,
                1,
                false,
                "",
                &HashSet::new()
            ),
            Action::MoveDown
        );
        assert_eq!(
            classify(
                VIM_DISABLED,
                Key::ArrowUp,
                false,
                0,
                1,
                false,
                "",
                &HashSet::new()
            ),
            Action::MoveUp
        );
        assert_eq!(
            classify(
                VIM_DISABLED,
                Key::ArrowDown,
                false,
                0,
                1,
                false,
                "",
                &HashSet::new()
            ),
            Action::MoveDown
        );
    }

    #[test]
    fn ctrl_p_n_move_selection() {
        assert_eq!(
            classify(
                VIM_DISABLED,
                Key::Char('\x10'),
                false,
                0,
                1,
                false,
                "",
                &HashSet::new()
            ),
            Action::MoveUp
        );
        assert_eq!(
            classify(
                VIM_DISABLED,
                Key::Char('\x0e'),
                false,
                0,
                1,
                false,
                "",
                &HashSet::new()
            ),
            Action::MoveDown
        );
    }

    #[test]
    fn space_toggles_selection() {
        assert_eq!(
            classify(
                VIM_DISABLED,
                Key::Char(' '),
                false,
                0,
                1,
                false,
                "",
                &HashSet::new()
            ),
            Action::ToggleSelection
        );
    }

    #[test]
    fn enter_checks_out() {
        assert_eq!(
            classify(
                VIM_DISABLED,
                Key::Enter,
                false,
                0,
                1,
                false,
                "",
                &HashSet::new()
            ),
            Action::Checkout
        );
    }

    #[test]
    fn delete_keys_require_selection() {
        let empty = HashSet::new();
        assert_eq!(
            classify(
                VIM_DISABLED,
                Key::Char('\x04'),
                false,
                0,
                1,
                false,
                "",
                &empty
            ),
            Action::None
        );

        let selected = HashSet::from(["feature/foo".to_string()]);
        assert_eq!(
            classify(
                VIM_DISABLED,
                Key::Char('\x04'),
                false,
                0,
                1,
                false,
                "",
                &selected
            ),
            Action::SafeDelete
        );
        assert_eq!(
            classify(
                VIM_DISABLED,
                Key::Char('\x06'),
                false,
                0,
                1,
                false,
                "",
                &selected
            ),
            Action::ForceDelete
        );
        assert_eq!(
            classify(
                VIM_DISABLED,
                Key::Char('\x18'),
                false,
                0,
                1,
                false,
                "",
                &selected
            ),
            Action::ForceDelete
        );
    }

    #[test]
    fn typing_inserts_characters() {
        assert_eq!(
            classify(
                VIM_DISABLED,
                Key::Char('f'),
                false,
                0,
                1,
                false,
                "",
                &HashSet::new()
            ),
            Action::InsertChar('f')
        );
    }

    #[test]
    fn backspace_requires_cursor_not_at_start() {
        let empty = HashSet::new();
        assert_eq!(
            classify(
                VIM_DISABLED,
                Key::Backspace,
                false,
                0,
                2,
                false,
                "a",
                &empty
            ),
            Action::None
        );
        assert_eq!(
            classify(
                VIM_DISABLED,
                Key::Backspace,
                false,
                1,
                2,
                false,
                "a",
                &empty
            ),
            Action::Backspace
        );
    }

    #[test]
    fn arrows_respect_cursor_bounds() {
        let empty = HashSet::new();
        assert_eq!(
            classify(
                VIM_DISABLED,
                Key::ArrowLeft,
                false,
                0,
                2,
                false,
                "a",
                &empty
            ),
            Action::None
        );
        assert_eq!(
            classify(
                VIM_DISABLED,
                Key::ArrowRight,
                false,
                2,
                2,
                false,
                "a",
                &empty
            ),
            Action::None
        );
        assert_eq!(
            classify(
                VIM_DISABLED,
                Key::ArrowRight,
                false,
                0,
                2,
                false,
                "a",
                &empty
            ),
            Action::MoveRight
        );
    }

    #[test]
    fn no_navigation_on_empty_list() {
        assert_eq!(
            classify(
                VIM_DISABLED,
                Key::ArrowUp,
                false,
                0,
                1,
                true,
                "",
                &HashSet::new()
            ),
            Action::None
        );
        assert_eq!(
            classify(
                VIM_DISABLED,
                Key::ArrowDown,
                false,
                0,
                1,
                true,
                "",
                &HashSet::new()
            ),
            Action::None
        );
    }

    #[test]
    fn unknown_keys_do_nothing() {
        assert_eq!(
            classify(
                VIM_DISABLED,
                Key::Char('\x1b'),
                false,
                0,
                1,
                false,
                "",
                &HashSet::new()
            ),
            Action::None
        );
    }
}
