use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};

use super::{convert_numbers_to_digits, format_number_commas, ProcessingOptions};

static SHORTHAND_REPLACEMENTS: Lazy<Vec<(Regex, &'static str)>> = Lazy::new(|| {
    [
        ("in my humble opinion", "imho"),
        ("for your information", "fyi"),
        ("as soon as possible", "asap"),
        ("to be honest", "tbh"),
        ("to be fair", "tbf"),
        ("by the way", "btw"),
        ("in my opinion", "imo"),
        ("i do not know", "idk"),
        ("i don't know", "idk"),
        ("let me know", "lmk"),
        ("be right back", "brb"),
        ("talk to you later", "ttyl"),
        ("oh my god", "omg"),
        ("laugh out loud", "lol"),
        ("not gonna lie", "ngl"),
        ("never mind", "nvm"),
    ]
    .into_iter()
    .map(|(phrase, replacement)| {
        let pattern = format!(r"\b{}\b", regex::escape(phrase));
        (
            RegexBuilder::new(&pattern)
                .case_insensitive(true)
                .build()
                .unwrap(),
            replacement,
        )
    })
    .collect()
});

static SPACE_BEFORE_PUNCTUATION_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\s+([,.;:!?])").unwrap());

static CONSECUTIVE_PUNCTUATION_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"[,.;:!?]{2,}").unwrap());

pub fn apply_casual_mode(text: &str) -> String {
    let mut result = text.to_lowercase();

    result.retain(|c| !matches!(c, '.' | ',' | '!' | '?' | ';' | ':'));

    result = result.split_whitespace().collect::<Vec<_>>().join(" ");

    result
}

pub fn apply_shorthand_mode(text: &str) -> String {
    let mut result = text.to_string();

    for (pattern, replacement) in SHORTHAND_REPLACEMENTS.iter() {
        result = pattern.replace_all(&result, *replacement).to_string();
    }

    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn filter_words(text: &str, filter_words: &[String]) -> Option<String> {
    let mut result = text.to_string();

    let mut filters: Vec<&str> = filter_words
        .iter()
        .map(|w| w.trim())
        .filter(|w| !w.is_empty())
        .collect();
    filters.sort_by(|a, b| b.split_whitespace().count().cmp(&a.split_whitespace().count()));

    for trimmed in filters {

        let pattern = format!(r"\b{}\b", regex::escape(trimmed));
        if let Ok(re) = RegexBuilder::new(&pattern).case_insensitive(true).build() {
            result = re.replace_all(&result, "").to_string();
        }
    }

    result = SPACE_BEFORE_PUNCTUATION_REGEX
        .replace_all(&result, "$1")
        .to_string();
    result = CONSECUTIVE_PUNCTUATION_REGEX
        .replace_all(&result, |captures: &regex::Captures| {
            captures
                .get(0)
                .and_then(|value| value.as_str().chars().next())
                .map(|character| character.to_string())
                .unwrap_or_default()
        })
        .to_string();

    result = result.split_whitespace().collect::<Vec<_>>().join(" ");
    result = result.trim_matches(|c: char| c.is_whitespace()).to_string();

    if result.is_empty()
        || result
            .chars()
            .all(|character| character.is_ascii_punctuation() || character.is_whitespace())
    {
        None
    } else {
        Some(result)
    }
}

pub fn capitalize_sentences(text: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in text.chars() {
        if capitalize_next && c.is_alphabetic() {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }

        if c == '.' || c == '!' || c == '?' {
            capitalize_next = true;
        }
    }

    result
}

pub fn apply_smart_quotes(text: &str) -> String {
    let mut result = String::new();
    let mut in_quotes = false;

    for c in text.chars() {
        if c == '"' {
            if in_quotes {
                result.push('"');
            } else {
                result.push('"');
            }
            in_quotes = !in_quotes;
        } else {
            result.push(c);
        }
    }

    result
}

pub fn process_text(text: &str, options: &ProcessingOptions) -> Option<String> {
    let mut result = text.to_string();

    if options.accounting_mode {
        result = convert_numbers_to_digits(&result);
    }

    if options.accounting_comma {
        result = format_number_commas(&result);
    }

    if let Some(filtered) = filter_words(&result, &options.filter_words) {
        result = filtered;
    } else {
        return None;
    }

    if options.casual_mode {
        result = apply_casual_mode(&result);
    }

    if options.capitalize_sentences && !options.casual_mode {
        result = capitalize_sentences(&result);
    }

    if options.smart_quotes {
        result = apply_smart_quotes(&result);
    }

    if options.shorthand_mode {
        result = apply_shorthand_mode(&result);
    }

    result = result.trim().to_string();

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_text_formats_large_spoken_numbers_for_accounting() {
        let options = ProcessingOptions {
            accounting_mode: true,
            accounting_comma: true,
            ..ProcessingOptions::default()
        };

        assert_eq!(process_text("2 million", &options), Some("2,000,000".to_string()));
        assert_eq!(
            process_text("two million three hundred thousand", &options),
            Some("2,300,000".to_string())
        );
    }

    #[test]
    fn test_shorthand_mode_replaces_common_phrases() {
        assert_eq!(
            apply_shorthand_mode("to be honest by the way i do not know"),
            "tbh btw idk"
        );
        assert_eq!(apply_shorthand_mode("Let me know"), "lmk");
    }

    #[test]
    fn test_process_text_applies_shorthand_without_casual_mode() {
        let options = ProcessingOptions {
            shorthand_mode: true,
            ..ProcessingOptions::default()
        };

        assert_eq!(
            process_text("To be honest, let me know.", &options),
            Some("tbh, lmk.".to_string())
        );
    }

    #[test]
    fn test_casual_mode() {
        assert_eq!(apply_casual_mode("Hello, World!"), "hello world");
    }

    #[test]
    fn test_filter_words() {
        let filters = vec!["thank you".to_string()];
        assert_eq!(
            filter_words("hello thank you world", &filters),
            Some("hello world".to_string())
        );
    }

    #[test]
    fn test_filter_words_is_case_insensitive_and_cleans_punctuation() {
        let filters = vec!["thank you".to_string()];
        assert_eq!(filter_words("Thank you.", &filters), None);
        assert_eq!(
            filter_words("hello, Thank You, world", &filters),
            Some("hello, world".to_string())
        );
    }

    #[test]
    fn test_capitalize_sentences() {
        assert_eq!(
            capitalize_sentences("hello. how are you?"),
            "Hello. How are you?"
        );
    }
}
