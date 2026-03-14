//! Text processing functionality
//!
//! Contains functions for post-processing transcribed text.

use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use super::ProcessingOptions;

/// Static regex patterns (compiled once)
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

const STYLE_NONE: &str = "none";
const STYLE_JAPANESE_EMOJIS: &str = "japanese_emojis";
const STYLE_JAPANESE_OMG_LEGACY: &str = "japanese_omg";
const STYLE_NIKO: &str = "niko_style";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StyleMood {
    Cheerful,
    Curious,
    Frustrated,
    Angry,
    Tired,
    Calm,
}

/// Convert number words to digits (Accounting Mode)
///
/// Examples:
/// - "one hundred twenty three" -> "123"
/// - "fifty dollars" -> "50 dollars"
/// - "two million" -> "2000000"
///
/// # Arguments
/// * `text` - Input text with potential number words
///
/// # Returns
/// Text with number words converted to digits
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

/// Add commas to large numbers
///
/// Examples:
/// - "1000" -> "1,000"
/// - "1000000" -> "1,000,000"
///
/// # Arguments
/// * `text` - Text containing numbers
///
/// # Returns
/// Text with commas added to numbers
pub fn format_number_commas(text: &str) -> String {
    let number_regex = Regex::new(r"\b(\d{4,})\b").unwrap();

    number_regex
        .replace_all(text, |caps: &regex::Captures| {
            let num: u64 = caps[1].parse().unwrap_or(0);
            format_with_commas(num)
        })
        .to_string()
}

/// Format a number with commas
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

/// Apply casual mode formatting
///
/// Converts text to lowercase and removes formal punctuation.
/// Examples:
/// - "Hello, World!" -> "hello world"
/// - "How are you?" -> "how are you"
///
/// # Arguments
/// * `text` - Input text
///
/// # Returns
/// Casually formatted text
pub fn apply_casual_mode(text: &str) -> String {
    let mut result = text.to_lowercase();

    // Strip punctuation for a looser chat-style output.
    result.retain(|c| !matches!(c, '.' | ',' | '!' | '?' | ';' | ':'));

    // Trim extra whitespace
    result = result.split_whitespace().collect::<Vec<_>>().join(" ");

    result
}

