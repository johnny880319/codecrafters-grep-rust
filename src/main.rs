use anyhow::Result;
use pattern::parser;
use std::{
    env, fs,
    io::{self, IsTerminal, Read},
    process,
};
use walkdir::WalkDir;
mod pattern;
mod print_result;

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() -> Result<()> {
    let grep_args = parse_args();

    let mut is_any_match = false;

    if grep_args.file_paths.is_empty() {
        let mut input_string = String::new();
        io::stdin().read_to_string(&mut input_string).unwrap();
        is_any_match |= match_content(&input_string, &grep_args, "stdin")?;
    }

    for file_path in &grep_args.file_paths {
        let file_content = fs::read_to_string(file_path).unwrap_or_else(|_| {
            eprintln!("Error: Could not read file {file_path}");
            process::exit(1);
        });
        is_any_match |= match_content(&file_content, &grep_args, file_path)?;
    }
    if !is_any_match {
        process::exit(1);
    }
    process::exit(0)
}

struct GrepArgs {
    pattern: String,
    file_paths: Vec<String>,
    print_file_name: bool,
    o_flag: bool,
    color_mode: bool,
}

fn parse_args() -> GrepArgs {
    let env_args: Vec<String> = env::args().collect();
    let o_flag = env_args.iter().any(|arg| arg == "-o");

    let is_color_always = env_args.iter().any(|arg| arg == "--color=always");
    let is_color_auto = env_args.iter().any(|arg| arg == "--color=auto");
    let color_mode = is_color_always || (is_color_auto && io::stdout().is_terminal());

    // First argument that is not a flag is the pattern
    let pattern = env_args
        .iter()
        .skip(1)
        .find(|arg| !arg.starts_with('-'))
        .cloned()
        .unwrap_or_else(|| {
            eprintln!("Error: No pattern provided");
            process::exit(1);
        });

    // Second argument that is not a flag is file or directory path
    let file_or_dir_paths = env_args
        .iter()
        .filter(|p| !p.starts_with('-'))
        .skip(2)
        .collect::<Vec<_>>();

    // If -r flag is provided, we need to recursively search for files in the provided directories
    let r_flag = env_args.iter().any(|arg| arg == "-r");
    let print_file_name = r_flag || file_or_dir_paths.len() > 1;

    let file_paths = if r_flag {
        let mut paths = Vec::new();
        for path in file_or_dir_paths {
            if fs::metadata(path).map(|m| m.is_dir()).unwrap_or(false) {
                for entry in WalkDir::new(path).follow_links(false).into_iter().flatten() {
                    if entry.file_type().is_file() {
                        paths.push(entry.path().to_string_lossy().to_string());
                    }
                }
            } else {
                paths.push(path.clone());
            }
        }
        paths
    } else {
        file_or_dir_paths.into_iter().cloned().collect()
    };

    GrepArgs {
        pattern,
        file_paths,
        print_file_name,
        o_flag,
        color_mode,
    }
}

fn match_content(content: &str, grep_args: &GrepArgs, file_path: &str) -> Result<bool> {
    let pattern_tokens = parser::parse_pattern(&grep_args.pattern)?;
    let mut is_any_match = false;
    for input_line in content.lines() {
        if grep_args.o_flag {
            is_any_match |= print_result::print_all_results(
                input_line,
                &pattern_tokens,
                file_path,
                grep_args.print_file_name,
            )?;
        } else if grep_args.color_mode {
            is_any_match |= print_result::print_colored_results(
                input_line,
                &pattern_tokens,
                file_path,
                grep_args.print_file_name,
            )?;
        } else {
            is_any_match |= print_result::print_result(
                input_line,
                &pattern_tokens,
                file_path,
                grep_args.print_file_name,
            )?;
        }
    }
    Ok(is_any_match)
}
