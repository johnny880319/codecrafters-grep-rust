use crate::pattern::{self, PatternToken};
use anyhow::Result;

pub fn print_result(
    input_line: &str,
    pattern_tokens: &[PatternToken],
    file_path: &str,
    print_file_name: bool,
) -> Result<bool> {
    let is_match = pattern::match_pattern(input_line, pattern_tokens)?;
    if !is_match {
        return Ok(false);
    }
    print_prefix(file_path, print_file_name);
    println!("{input_line}");

    Ok(true)
}

pub fn print_all_results(
    input_line: &str,
    pattern_tokens: &[PatternToken],
    file_path: &str,
    print_file_name: bool,
) -> Result<bool> {
    let pattern_matchs = pattern::match_all_patterns(input_line, pattern_tokens)?;
    for (start, end) in &pattern_matchs.matched_idx {
        print_prefix(file_path, print_file_name);
        println!("{}", &input_line[*start..*end]);
    }
    Ok(pattern_matchs.is_match)
}

pub fn print_colored_results(
    input_line: &str,
    pattern_tokens: &[PatternToken],
    file_path: &str,
    print_file_name: bool,
) -> Result<bool> {
    let pattern_matchs = pattern::match_all_patterns(input_line, pattern_tokens)?;
    if !pattern_matchs.is_match {
        return Ok(false);
    }
    let mut last_end = 0;
    print_prefix(file_path, print_file_name);
    for (start, end) in &pattern_matchs.matched_idx {
        print!("{}", &input_line[last_end..*start]);
        print!("\x1b[01;31m{}\x1b[m", &input_line[*start..*end]);
        last_end = *end;
    }
    println!("{}", &input_line[last_end..]);
    Ok(true)
}

fn print_prefix(file_path: &str, print_file_name: bool) {
    if print_file_name {
        print!("{file_path}:");
    }
}
