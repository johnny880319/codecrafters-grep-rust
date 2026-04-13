use anyhow::Result;

pub fn match_pattern(input_line: &str, pattern: &str) -> Result<bool> {
    let pattern_tokens = parse_pattern(pattern)?;

    for start in 0..input_line.len() {
        let (is_match, _) = match_tokens(input_line.as_bytes(), start, &pattern_tokens)?;
        if is_match {
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn match_all_patterns(input_line: &str, pattern: &str) -> Result<Vec<(usize, usize)>> {
    let pattern_tokens = parse_pattern(pattern)?;
    let mut start = 0;
    let mut matched_idx = Vec::new();

    while start <= input_line.len() {
        let (is_match, end) = match_tokens(input_line.as_bytes(), start, &pattern_tokens)?;
        if is_match {
            matched_idx.push((start, end));
            start = end; // Move past the matched portion for the next search
            continue;
        }
        start += 1;
    }
    Ok(matched_idx)
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
            Self::Literal(c) => Self::match_single_char(input_bytes, index, |b| b as char == *c),
            Self::Digit => Self::match_single_char(input_bytes, index, |b| b.is_ascii_digit()),
            Self::WordChar => Self::match_single_char(input_bytes, index, |b| {
                b.is_ascii_alphanumeric() || b == b'_'
            }),
            Self::Wildcard => Self::match_single_char(input_bytes, index, |_| true),
            Self::CharacterGroup(chars) => {
                Self::match_single_char(input_bytes, index, |b| chars.contains(&(b as char)))
            }
            Self::NegatedCharacterGroup(chars) => {
                Self::match_single_char(input_bytes, index, |b| !chars.contains(&(b as char)))
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
                unreachable!("This should be handled in the recursive matching logic.")
            }
        }
    }

    fn match_single_char(
        input_bytes: &[u8],
        index: usize,
        predicate: impl Fn(u8) -> bool,
    ) -> (bool, usize) {
        if index >= input_bytes.len() || !predicate(input_bytes[index]) {
            return (false, index);
        }
        (true, index + 1)
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
            '.' => {
                tokens.push(PatternToken::Wildcard);
                i += 1;
            }
            '[' => {
                let new_token;
                (new_token, i) = parse_character_group(pattern, i + 1)?;
                tokens.push(new_token);
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
                let new_token;
                (new_token, i) = parse_alternation(pattern, i + 1)?;
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

fn parse_character_group(pattern: &str, start: usize) -> Result<(PatternToken, usize)> {
    let end = pattern[start..]
        .find(']')
        .ok_or_else(|| anyhow::anyhow!("Unmatched [ in pattern"))?
        + start;
    let group_content = &pattern[start..end];
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

fn parse_alternation(pattern: &str, mut start: usize) -> Result<(PatternToken, usize)> {
    let mut depth = 1;
    let mut end = start;
    let mut alternatives = Vec::new();
    while end < pattern.len() {
        match pattern.as_bytes()[end] as char {
            '(' => {
                depth += 1;
            }
            '|' if depth == 1 => {
                alternatives.push(parse_pattern(&pattern[start..end])?);
                start = end + 1;
            }
            ')' => {
                depth -= 1;
                if depth == 0 {
                    alternatives.push(parse_pattern(&pattern[start..end])?);
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

fn match_tokens(
    input_bytes: &[u8],
    index: usize,
    tokens: &[PatternToken],
) -> Result<(bool, usize)> {
    if tokens.is_empty() {
        return Ok((true, index));
    }

    let token = &tokens[0];
    let rest_tokens = &tokens[1..];
    match token {
        PatternToken::Quantifier { min, max, inner } => {
            let mut match_count = 0;
            let mut positions = vec![index];
            let mut candidate_index = index;
            while match_count < *max {
                let is_match;
                (is_match, candidate_index) = match_tokens(
                    input_bytes,
                    candidate_index,
                    std::slice::from_ref(inner.as_ref()),
                )?;
                if !is_match {
                    break;
                }
                match_count += 1;
                positions.push(candidate_index);
            }
            if match_count < *min {
                return Ok((false, index));
            }
            for count in (*min..=match_count).rev() {
                let try_idx = positions[count];
                let (is_match, end) = match_tokens(input_bytes, try_idx, rest_tokens)?;
                if is_match {
                    return Ok((true, end));
                }
            }
            Ok((false, index))
        }
        PatternToken::Alternation(alternatives) => {
            for alt_tokens in alternatives {
                let mut combined_tokens = alt_tokens.clone();
                combined_tokens.extend_from_slice(rest_tokens);
                let (is_match, end) = match_tokens(input_bytes, index, &combined_tokens)?;
                if is_match {
                    return Ok((true, end));
                }
            }
            Ok((false, index))
        }
        _ => {
            let (is_match, index) = token.matches(input_bytes, index);
            if !is_match {
                return Ok((false, index));
            }
            match_tokens(input_bytes, index, rest_tokens)
        }
    }
}
