use crate::pattern::CompiledPattern;
use anyhow::Result;

pub fn print_matching_line(
    input_line: &str,
    compiled_pattern: &CompiledPattern,
    file_path: &str,
    print_file_name: bool,
) -> Result<bool> {
    let is_match = compiled_pattern.match_pattern(input_line)?;
    if !is_match {
        return Ok(false);
    }
    print_prefix(file_path, print_file_name);
    println!("{input_line}");

    Ok(true)
}

pub fn print_colored_line(
    input_line: &str,
    compiled_pattern: &CompiledPattern,
    file_path: &str,
    print_file_name: bool,
) -> Result<bool> {
    let pattern_matches = compiled_pattern.match_all_patterns(input_line)?;
    if !pattern_matches.has_match {
        return Ok(false);
    }
    let mut last_end = 0;
    print_prefix(file_path, print_file_name);
    for (start, end) in &pattern_matches.ranges {
        print!("{}", &input_line[last_end..*start]);
        print_colored_match(&input_line[*start..*end]);
        last_end = *end;
    }
    println!("{}", &input_line[last_end..]);
    Ok(true)
}

pub fn print_matching_ranges(
    input_line: &str,
    compiled_pattern: &CompiledPattern,
    file_path: &str,
    print_file_name: bool,
    color_mode: bool,
) -> Result<bool> {
    let pattern_matches = compiled_pattern.match_all_patterns(input_line)?;
    for (start, end) in &pattern_matches.ranges {
        print_prefix(file_path, print_file_name);
        if color_mode {
            print_colored_match(&input_line[*start..*end]);
            println!();
        } else {
            println!("{}", &input_line[*start..*end]);
        }
    }
    Ok(pattern_matches.has_match)
}

fn print_prefix(file_path: &str, print_file_name: bool) {
    if print_file_name {
        print!("{file_path}:");
    }
}

fn print_colored_match(text: &str) {
    print!("\x1b[01;31m{text}\x1b[m");
}
