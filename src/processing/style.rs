use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
