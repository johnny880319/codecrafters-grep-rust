mod matcher;
mod parser;

pub use matcher::{CompiledPattern, match_all_patterns, match_pattern};
pub use parser::parse_pattern;
