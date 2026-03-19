use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};

static NUMBER_WORD_REGEX: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(
        r"\b(zero|one|two|three|four|five|six|seven|eight|nine|ten|eleven|twelve|thirteen|fourteen|fifteen|sixteen|seventeen|eighteen|nineteen|twenty|thirty|forty|fifty|sixty|seventy|eighty|ninety|hundred|thousand|million|billion)\b",
    )
    .case_insensitive(true)
    .build()
    .unwrap()
});

static NUMBER_SEQUENCE_REGEX: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(
        r"\b(?:\d+|zero|one|two|three|four|five|six|seven|eight|nine|ten|eleven|twelve|thirteen|fourteen|fifteen|sixteen|seventeen|eighteen|nineteen|twenty|thirty|forty|fifty|sixty|seventy|eighty|ninety|hundred|thousand|million|billion|and)(?:[-\s]+(?:\d+|zero|one|two|three|four|five|six|seven|eight|nine|ten|eleven|twelve|thirteen|fourteen|fifteen|sixteen|seventeen|eighteen|nineteen|twenty|thirty|forty|fifty|sixty|seventy|eighty|ninety|hundred|thousand|million|billion|and))*\b",
    )
    .case_insensitive(true)
    .build()
    .unwrap()
});

static NUMBER_COMMAS_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b(\d{4,})\b").unwrap());

pub fn convert_numbers_to_digits(text: &str) -> String {
    if !NUMBER_WORD_REGEX.is_match(text) {
        return text.to_string();
    }

    NUMBER_SEQUENCE_REGEX
        .replace_all(text, |captures: &regex::Captures| {
            let matched = captures.get(0).map(|value| value.as_str()).unwrap_or_default();
            render_number_phrase(matched).unwrap_or_else(|| matched.to_string())
        })
        .to_string()
}

fn render_number_phrase(phrase: &str) -> Option<String> {
    let normalized_tokens = phrase
        .to_ascii_lowercase()
        .replace('-', " ")
        .split_whitespace()
        .filter(|token| *token != "and")
        .map(str::to_string)
        .collect::<Vec<_>>();

    let has_scale_token = normalized_tokens.iter().any(|token| {
        matches!(
            token.as_str(),
            "hundred" | "thousand" | "million" | "billion"
        )
    });
    let has_digit_token = normalized_tokens
        .iter()
        .any(|token| token.chars().all(|character| character.is_ascii_digit()));

    if has_digit_token && !has_scale_token {
        return None;
    }

    if normalized_tokens.len() > 1
        && normalized_tokens
            .iter()
            .all(|token| digit_like_value(token.as_str()).is_some())
    {
        return Some(
            normalized_tokens
                .iter()
                .filter_map(|token| digit_like_value(token.as_str()))
                .map(|value| value.to_string())
                .collect::<Vec<_>>()
                .join(" "),
        );
    }

    parse_number_phrase(phrase).map(|value| value.to_string())
}

fn parse_number_phrase(phrase: &str) -> Option<u64> {
    let normalized = phrase.to_ascii_lowercase().replace('-', " ");
    let mut total = 0_u64;
    let mut current_group = 0_u64;
    let mut saw_any_number = false;
    let mut group_has_base_value = false;

    for token in normalized.split_whitespace() {
        if token == "and" {
            continue;
        }

        if let Some(value) = base_number_value(token) {
            current_group = current_group.checked_add(value)?;
            saw_any_number = true;
            group_has_base_value = true;
            continue;
        }

        if token == "hundred" {
            if !group_has_base_value {
                return None;
            }

            current_group = current_group.checked_mul(100)?;
            saw_any_number = true;
            continue;
        }

        if let Some(scale) = scale_number_value(token) {
            if !group_has_base_value {
                return None;
            }

            total = total.checked_add(current_group.checked_mul(scale)?)?;
            current_group = 0;
            group_has_base_value = false;
            saw_any_number = true;
            continue;
        }

        return None;
    }

    if !saw_any_number {
        None
    } else {
        total.checked_add(current_group)
    }
}

fn unit_digit_value(token: &str) -> Option<u8> {
    match token {
        "zero" => Some(0),
        "one" => Some(1),
        "two" => Some(2),
        "three" => Some(3),
        "four" => Some(4),
        "five" => Some(5),
        "six" => Some(6),
        "seven" => Some(7),
        "eight" => Some(8),
        "nine" => Some(9),
        _ => None,
    }
}

fn digit_like_value(token: &str) -> Option<u64> {
    token
        .parse::<u64>()
        .ok()
        .filter(|value| *value < 10)
        .or_else(|| unit_digit_value(token).map(u64::from))
}

fn base_number_value(token: &str) -> Option<u64> {
    token.parse::<u64>().ok().or(match token {
        "zero" => Some(0),
        "one" => Some(1),
        "two" => Some(2),
        "three" => Some(3),
        "four" => Some(4),
        "five" => Some(5),
        "six" => Some(6),
        "seven" => Some(7),
        "eight" => Some(8),
        "nine" => Some(9),
        "ten" => Some(10),
        "eleven" => Some(11),
        "twelve" => Some(12),
        "thirteen" => Some(13),
        "fourteen" => Some(14),
        "fifteen" => Some(15),
        "sixteen" => Some(16),
        "seventeen" => Some(17),
        "eighteen" => Some(18),
        "nineteen" => Some(19),
        "twenty" => Some(20),
        "thirty" => Some(30),
        "forty" => Some(40),
        "fifty" => Some(50),
        "sixty" => Some(60),
        "seventy" => Some(70),
        "eighty" => Some(80),
        "ninety" => Some(90),
        _ => None,
    })
}

fn scale_number_value(token: &str) -> Option<u64> {
    match token {
        "thousand" => Some(1_000),
        "million" => Some(1_000_000),
        "billion" => Some(1_000_000_000),
        _ => None,
    }
}

pub fn format_number_commas(text: &str) -> String {
    NUMBER_COMMAS_REGEX
        .replace_all(text, |caps: &regex::Captures| {
            let num: u64 = match caps[1].parse() {
                Ok(n) => n,
                Err(_) => return caps[0].to_string(),
            };
            format_with_commas(num)
        })
        .to_string()
}

fn format_with_commas(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();

    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }

    result.chars().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_numbers() {
        assert_eq!(convert_numbers_to_digits("one two three"), "1 2 3");
        assert_eq!(convert_numbers_to_digits("ten dollars"), "10 dollars");
        assert_eq!(convert_numbers_to_digits("2 million"), "2000000");
        assert_eq!(
            convert_numbers_to_digits("two million three hundred thousand and five"),
            "2300005"
        );
    }

    #[test]
    fn test_convert_numbers_handles_hyphenated_compounds() {
        assert_eq!(convert_numbers_to_digits("twenty-one pilots"), "21 pilots");
    }

    #[test]
    fn test_format_commas() {
        assert_eq!(format_number_commas("1000"), "1,000");
        assert_eq!(format_number_commas("1000000"), "1,000,000");
    }
}
