// Most code in here taken from https://github.com/console-rs/dialoguer

use crate::theme::Theme;
use console::{style, Style};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::fmt;

pub struct ColorfulTheme {
    pub prompt_prefix: String,
    pub prompt_suffix: String,
    pub prompt_style: Style,
    pub active_item_prefix: String,
    pub inactive_item_prefix: String,
    pub active_item_style: Style,
    pub fuzzy_match_highlight_style: Style,
    pub fuzzy_cursor_style: Style,
    pub success_prefix: String,
    pub success_style: Style,
}

impl Default for ColorfulTheme {
    fn default() -> Self {
        Self {
            prompt_prefix: style("?".to_string()).yellow().for_stderr().to_string(),
            prompt_suffix: style("›".to_string())
                .black()
                .bright()
                .for_stderr()
                .to_string(),
            prompt_style: Style::new().for_stderr().bold(),
            active_item_prefix: style("❯".to_string()).green().for_stderr().to_string(),
            inactive_item_prefix: " ".to_string(),
            active_item_style: Style::new().for_stderr().cyan(),
            fuzzy_match_highlight_style: Style::new().for_stderr().bold(),
            fuzzy_cursor_style: Style::new().for_stderr().black().on_white(),
            success_prefix: style("✔".to_string()).green().for_stderr().to_string(),
            success_style: Style::new().for_stderr().green(),
        }
    }
}

impl Theme for ColorfulTheme {
    fn format_fuzzy_select_prompt(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        search_term: &str,
        bytes_pos: usize,
    ) -> fmt::Result {
        if !prompt.is_empty() {
            write!(
                f,
                "{} {} ",
                self.prompt_prefix,
                self.prompt_style.apply_to(prompt)
            )?;
        }

        let (st_head, remaining) = search_term.split_at(bytes_pos);
        let mut chars = remaining.chars();
        let chr = chars.next().unwrap_or(' ');
        let st_cursor = self.fuzzy_cursor_style.apply_to(chr);
        let st_tail = chars.as_str();

        write!(f, "{} {st_head}{st_cursor}{st_tail}", self.prompt_suffix)
    }

    fn format_fuzzy_select_prompt_item(
        &self,
        f: &mut dyn fmt::Write,
        text: &str,
        active: bool,
        highlight_matches: bool,
        matcher: &SkimMatcherV2,
        search_term: &str,
    ) -> fmt::Result {
        write!(
            f,
            "{} ",
            if active {
                &self.active_item_prefix
            } else {
                &self.inactive_item_prefix
            }
        )?;

        if highlight_matches {
            if let Some((_score, indices)) = matcher.fuzzy_indices(text, search_term) {
                for (idx, c) in text.chars().enumerate() {
                    if indices.contains(&idx) {
                        if active {
                            write!(
                                f,
                                "{}",
                                self.active_item_style
                                    .apply_to(self.fuzzy_match_highlight_style.apply_to(c))
                            )?;
                        } else {
                            write!(f, "{}", self.fuzzy_match_highlight_style.apply_to(c))?;
                        }
                    } else if active {
                        write!(f, "{}", self.active_item_style.apply_to(c))?;
                    } else {
                        write!(f, "{}", c)?;
                    }
                }

                return Ok(());
            }
        }

        if active {
            write!(f, "{}", self.active_item_style.apply_to(text))
        } else {
            write!(f, "{}", text)
        }
    }

    fn format_input_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        sel: &str,
    ) -> fmt::Result {
        if !prompt.is_empty() {
            write!(
                f,
                "{} {} ",
                self.success_prefix,
                self.prompt_style.apply_to(prompt)
            )?;
        }
        write!(
            f,
            "{} {}",
            self.prompt_suffix,
            self.success_style.apply_to(sel)
        )
    }
}
