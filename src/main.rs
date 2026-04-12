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
    let mut input_strings = env::args()
        .filter(|p| !p.starts_with('-'))
        .skip(2)
        .map(|s| {
            let content = std::fs::read_to_string(&s).unwrap_or_else(|_| {
                eprintln!("Error: Could not read file {s}");
                process::exit(1);
            });
            (Some(s), content)
        })
        .collect::<Vec<_>>();

    if input_strings.is_empty() {
        input_strings.push((None, {
            let mut input_string = String::new();
            io::stdin().read_to_string(&mut input_string).unwrap();
            input_string
        }));
    }

    if input_strings.len() == 1 {
        input_strings[0].0 = None;
    }

    let mut is_any_match = false;

    for (file_name, file_content) in input_strings {
        for input_line in file_content.lines() {
            if is_o_flag {
                is_any_match |= print_result::print_all_results(input_line, &pattern)?;
            } else if is_color_flag {
                is_any_match |= print_result::print_colored_results(input_line, &pattern)?;
            } else {
                is_any_match |=
                    print_result::print_result(input_line, &pattern, file_name.as_deref())?;
            }
        }
    }
    if !is_any_match {
        process::exit(1);
    }
    process::exit(0)
}
