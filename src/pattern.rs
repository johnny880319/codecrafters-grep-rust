use anyhow::Result;

pub fn match_pattern(input_line: &str, pattern: &str) -> Result<bool> {
    let pattern_tokens = parse_pattern(pattern)?;

    for start in 0..input_line.len() {
        let mut input_idx = start;
        let mut is_match = true;

        for pattern_token in &pattern_tokens {
            (is_match, input_idx) = pattern_token.matches(input_line.as_bytes(), input_idx);
            if !is_match {
                break;
            }
        }
        if is_match {
            return Ok(true);
        }
    }
    Ok(false)
}

enum PatternToken {
    Literal(char),
    Digit,
    WordChar,
    CharacterGroup(Vec<char>),
    NegatedCharacterGroup(Vec<char>),
    StartAnchor,
    EndAnchor,
}

impl PatternToken {
    fn matches(&self, input_bytes: &[u8], index: usize) -> (bool, usize) {
        match self {
            Self::Literal(c) => {
                if index >= input_bytes.len() || input_bytes[index] as char != *c {
                    return (false, index);
                }
                (true, index + 1)
            }
            Self::Digit => {
                if index >= input_bytes.len() || !input_bytes[index].is_ascii_digit() {
                    return (false, index);
                }
                (true, index + 1)
            }
            Self::WordChar => {
                if index >= input_bytes.len()
                    || (!input_bytes[index].is_ascii_alphanumeric() && input_bytes[index] != b'_')
                {
                    return (false, index);
                }
                (true, index + 1)
            }
            Self::CharacterGroup(chars) => {
                if index >= input_bytes.len() || !chars.contains(&(input_bytes[index] as char)) {
                    return (false, index);
                }
                (true, index + 1)
            }
            Self::NegatedCharacterGroup(chars) => {
                if index >= input_bytes.len() || chars.contains(&(input_bytes[index] as char)) {
                    return (false, index);
                }
                (true, index + 1)
            }
            Self::StartAnchor => {
                if index != 0 {
                    return (false, index);
                }
                (true, index)
            }
            Self::EndAnchor => {
                if index != input_bytes.len() {
                    return (false, index);
                }
                (true, index)
            }
        }
    }
}

fn parse_pattern(pattern: &str) -> Result<Vec<PatternToken>> {
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < pattern.len() {
        let c = pattern.as_bytes()[i] as char;
        match c {
            '\\' => {
                if i + 1 >= pattern.len() {
                    return Err(anyhow::anyhow!("Pattern ends with a single backslash"));
                }
                let next_char = pattern.as_bytes()[i + 1] as char;
                match next_char {
                    'd' => tokens.push(PatternToken::Digit),
                    'w' => tokens.push(PatternToken::WordChar),
                    _ => return Err(anyhow::anyhow!("Unknown escape sequence: \\{}", next_char)),
                }
                i += 2;
            }
            '[' => {
                let end_idx = pattern[i..]
                    .find(']')
                    .ok_or_else(|| anyhow::anyhow!("Unmatched [ in pattern"))?
                    + i;
                let group_content = &pattern[i + 1..end_idx];
                if let Some(inner) = group_content.strip_prefix('^') {
                    tokens.push(PatternToken::NegatedCharacterGroup(inner.chars().collect()));
                } else {
                    tokens.push(PatternToken::CharacterGroup(
                        group_content.chars().collect(),
                    ));
                }
                i = end_idx + 1;
            }
            '^' if i == 0 => {
                tokens.push(PatternToken::StartAnchor);
                i += 1;
            }
            '$' if i == pattern.len() - 1 => {
                tokens.push(PatternToken::EndAnchor);
                i += 1;
            }
            _ => {
                tokens.push(PatternToken::Literal(c));
                i += 1;
            }
        }
    }
    Ok(tokens)
}
