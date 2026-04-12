use std::vec;

use anyhow::Result;

pub fn match_pattern(input_line: &str, pattern: &str) -> Result<bool> {
    let pattern_tokens = parse_pattern(pattern)?;

    for start in 0..input_line.len() {
        let is_match = match_tokens_recursive(input_line.as_bytes(), start, &pattern_tokens)?;
        if is_match {
            return Ok(true);
        }
    }
    Ok(false)
}

fn match_tokens_recursive(
    input_bytes: &[u8],
    start: usize,
    tokens: &[PatternToken],
) -> Result<bool> {
    if tokens.is_empty() {
        return Ok(true);
    }

    let token = &tokens[0];
    let rest_tokens = &tokens[1..];
    if let PatternToken::Quantifier { min, max, inner } = token {
        let mut match_count = 0;
        let mut positions = vec![start];
        let mut next_idx = start;
        let mut is_match;
        while match_count < *max {
            (is_match, next_idx) = inner.matches(input_bytes, next_idx);
            if !is_match {
                break;
            }
            match_count += 1;
            positions.push(next_idx);
        }
        if match_count < *min {
            return Ok(false);
        }
        for count in (*min..=match_count).rev() {
            let try_idx = positions[count];
            if match_tokens_recursive(input_bytes, try_idx, rest_tokens)? {
                return Ok(true);
            }
        }
        Ok(false)
    } else if let PatternToken::Alternation(alternatives) = token {
        for alt_tokens in alternatives {
            let mut combined_tokens = alt_tokens.clone();
            combined_tokens.extend_from_slice(rest_tokens);
            if match_tokens_recursive(input_bytes, start, &combined_tokens)? {
                return Ok(true);
            }
        }
        Ok(false)
    } else {
        let (is_match, next_idx) = token.matches(input_bytes, start);
        if !is_match {
            return Ok(false);
        }
        match_tokens_recursive(input_bytes, next_idx, rest_tokens)
    }
}

#[derive(Clone)]
enum PatternToken {
    Literal(char),
    Digit,
    WordChar,
    Wildcard,
    CharacterGroup(Vec<char>),
    NegatedCharacterGroup(Vec<char>),
    StartAnchor,
    EndAnchor,
    Quantifier {
        min: usize,
        max: usize,
        inner: Box<Self>,
    },
    Alternation(Vec<Vec<Self>>),
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
            Self::Wildcard => {
                if index >= input_bytes.len() {
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
            Self::Quantifier { .. } | Self::Alternation(_) => {
                // This case is handled in the main matching logic
                (false, index)
            }
        }
    }
}

#[allow(clippy::too_many_lines)]
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
            '.' => {
                tokens.push(PatternToken::Wildcard);
                i += 1;
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
            '+' | '*' | '?' => {
                if tokens.is_empty() {
                    return Err(anyhow::anyhow!(
                        "Quantifier '{}' cannot be the first token in the pattern",
                        c
                    ));
                }
                let prev_token = tokens.pop().unwrap();
                let new_token = match c {
                    '+' => PatternToken::Quantifier {
                        min: 1,
                        max: usize::MAX,
                        inner: Box::new(prev_token),
                    },
                    '*' => PatternToken::Quantifier {
                        min: 0,
                        max: usize::MAX,
                        inner: Box::new(prev_token),
                    },
                    '?' => PatternToken::Quantifier {
                        min: 0,
                        max: 1,
                        inner: Box::new(prev_token),
                    },
                    _ => unreachable!(),
                };
                tokens.push(new_token);
                i += 1;
            }
            '(' => {
                let mut depth = 1;
                let mut left = i + 1;
                let mut right = left;
                while right < pattern.len() {
                    if pattern.as_bytes()[right] as char == '(' {
                        depth += 1;
                    } else if pattern.as_bytes()[right] as char == '|' && depth == 1 {
                        let group_content = &pattern[left..right];
                        let group_tokens = parse_pattern(group_content)?;
                        tokens.push(PatternToken::Alternation(vec![group_tokens]));
                        left = right + 1;
                    } else if pattern.as_bytes()[right] as char == ')' {
                        depth -= 1;
                        if depth == 0 {
                            let group_content = &pattern[left..right];
                            let group_tokens = parse_pattern(group_content)?;
                            tokens.push(PatternToken::Alternation(vec![group_tokens]));
                            break;
                        }
                    }
                    right += 1;
                }
                if depth != 0 {
                    return Err(anyhow::anyhow!("Unmatched ( in pattern"));
                }
            }
            _ => {
                tokens.push(PatternToken::Literal(c));
                i += 1;
            }
        }
    }
    Ok(tokens)
}
