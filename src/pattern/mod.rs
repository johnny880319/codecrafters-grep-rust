mod matcher;
mod parser;

pub use matcher::{PatternToken, match_all_patterns, match_pattern};
pub use parser::parse_pattern;
