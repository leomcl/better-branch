mod prompt;
mod renderer;
mod theme;

pub use prompt::FuzzySelect;
#[allow(unused_imports)]
pub use theme::{ColorfulTheme, Theme};

pub type Result<T = ()> = std::result::Result<T, std::io::Error>;
