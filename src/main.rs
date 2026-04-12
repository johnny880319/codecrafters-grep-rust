use anyhow::Result;
use std::env;
use std::io::{self, IsTerminal, Read};
use std::process;
mod pattern;
mod print_result;

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() -> Result<()> {
    let is_o_flag = env::args().any(|arg| arg == "-o");
    let is_color_always = env::args().any(|arg| arg == "--color=always");
    let is_color_auto = env::args().any(|arg| arg == "--color=auto");
    let is_color_flag = is_color_always || (is_color_auto && std::io::stdout().is_terminal());

    // First argument that is not a flag is the pattern
    let pattern = env::args()
        .skip(1)
        .find(|arg| !arg.starts_with('-'))
        .unwrap_or_else(|| {
            eprintln!("Error: No pattern provided");
            process::exit(1);
        });

    // Second argument that is not a flag is file path
    let file_path = env::args()
        .skip(1)
        .find(|arg| !arg.starts_with('-') && *arg != pattern);

    let mut input_string;
    if let Some(file_path) = file_path {
        input_string = std::fs::read_to_string(file_path).unwrap_or_else(|err| {
            eprintln!("Error reading file: {err}");
            process::exit(1);
        });
    } else {
        input_string = String::new();
        io::stdin().read_to_string(&mut input_string).unwrap();
    }

    let input_lines = input_string.lines();

    let mut is_any_match = false;
    for input_line in input_lines {
        if is_o_flag {
            is_any_match |= print_result::print_all_results(input_line, &pattern)?;
        } else if is_color_flag {
            is_any_match |= print_result::print_colored_results(input_line, &pattern)?;
        } else {
            is_any_match |= print_result::print_result(input_line, &pattern)?;
        }
    }
    if !is_any_match {
        process::exit(1);
    }
    process::exit(0)
}
