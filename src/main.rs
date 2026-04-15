use crate::args::GrepArgs;
use crate::pattern::CompiledPattern;
use anyhow::Result;
use std::{
    fs,
    io::{self, Read},
    process,
};

mod args;
mod output;
mod pattern;

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() -> Result<()> {
    let grep_args = args::parse_args();
    let compiled_pattern = CompiledPattern::parse(&grep_args.pattern_text)?;

    let mut is_any_match = false;

    if grep_args.file_paths.is_empty() {
        let mut input_string = String::new();
        io::stdin().read_to_string(&mut input_string)?;
        is_any_match |= match_content(&input_string, &compiled_pattern, &grep_args, "stdin")?;
    }

    for file_path in &grep_args.file_paths {
        let file_content = fs::read_to_string(file_path).unwrap_or_else(|_| {
            eprintln!("Error: Could not read file {file_path}");
            process::exit(1);
        });
        is_any_match |= match_content(&file_content, &compiled_pattern, &grep_args, file_path)?;
    }
    if !is_any_match {
        process::exit(1);
    }
    process::exit(0)
}

fn match_content(
    content: &str,
    compiled_pattern: &CompiledPattern,
    grep_args: &GrepArgs,
    file_path: &str,
) -> Result<bool> {
    let mut is_any_match = false;
    for input_line in content.lines() {
        if grep_args.only_matching {
            is_any_match |= output::print_all_results(
                input_line,
                compiled_pattern,
                file_path,
                grep_args.print_file_name,
                grep_args.color_mode,
            )?;
        } else if grep_args.color_mode {
            is_any_match |= output::print_colored_results(
                input_line,
                compiled_pattern,
                file_path,
                grep_args.print_file_name,
            )?;
        } else {
            is_any_match |= output::print_result(
                input_line,
                compiled_pattern,
                file_path,
                grep_args.print_file_name,
            )?;
        }
    }
    Ok(is_any_match)
}
