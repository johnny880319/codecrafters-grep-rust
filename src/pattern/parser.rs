use super::matcher::{CompiledPattern, PatternToken};
use anyhow::Result;

pub fn parse_pattern(pattern_text: &str) -> Result<CompiledPattern> {
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < pattern_text.len() {
        let c = pattern_text.as_bytes()[i] as char;
        match c {
            '\\' => {
                let (new_token, new_i) = parse_escape_sequence(pattern_text, i)?;
                tokens.push(new_token);
                i = new_i;
            }
            '.' => {
                tokens.push(PatternToken::Wildcard);
                i += 1;
            }
            '[' => {
                let (new_token, new_i) = parse_character_group(pattern_text, i + 1)?;
                tokens.push(new_token);
                i = new_i;
            }
            '^' if i == 0 => {
                tokens.push(PatternToken::StartAnchor);
                i += 1;
            }
            '$' if i == pattern_text.len() - 1 => {
                tokens.push(PatternToken::EndAnchor);
                i += 1;
            }
            '+' | '*' | '?' | '{' => {
                let prev_token = pop_previous_token(&mut tokens)?;
                let (new_token, new_i) = parse_quantifier(pattern_text, i, prev_token)?;
                tokens.push(new_token);
                i = new_i;
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
    Ok(CompiledPattern { tokens })
}

fn parse_escape_sequence(pattern_text: &str, start: usize) -> Result<(PatternToken, usize)> {
    if start + 1 >= pattern_text.len() {
        return Err(anyhow::anyhow!("Pattern ends with a single backslash"));
    }
    let next_char = pattern_text.as_bytes()[start + 1] as char;
    match next_char {
        'd' => Ok((PatternToken::Digit, start + 2)),
        'w' => Ok((PatternToken::WordChar, start + 2)),
        '1'..='9' => {
            let (backref_num, end) = parse_number(pattern_text, start + 1)?;
            Ok((PatternToken::Backreference(backref_num), end))
        }
        _ => Err(anyhow::anyhow!("Unknown escape sequence: \\{next_char}")),
    }
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

fn pop_previous_token(tokens: &mut Vec<PatternToken>) -> Result<PatternToken> {
    let prev_token = tokens
        .pop()
        .ok_or_else(|| anyhow::anyhow!("Quantifier cannot be the first token in the pattern"))?;
    Ok(prev_token)
}

fn parse_quantifier(
    pattern_text: &str,
    start: usize,
    prev_token: PatternToken,
) -> Result<(PatternToken, usize)> {
    match pattern_text.as_bytes()[start] as char {
        '+' => Ok((
            PatternToken::Quantifier {
                min: 1,
                max: usize::MAX,
                inner: Box::new(prev_token),
            },
            start + 1,
        )),
        '*' => Ok((
            PatternToken::Quantifier {
                min: 0,
                max: usize::MAX,
                inner: Box::new(prev_token),
            },
            start + 1,
        )),
        '?' => Ok((
            PatternToken::Quantifier {
                min: 0,
                max: 1,
                inner: Box::new(prev_token),
            },
            start + 1,
        )),
        '{' => parse_range_quantifier(pattern_text, start + 1, prev_token),
        _ => unreachable!(),
    }
}

fn parse_range_quantifier(
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
                alternatives.push(parse_pattern(&pattern_text[start..end])?.tokens);
                start = end + 1;
            }
            ')' => {
                depth -= 1;
                if depth == 0 {
                    alternatives.push(parse_pattern(&pattern_text[start..end])?.tokens);
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