/// Apply chat-style shorthand replacements.
///
/// Examples:
/// - "to be honest" -> "tbh"
/// - "by the way" -> "btw"
pub fn apply_shorthand_mode(text: &str) -> String {
    let mut result = text.to_string();

    for (pattern, replacement) in SHORTHAND_REPLACEMENTS.iter() {
        result = pattern.replace_all(&result, *replacement).to_string();
    }

    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Apply a final style preset to the processed text.
pub fn apply_style_preset(text: &str, style: &str, language: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let mood = detect_style_mood(trimmed, language);

    match style {
        STYLE_JAPANESE_EMOJIS | STYLE_JAPANESE_OMG_LEGACY => {
            format!("{trimmed} {}", japanese_expression(trimmed, mood))
        }
        STYLE_NIKO => format!("{trimmed} {}", niko_expression(trimmed, mood)),
        STYLE_NONE => trimmed.to_string(),
        _ => trimmed.to_string(),
    }
}

fn ascii_lower_words(text: &str) -> Vec<String> {
    text.split(|character: char| !character.is_ascii_alphanumeric() && character != '\'')
        .filter(|token| !token.is_empty())
        .map(str::to_ascii_lowercase)
        .collect()
}

fn contains_word(words: &[String], needle: &str) -> bool {
    words.iter().any(|word| word == needle)
}

fn contains_any_word(words: &[String], needles: &[&str]) -> bool {
    needles.iter().any(|needle| contains_word(words, needle))
}

fn contains_phrase(words: &[String], phrase: &[&str]) -> bool {
    let phrase_len = phrase.len();
    phrase_len > 0
        && words.windows(phrase_len).any(|window| {
            window
                .iter()
                .map(String::as_str)
                .zip(phrase.iter().copied())
                .all(|(left, right)| left == right)
        })
}

fn uses_english_style_hints(language: &str) -> bool {
    let normalized = language.trim().to_ascii_lowercase();
    normalized.is_empty() || normalized == "auto" || normalized.starts_with("en")
}

fn detect_style_mood(text: &str, language: &str) -> StyleMood {
    let trimmed = text.trim();
    let has_question = trimmed.contains('?') || trimmed.contains('？');
    let has_exclamation = trimmed.contains('!') || trimmed.contains('！');
    let has_ellipsis = trimmed.contains("...") || trimmed.contains('…');

    if uses_english_style_hints(language) {
        let words = ascii_lower_words(trimmed);

        if contains_any_word(
            &words,
            &[
                "angry",
                "mad",
                "furious",
                "annoyed",
                "hate",
                "damn",
                "stupid",
                "wtf",
                "ugh",
                "grr",
            ],
        ) {
            return StyleMood::Angry;
        }

        if contains_any_word(&words, &["tired", "sleepy", "exhausted", "drained", "eepy"]) {
            return StyleMood::Tired;
        }

        if contains_phrase(&words, &["not", "really"])
            || contains_any_word(
                &words,
                &["whatever", "fine", "sigh", "meh", "unfortunately", "disappointing"],
            )
        {
            return StyleMood::Frustrated;
        }

        if has_question
            || contains_any_word(&words, &["why", "what", "how", "really", "huh", "maybe"])
        {
            return StyleMood::Curious;
        }

        if has_exclamation
            || contains_any_word(
                &words,
                &["yay", "great", "awesome", "love", "cute", "happy", "nice", "amazing"],
            )
        {
            return StyleMood::Cheerful;
        }
    }

    if has_question {
        StyleMood::Curious
    } else if has_exclamation {
        StyleMood::Cheerful
    } else if has_ellipsis {
        StyleMood::Tired
    } else {
        StyleMood::Calm
    }
}

fn stable_pick<'a>(seed: &str, options: &'a [&'a str]) -> &'a str {
    let mut hasher = DefaultHasher::new();
    seed.hash(&mut hasher);
    let index = (hasher.finish() as usize) % options.len();
    options[index]
}

fn japanese_expression(text: &str, mood: StyleMood) -> &'static str {
    match mood {
        StyleMood::Cheerful => stable_pick(text, &["(^_^)", "(≧▽≦)", "(ﾉ´ヮ`)ﾉ*: ･ﾟ"]),
        StyleMood::Curious => stable_pick(text, &["(・o・)?", "(・・? )", "(°ロ°)?"]),
        StyleMood::Frustrated => stable_pick(text, &["(-_-;)", "(¬_¬\")", "(￣ヘ￣)"]),
        StyleMood::Angry => stable_pick(text, &["(#`Д´)", "(`ω´)", "٩(╬ʘ益ʘ╬)۶"]),
        StyleMood::Tired => stable_pick(text, &["(=_=)", "(-_-) zzz", "(￣ω￣;)"]),
        StyleMood::Calm => stable_pick(text, &["(^_^)", "(´• ω •`)", "(￣ω￣)"]),
    }
}

fn niko_expression(text: &str, mood: StyleMood) -> &'static str {
    match mood {
        StyleMood::Cheerful => stable_pick(text, &["nya! (=^･ω･^=)", "nya! (=^･^=)", "nya! (=^‥^=)"]),
        StyleMood::Curious => stable_pick(text, &["nya? (=^･ω･^=)", "nya? /ᐠ｡‸｡ᐟ\\", "nya? (=｀ω´=)?"]),
        StyleMood::Frustrated => stable_pick(text, &["nya... (=｀ω´=)", "nya... (=`ω´=)", "nya... (=ω=;)"]),
        StyleMood::Angry => stable_pick(text, &["NYA! (=｀ω´=)", "nya! ლ(=`ω´=)ლ", "nya!! ᕦ(=`ω´=)ᕤ"]),
        StyleMood::Tired => stable_pick(text, &["nya... (=ω=)..", "munya... (=^-ω-^=)", "nya... (= ; ω ; =)"]),
        StyleMood::Calm => stable_pick(text, &["nya (=^･ω･^=)", "nya (=^･^=)", "nya (=^‥^=)"]),
    }
}

