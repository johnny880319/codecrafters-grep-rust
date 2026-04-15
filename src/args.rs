use anyhow::Result;
use std::{
    env, fs,
    io::{self, IsTerminal},
};
use walkdir::WalkDir;

pub struct GrepArgs {
    pub pattern_text: String,
    pub file_paths: Vec<String>,
    pub print_file_name: bool,
    pub only_matching: bool,
    pub color_mode: bool,
}

pub fn parse_args() -> Result<GrepArgs> {
    let env_args: Vec<String> = env::args().collect();
    let only_matching = env_args.iter().any(|arg| arg == "-o");

    let is_color_always = env_args.iter().any(|arg| arg == "--color=always");
    let is_color_auto = env_args.iter().any(|arg| arg == "--color=auto");
    let color_mode = is_color_always || (is_color_auto && io::stdout().is_terminal());

    // First argument that is not a flag is the pattern
    let pattern_text = env_args
        .iter()
        .skip(1)
        .find(|arg| !arg.starts_with('-'))
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("No pattern provided"))?;

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

    Ok(GrepArgs {
        pattern_text,
        file_paths,
        print_file_name,
        only_matching,
        color_mode,
    })
}
