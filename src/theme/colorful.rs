use crate::theme::Theme;
use console::{style, Style};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::fmt;

#[allow(clippy::too_many_arguments)]
fn write_item_text(
    f: &mut dyn fmt::Write,
    text: &str,
    active: bool,
    highlight_matches: bool,
    matcher: &SkimMatcherV2,
    search_term: &str,
    highlight_style: &Style,
    base_style: &Style,
) -> fmt::Result {
    if highlight_matches {
        if let Some((_score, indices)) = matcher.fuzzy_indices(text, search_term) {
            for (idx, c) in text.chars().enumerate() {
                if indices.contains(&idx) {
                    if active {
                        write!(f, "{}", base_style.apply_to(highlight_style.apply_to(c)))?;
                    } else {
                        write!(f, "{}", highlight_style.apply_to(c))?;
                    }
                } else if active {
                    write!(f, "{}", base_style.apply_to(c))?;
                } else {
                    write!(f, "{}", c)?;
                }
            }
            return Ok(());
        }
    }

    if active {
        write!(f, "{}", base_style.apply_to(text))
    } else {
        write!(f, "{}", text)
    }
}

pub struct ColorfulTheme {
    pub prompt_prefix: String,
    pub prompt_suffix: String,
    pub prompt_style: Style,
    pub active_item_prefix: String,
    pub inactive_item_prefix: String,
    pub active_item_style: Style,
    pub current_branch_style: Style,
    pub fuzzy_match_highlight_style: Style,
    pub fuzzy_cursor_style: Style,
    #[allow(dead_code)]
    pub success_prefix: String,
    #[allow(dead_code)]
    pub success_style: Style,
    pub selected_tick_style: Style,
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
            current_branch_style: Style::new().for_stderr().yellow(),
            fuzzy_match_highlight_style: Style::new().for_stderr().bold(),
            fuzzy_cursor_style: Style::new().for_stderr().black().on_white(),
            success_prefix: style("✔".to_string()).green().for_stderr().to_string(),
            success_style: Style::new().for_stderr().green(),
            selected_tick_style: Style::new().for_stderr().green(),
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
        selection_count: usize,
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

        write!(f, "{} {st_head}{st_cursor}{st_tail}", self.prompt_suffix)?;

        if selection_count > 0 {
            write!(
                f,
                "  {}",
                self.prompt_style.apply_to(format!(
                    "[ {} branch{} selected ]",
                    selection_count,
                    if selection_count == 1 { "" } else { "es" }
                ))
            )?;
        }

        Ok(())
    }

    fn format_fuzzy_select_prompt_item(
        &self,
        f: &mut dyn fmt::Write,
        text: &str,
        active: bool,
        selected: bool,
        current: bool,
        highlight_matches: bool,
        matcher: &SkimMatcherV2,
        search_term: &str,
    ) -> fmt::Result {
        write!(
            f,
            "{} ",
            if active {
                self.active_item_prefix.clone()
            } else {
                self.inactive_item_prefix.clone()
            }
        )?;

        // Selection tick
        if selected {
            write!(f, "{} ", self.selected_tick_style.apply_to("✔"))?;
        }

        let base_style = if active {
            &self.active_item_style
        } else if current {
            &self.current_branch_style
        } else {
            return write_item_text(
                f,
                text,
                false,
                highlight_matches,
                matcher,
                search_term,
                &self.fuzzy_match_highlight_style,
                &Style::new(),
            );
        };

        write_item_text(
            f,
            text,
            true,
            highlight_matches,
            matcher,
            search_term,
            &self.fuzzy_match_highlight_style,
            base_style,
        )
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