/// Filter out unwanted words/phrases
///
/// Removes specified words from the text. Useful for filtering
/// common hallucinations like "thank you".
///
/// # Arguments
/// * `text` - Input text
/// * `filter_words` - List of words/phrases to remove
///
/// # Returns
/// Filtered text (or None if text becomes empty)
pub fn filter_words(text: &str, filter_words: &[String]) -> Option<String> {
    let mut result = text.to_string();

    for word in filter_words {
        let trimmed = word.trim();
        if trimmed.is_empty() {
            continue;
        }

        let pattern = format!(r"\b{}\b", regex::escape(trimmed));
        if let Ok(re) = RegexBuilder::new(&pattern).case_insensitive(true).build() {
            result = re.replace_all(&result, "").to_string();
        }
    }

    // Clean up spaces left behind before punctuation after removing a phrase.
    if let Ok(re) = Regex::new(r"\s+([,.;:!?])") {
        result = re.replace_all(&result, "$1").to_string();
    }
    if let Ok(re) = Regex::new(r"[,.;:!?]{2,}") {
        result = re
            .replace_all(&result, |captures: &regex::Captures| {
                captures
                    .get(0)
                    .and_then(|value| value.as_str().chars().next())
                    .map(|character| character.to_string())
                    .unwrap_or_default()
            })
            .to_string();
    }

    // Clean up extra whitespace
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

/// Capitalize the first letter of sentences
///
/// Examples:
/// - "hello world. how are you?" -> "Hello world. How are you?"
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

        // Capitalize after sentence endings
        if c == '.' || c == '!' || c == '?' {
            capitalize_next = true;
        }
    }

    result
}

/// Apply smart quotes conversion
///
/// Converts straight quotes to curly quotes:
/// - "\"Hello\"" -> "\"Hello\"" (with smart quotes)
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

