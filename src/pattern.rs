pub fn match_pattern(input_line: &str, pattern: &str) -> bool {
    if pattern == "\\d" {
        return input_line.chars().any(|c| c.is_ascii_digit());
    }
    if pattern == "\\w" {
        return input_line
            .chars()
            .any(|c| c.is_ascii_alphanumeric() || c == '_');
    }
    if pattern.starts_with("[^") && pattern.ends_with(']') {
        let chars_to_not_match = &pattern[2..pattern.len() - 1];
        return input_line.chars().any(|c| !chars_to_not_match.contains(c));
    }
    if pattern.starts_with('[') && pattern.ends_with(']') {
        let chars_to_match = &pattern[1..pattern.len() - 1];
        return input_line.chars().any(|c| chars_to_match.contains(c));
    }
    if pattern.chars().count() == 1 {
        input_line.contains(pattern)
    } else {
        panic!("Unhandled pattern: {pattern}")
    }
}
