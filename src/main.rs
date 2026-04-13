use anyhow::Result;
use std::env;
use std::io::{self, IsTerminal, Read};
use std::process;
use walkdir::WalkDir;
mod pattern;
mod print_result;

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() -> Result<()> {
    let grep_args = parse_args();
    let mut paths_and_contents = grep_args.paths_and_contents;

    if paths_and_contents.is_empty() {
        paths_and_contents.push((String::new(), {
            let mut input_string = String::new();
            io::stdin().read_to_string(&mut input_string).unwrap();
            input_string
        }));
    }

    let mut is_any_match = false;
    for (file_name, file_content) in paths_and_contents {
        for input_line in file_content.lines() {
            if grep_args.o_flag {
                is_any_match |= print_result::print_all_results(
                    input_line,
                    &grep_args.pattern,
                    &file_name,
                    grep_args.print_file_name,
                )?;
            } else if grep_args.color_mode {
                is_any_match |= print_result::print_colored_results(
                    input_line,
                    &grep_args.pattern,
                    &file_name,
                    grep_args.print_file_name,
                )?;
            } else {
                is_any_match |= print_result::print_result(
                    input_line,
                    &grep_args.pattern,
                    &file_name,
                    grep_args.print_file_name,
                )?;
            }
        }
    }
    if !is_any_match {
        process::exit(1);
    }
    process::exit(0)
}

struct GrepArgs {
    pattern: String,
    paths_and_contents: Vec<(String, String)>,
    print_file_name: bool,
    o_flag: bool,
    color_mode: bool,
}

fn parse_args() -> GrepArgs {
    let env_args: Vec<String> = env::args().collect();
    let o_flag = env_args.iter().any(|arg| arg == "-o");

    let is_color_always = env_args.iter().any(|arg| arg == "--color=always");
    let is_color_auto = env_args.iter().any(|arg| arg == "--color=auto");
    let color_mode = is_color_always || (is_color_auto && std::io::stdout().is_terminal());

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
    let file_or_dir_paths = env::args()
        .filter(|p| !p.starts_with('-'))
        .skip(2)
        .collect::<Vec<_>>();

    // If -r flag is provided, we need to recursively search for files in the provided directories
    let r_flag = env::args().any(|arg| arg == "-r");
    let print_file_name = r_flag || file_or_dir_paths.len() > 1;

    let file_paths = if r_flag {
        let mut paths = Vec::new();
        for path in file_or_dir_paths {
            if std::fs::metadata(&path)
                .map(|m| m.is_dir())
                .unwrap_or(false)
            {
                for entry in WalkDir::new(&path)
                    .follow_links(false)
                    .into_iter()
                    .flatten()
                {
                    if entry.file_type().is_file() {
                        paths.push(entry.path().to_string_lossy().to_string());
                    }
                }
            } else {
                paths.push(path);
            }
        }
        paths
    } else {
        file_or_dir_paths
    };

    let paths_and_contents = file_paths
        .into_iter()
        .map(|s| {
            let content = std::fs::read_to_string(&s).unwrap_or_else(|_| {
                eprintln!("Error: Could not read file {s}");
                process::exit(1);
            });
            (s, content)
        })
        .collect::<Vec<_>>();

    GrepArgs {
        pattern,
        paths_and_contents,
        print_file_name,
        o_flag,
        color_mode,
    }
}
