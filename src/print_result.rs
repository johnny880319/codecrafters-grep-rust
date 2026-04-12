use crate::pattern;
use anyhow::Result;

pub fn print_result(input_line: &str, pattern: &str, file_name: Option<&str>) -> Result<bool> {
    let is_match = pattern::match_pattern(input_line, pattern)?;
    if !is_match {
        return Ok(false);
    }
    match file_name {
        Some(name) => println!("{name}:{input_line}"),
        None => println!("{input_line}"),
    }
    Ok(true)
}

pub fn print_all_results(input_line: &str, pattern: &str) -> Result<bool> {
    let matched_idx = pattern::match_all_patterns(input_line, pattern)?;
    for (start, end) in &matched_idx {
        println!("{}", &input_line[*start..*end]);
    }
    Ok(!matched_idx.is_empty())
}

pub fn print_colored_results(input_line: &str, pattern: &str) -> Result<bool> {
    let matched_idx = pattern::match_all_patterns(input_line, pattern)?;
    if matched_idx.is_empty() {
        return Ok(false);
    }
    let mut last_end = 0;
    for (start, end) in &matched_idx {
        print!("{}", &input_line[last_end..*start]);
        print!("\x1b[01;31m{}\x1b[m", &input_line[*start..*end]);
        last_end = *end;
    }
    println!("{}", &input_line[last_end..]);
    Ok(true)
}
