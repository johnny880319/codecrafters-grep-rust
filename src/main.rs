use anyhow::Result;
use std::env;
use std::io::{self, Read};
use std::process;
mod pattern;

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() -> Result<()> {
    let is_o_flag = env::args().any(|arg| arg == "-o");

    let pattern = env::args().next_back().unwrap();
    let mut input_string = String::new();

    io::stdin().read_to_string(&mut input_string).unwrap();

    let input_lines = input_string.lines();
    let mut is_any_match = false;
    for input_line in input_lines {
        let (is_match, start, end) = pattern::match_pattern(input_line, &pattern)?;
        if is_match {
            if is_o_flag {
                println!("{}", &input_line[start..end]);
            } else {
                println!("{input_line}");
            }
            is_any_match = true;
        }
    }
    if !is_any_match {
        process::exit(1);
    }
    process::exit(0)
}
