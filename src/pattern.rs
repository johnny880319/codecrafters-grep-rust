use anyhow::Result;

pub fn match_pattern(input_line: &str, pattern: &str) -> Result<bool> {
    for start in 0..input_line.len() {
        let mut input_idx = start;
        let mut pattern_idx = 0;
        while input_idx < input_line.len() && pattern_idx < pattern.len() {
            let input_char = input_line.as_bytes()[input_idx] as char;

            match &pattern[pattern_idx..] {
                p if p.starts_with("\\d") => {
                    if !input_char.is_ascii_digit() {
                        break;
                    }
                    pattern_idx += 2;
                }
                p if p.starts_with("\\w") => {
                    if !input_char.is_ascii_alphanumeric() && input_char != '_' {
                        break;
                    }
                    pattern_idx += 2;
                }
                p if p.starts_with("[^") && p.contains(']') => {
                    let end_idx = pattern[pattern_idx..]
                        .find(']')
                        .ok_or_else(|| anyhow::anyhow!("Unmatched [ in pattern"))?
                        + pattern_idx;
                    let chars_to_not_match = &pattern[pattern_idx + 2..end_idx];
                    if chars_to_not_match.contains(input_char) {
                        break;
                    }
                    pattern_idx = end_idx + 1;
                }
                p if p.starts_with('[') && p.contains(']') => {
                    let end_idx = pattern[pattern_idx..]
                        .find(']')
                        .ok_or_else(|| anyhow::anyhow!("Unmatched [ in pattern"))?
                        + pattern_idx;
                    let chars_to_match = &pattern[pattern_idx + 1..end_idx];
                    if !chars_to_match.contains(input_char) {
                        break;
                    }
                    pattern_idx = end_idx + 1;
                }
                _ => {
                    let pattern_char = pattern.as_bytes()[pattern_idx] as char;
                    if input_char != pattern_char {
                        break;
                    }
                    pattern_idx += 1;
                }
            }
            input_idx += 1;
        }
        if pattern_idx == pattern.len() {
            return Ok(true);
        }
    }
    Ok(false)
}
