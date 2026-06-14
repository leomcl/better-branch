// Most code in here taken from https://github.com/console-rs/dialoguer

use fuzzy_matcher::skim::SkimMatcherV2;
use std::fmt;

pub mod colorful;
pub mod render;

pub trait Theme {
    fn format_fuzzy_select_prompt(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        search_term: &str,
        bytes_pos: usize,
    ) -> fmt::Result;

    fn format_fuzzy_select_prompt_item(
        &self,
        f: &mut dyn fmt::Write,
        text: &str,
        active: bool,
        highlight_matches: bool,
        matcher: &SkimMatcherV2,
        search_term: &str,
    ) -> fmt::Result;

    fn format_input_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        sel: &str,
    ) -> fmt::Result;
}
