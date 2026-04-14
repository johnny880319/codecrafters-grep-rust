use anyhow::Result;
use std::slice;

#[derive(Clone)]
pub enum PatternToken {
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
    Backreference(usize),
}

pub fn match_pattern(input_line: &str, pattern_tokens: &[PatternToken]) -> Result<bool> {
    for start in 0..=input_line.len() {
        let (is_match, _) = match_tokens(
            input_line.as_bytes(),
            start,
            pattern_tokens,
            &mut Vec::new(),
        )?;
        if is_match {
            return Ok(true);
        }
    }
    Ok(false)
}

pub struct PatternMatches {
    pub has_match: bool,
    pub ranges: Vec<(usize, usize)>,
}

pub fn match_all_patterns(
    input_line: &str,
    pattern_tokens: &[PatternToken],
) -> Result<PatternMatches> {
    let mut start = 0;
    let mut has_match = false;
    let mut ranges = Vec::new();

    while start <= input_line.len() {
        let (is_match, end) = match_tokens(
            input_line.as_bytes(),
            start,
            pattern_tokens,
            &mut Vec::new(),
        )?;
        has_match |= is_match;
        if !is_match || end <= start {
            start += 1; // Move to the next character if no match or empty match
            continue;
        }
        ranges.push((start, end));
        start = end; // Move past the matched portion for the next search
    }
    Ok(PatternMatches { has_match, ranges })
}

fn match_tokens(
    input_bytes: &[u8],
    index: usize,
    tokens: &[PatternToken],
    captures: &mut Vec<(usize, usize)>,
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
                let (is_match, new_index) = match_tokens(
                    input_bytes,
                    candidate_index,
                    slice::from_ref(inner.as_ref()),
                    captures,
                )?;
                if !is_match {
                    break;
                }
                match_count += 1;
                positions.push(new_index);
                if new_index == candidate_index {
                    break; // Prevent infinite loop on empty matches
                }
                candidate_index = new_index;
            }
            if match_count < *min {
                return Ok((false, index));
            }
            for count in (*min..=match_count).rev() {
                let try_idx = positions[count];
                let (is_match, end) = match_tokens(input_bytes, try_idx, rest_tokens, captures)?;
                if is_match {
                    return Ok((true, end));
                }
            }
            Ok((false, index))
        }
        PatternToken::Alternation(alternatives) => {
            for alt_tokens in alternatives {
                let prev_len = captures.len();
                let mut combined_tokens = alt_tokens.clone();
                captures.push((index, index)); // Placeholder for the start of this alternative
                let (is_match, end) = match_tokens(input_bytes, index, &combined_tokens, captures)?;
                if !is_match {
                    captures.truncate(prev_len); // Backtrack captures if this alternative fails
                    continue;
                }
                combined_tokens.extend_from_slice(rest_tokens);
                captures[prev_len] = (index, end); // Update capture for this alternative
                let (is_match, end) = match_tokens(input_bytes, index, &combined_tokens, captures)?;
                if is_match {
                    return Ok((true, end));
                }
            }
            Ok((false, index))
        }
        PatternToken::Backreference(group_num) => {
            let (start, end) = captures
                .get(*group_num - 1)
                .ok_or_else(|| anyhow::anyhow!("Invalid backreference: \\{}", group_num))?;
            let group_len = end - start;
            if index + group_len > input_bytes.len()
                || input_bytes[*start..*end] != input_bytes[index..index + group_len]
            {
                return Ok((false, index));
            }
            match_tokens(input_bytes, index + group_len, rest_tokens, captures)
        }
        _ => {
            let (is_match, index) = token.matches(input_bytes, index);
            if !is_match {
                return Ok((false, index));
            }
            match_tokens(input_bytes, index, rest_tokens, captures)
        }
    }
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
            Self::Quantifier { .. } | Self::Alternation(_) | Self::Backreference(_) => {
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

#[cfg(test)]
mod tests {
    use super::{match_all_patterns, match_pattern};
    use crate::pattern;
    use anyhow::Result;

    fn assert_match_pattern(input_line: &str, pattern_text: &str, expected: bool) -> Result<()> {
        let tokens = pattern::parse_pattern(pattern_text)?;
        let is_match = match_pattern(input_line, &tokens)?;
        assert_eq!(is_match, expected);
        Ok(())
    }

    fn assert_match_all_patterns(
        input_line: &str,
        pattern_text: &str,
        expect_match: bool,
        expected_idx: &[(usize, usize)],
    ) -> Result<()> {
        let tokens = pattern::parse_pattern(pattern_text)?;
        let pattern_matches = match_all_patterns(input_line, &tokens)?;
        assert_eq!(pattern_matches.has_match, expect_match);
        assert_eq!(pattern_matches.ranges, expected_idx);
        Ok(())
    }

    #[test]
    fn start_anchor_matches_start_of_line() -> Result<()> {
        assert_match_pattern("abcdef", "^abc", true)
    }

    #[test]
    fn end_anchor_matches_end_of_line() -> Result<()> {
        assert_match_pattern("123abc", "abc$", true)
    }

    #[test]
    fn end_anchor_can_match_empty_position_at_end_of_line() -> Result<()> {
        assert_match_pattern("abc", "$", true)
    }

    #[test]
    fn end_anchor_can_match_empty_line() -> Result<()> {
        assert_match_pattern("", "$", true)
    }

    #[test]
    fn start_anchor_with_star_quantifier() -> Result<()> {
        assert_match_pattern("abc\n", "^*", true)
    }

    #[test]
    fn start_anchor_with_plus_quantifier() -> Result<()> {
        assert_match_pattern("abc", "^+", true)
    }

    #[test]
    fn start_anchor_with_optional_quantifier() -> Result<()> {
        assert_match_pattern("abc", "()*", true)
    }

    #[test]
    fn quantifier_with_range() -> Result<()> {
        assert_match_pattern("cabbageee_soup", "cabbage{2,4}_soup", true)
    }

    #[test]
    fn match_all_patterns_returns_non_overlapping_matches() -> Result<()> {
        assert_match_all_patterns("banana", "a", true, &[(1, 2), (3, 4), (5, 6)])
    }

    #[test]
    fn match_all_patterns_skips_empty_matches_for_star_quantifier() -> Result<()> {
        assert_match_all_patterns("abc", "a*", true, &[(0, 1)])
    }

    #[test]
    fn match_all_patterns_skips_empty_matches_for_optional_quantifier() -> Result<()> {
        assert_match_all_patterns("abc", "a?", true, &[(0, 1)])
    }
}
