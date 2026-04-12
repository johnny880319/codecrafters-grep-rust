use anyhow::Result;
use std::env;
use std::io;
use std::io::Read;
use std::process;
mod pattern;

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() -> Result<()> {
    if env::args().nth(1).unwrap() != "-E" {
        println!("Expected first argument to be '-E'");
        process::exit(1);
    }

    let pattern = env::args().nth(2).unwrap();
    let mut input_string = String::new();

    io::stdin().read_to_string(&mut input_string).unwrap();

    let input_lines = input_string.lines().collect::<Vec<&str>>();
    let mut is_match = false;
    for input_line in input_lines {
        if pattern::match_pattern(input_line, &pattern)? {
            println!("{input_line}");
            is_match = true;
        }
    }
    if !is_match {
        process::exit(1);
    }
    process::exit(0)
}
