use super::matcher::PatternToken;
use anyhow::Result;

pub fn parse_pattern(pattern_text: &str) -> Result<Vec<PatternToken>> {
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < pattern_text.len() {
        let c = pattern_text.as_bytes()[i] as char;
        match c {
            '\\' => {
                if i + 1 >= pattern_text.len() {
                    return Err(anyhow::anyhow!("Pattern ends with a single backslash"));
                }
                let next_char = pattern_text.as_bytes()[i + 1] as char;
                match next_char {
                    'd' => {
                        tokens.push(PatternToken::Digit);
                        i += 2;
                    }
                    'w' => {
                        tokens.push(PatternToken::WordChar);
                        i += 2;
                    }
                    '1'..='9' => {
                        let backref_num;
                        (backref_num, i) = parse_number(pattern_text, i + 1)?;
                        tokens.push(PatternToken::Backreference(backref_num));
                    }
                    _ => return Err(anyhow::anyhow!("Unknown escape sequence: \\{}", next_char)),
                }
            }
            '.' => {
                tokens.push(PatternToken::Wildcard);
                i += 1;
            }
            '[' => {
                let new_token;
                (new_token, i) = parse_character_group(pattern_text, i + 1)?;
                tokens.push(new_token);
            }
            '^' if i == 0 => {
                tokens.push(PatternToken::StartAnchor);
                i += 1;
            }
            '$' if i == pattern_text.len() - 1 => {
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
            '{' => {
                if tokens.is_empty() {
                    return Err(anyhow::anyhow!(
                        "Quantifier '{}' cannot be the first token in the pattern",
                        c
                    ));
                }
                let prev_token = tokens.pop().unwrap();
                let new_token;
                (new_token, i) = parse_quantifier(pattern_text, i + 1, prev_token)?;
                tokens.push(new_token);
            }
            '(' => {
                let new_token;
                (new_token, i) = parse_alternation(pattern_text, i + 1)?;
                tokens.push(new_token);
            }
            _ => {
                tokens.push(PatternToken::Literal(c));
                i += 1;
            }
        }
    }
    Ok(tokens)
}

fn parse_number(pattern_text: &str, start: usize) -> Result<(usize, usize)> {
    let mut end = start;
    while end < pattern_text.len() && pattern_text.as_bytes()[end].is_ascii_digit() {
        end += 1;
    }
    let number = pattern_text[start..end]
        .parse::<usize>()
        .map_err(|_| anyhow::anyhow!("Invalid number format: {}", &pattern_text[start..end]))?;
    Ok((number, end))
}

fn parse_character_group(pattern_text: &str, start: usize) -> Result<(PatternToken, usize)> {
    let end = pattern_text[start..]
        .find(']')
        .ok_or_else(|| anyhow::anyhow!("Unmatched [ in pattern"))?
        + start;
    let group_content = &pattern_text[start..end];
    if let Some(inner) = group_content.strip_prefix('^') {
        return Ok((
            PatternToken::NegatedCharacterGroup(inner.chars().collect()),
            end + 1,
        ));
    }
    Ok((
        PatternToken::CharacterGroup(group_content.chars().collect()),
        end + 1,
    ))
}

fn parse_alternation(pattern_text: &str, mut start: usize) -> Result<(PatternToken, usize)> {
    let mut depth = 1;
    let mut end = start;
    let mut alternatives = Vec::new();
    while end < pattern_text.len() {
        match pattern_text.as_bytes()[end] as char {
            '(' => {
                depth += 1;
            }
            '|' if depth == 1 => {
                alternatives.push(parse_pattern(&pattern_text[start..end])?);
                start = end + 1;
            }
            ')' => {
                depth -= 1;
                if depth == 0 {
                    alternatives.push(parse_pattern(&pattern_text[start..end])?);
                    break;
                }
            }
            _ => {}
        }
        end += 1;
    }
    if depth != 0 {
        return Err(anyhow::anyhow!("Unmatched ( in pattern"));
    }
    Ok((PatternToken::Alternation(alternatives), end + 1))
}

fn parse_quantifier(
    pattern_text: &str,
    start: usize,
    prev_token: PatternToken,
) -> Result<(PatternToken, usize)> {
    let end = pattern_text[start..]
        .find('}')
        .ok_or_else(|| anyhow::anyhow!("Unmatched {{ in pattern"))?
        + start;
    let quantifier_content = &pattern_text[start..end];
    let parts: Vec<&str> = quantifier_content.split(',').collect();
    match parts.len() {
        1 => {
            let count = parts[0].parse::<usize>().map_err(|_| {
                anyhow::anyhow!("Invalid quantifier format: {{{}}}", quantifier_content)
            })?;
            Ok((
                PatternToken::Quantifier {
                    min: count,
                    max: count,
                    inner: Box::new(prev_token),
                },
                end + 1,
            ))
        }
        2 => {
            let min = parts[0].parse::<usize>().map_err(|_| {
                anyhow::anyhow!("Invalid quantifier format: {{{}}}", quantifier_content)
            })?;
            let max = if parts[1].is_empty() {
                usize::MAX
            } else {
                parts[1].parse::<usize>().map_err(|_| {
                    anyhow::anyhow!("Invalid quantifier format: {{{}}}", quantifier_content)
                })?
            };
            if min > max {
                return Err(anyhow::anyhow!(
                    "Quantifier min cannot be greater than max: {{{}}}",
                    quantifier_content
                ));
            }
            Ok((
                PatternToken::Quantifier {
                    min,
                    max,
                    inner: Box::new(prev_token),
                },
                end + 1,
            ))
        }
        _ => Err(anyhow::anyhow!(
            "Invalid quantifier format: {{{}}}",
            quantifier_content
        )),
    }
}