/// Full text processing pipeline
///
/// Applies all enabled transformations in order:
/// 1. Number conversion (if accounting_mode)
/// 2. Comma formatting (if accounting_comma)
/// 3. Word filtering
/// 4. Casual mode (if enabled)
/// 5. Sentence capitalization (if enabled)
/// 6. Smart quotes (if enabled)
/// 7. Shorthand mode (if enabled)
///
/// # Arguments
/// * `text` - Raw transcription from API
/// * `options` - Processing options
///
/// # Returns
/// Processed text ready for typing
pub fn process_text(text: &str, options: &ProcessingOptions) -> Option<String> {
    let mut result = text.to_string();

    // 1. Convert numbers to digits
    if options.accounting_mode {
        result = convert_numbers_to_digits(&result);
    }

    // 2. Add commas to numbers
    if options.accounting_comma {
        result = format_number_commas(&result);
    }

    // 3. Filter unwanted words
    if let Some(filtered) = filter_words(&result, &options.filter_words) {
        result = filtered;
    } else {
        return None;
    }

    // 4. Apply casual mode
    if options.casual_mode {
        result = apply_casual_mode(&result);
    }

    // 5. Capitalize sentences
    if options.capitalize_sentences && !options.casual_mode {
        result = capitalize_sentences(&result);
    }

    // 6. Apply smart quotes
    if options.smart_quotes {
        result = apply_smart_quotes(&result);
    }

    // 7. Apply shorthand replacements at the end so capitalization
    // does not turn chat abbreviations like "tbh" back into "Tbh".
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
    fn test_apply_style_preset_uses_japanese_expression_pool() {
        let cheerful = apply_style_preset("We did it!", STYLE_JAPANESE_EMOJIS, "en");
        let curious = apply_style_preset("Are you there?", STYLE_JAPANESE_EMOJIS, "en");

        assert!(
            [
                "We did it! (^_^)",
                "We did it! (≧▽≦)",
                "We did it! (ﾉ´ヮ`)ﾉ*: ･ﾟ",
            ]
            .contains(&cheerful.as_str())
        );
        assert!(
            [
                "Are you there? (・o・)?",
                "Are you there? (・・? )",
                "Are you there? (°ロ°)?",
            ]
            .contains(&curious.as_str())
        );
        assert_eq!(apply_style_preset("hello", STYLE_NONE, "en"), "hello");
    }

    #[test]
    fn test_apply_style_preset_uses_niko_expression_pool() {
        let cheerful = apply_style_preset("We did it!", STYLE_NIKO, "en");
        let curious = apply_style_preset("Are you there?", STYLE_NIKO, "en");

        assert!(
            [
                "We did it! nya! (=^･ω･^=)",
                "We did it! nya! (=^･^=)",
                "We did it! nya! (=^‥^=)",
            ]
            .contains(&cheerful.as_str())
        );
        assert!(
            [
                "Are you there? nya? (=^･ω･^=)",
                "Are you there? nya? /ᐠ｡‸｡ᐟ\\",
                "Are you there? nya? (=｀ω´=)?",
            ]
            .contains(&curious.as_str())
        );
    }

    #[test]
    fn test_apply_style_preset_uses_exact_english_word_matching() {
        let calm = apply_style_preset("Madeline is here", STYLE_JAPANESE_EMOJIS, "en");
        let also_calm = apply_style_preset("whatsoever", STYLE_JAPANESE_EMOJIS, "en");

        assert!(
            [
                "Madeline is here (^_^)",
                "Madeline is here (´• ω •`)",
                "Madeline is here (￣ω￣)",
            ]
            .contains(&calm.as_str())
        );
        assert!(
            [
                "whatsoever (^_^)",
                "whatsoever (´• ω •`)",
                "whatsoever (￣ω￣)",
            ]
            .contains(&also_calm.as_str())
        );
    }

    #[test]
    fn test_apply_style_preset_uses_punctuation_for_non_english_languages() {
        let cheerful = apply_style_preset("Hola!", STYLE_JAPANESE_EMOJIS, "es");
        let calm = apply_style_preset("I am angry about this", STYLE_JAPANESE_EMOJIS, "ja");

        assert!(
            ["Hola! (^_^)", "Hola! (≧▽≦)", "Hola! (ﾉ´ヮ`)ﾉ*: ･ﾟ"]
                .contains(&cheerful.as_str())
        );
        assert!(
            [
                "I am angry about this (^_^)",
                "I am angry about this (´• ω •`)",
                "I am angry about this (￣ω￣)",
            ]
            .contains(&calm.as_str())
        );
    }

    #[test]
    fn test_apply_style_preset_uses_multiple_japanese_moods() {
        let cheerful = apply_style_preset("We did it!", STYLE_JAPANESE_EMOJIS, "en");
        let angry = apply_style_preset("I am angry about this", STYLE_JAPANESE_EMOJIS, "en");

        assert!(
            ["(^_^)", "(≧▽≦)", "(ﾉ´ヮ`)ﾉ*: ･ﾟ"]
                .iter()
                .any(|suffix| cheerful.ends_with(suffix))
        );
        assert!(
            ["(#`Д´)", "(`ω´)", "٩(╬ʘ益ʘ╬)۶"]
                .iter()
                .any(|suffix| angry.ends_with(suffix))
        );
    }

    #[test]
    fn test_apply_style_preset_uses_niko_cat_moods() {
        let calm = apply_style_preset("hello", STYLE_NIKO, "en");
        let curious = apply_style_preset("what is this?", STYLE_NIKO, "en");

        assert!(calm.contains(" nya "));
        assert!(curious.contains(" nya? "));
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
