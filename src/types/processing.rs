//! Processing types for Voice Type
//!
//! Contains types related to text processing options.

/// Options for text processing
#[derive(Debug, Clone)]
pub struct ProcessingOptions {
    /// Convert number words to digits
    pub accounting_mode: bool,
    /// Add commas to large numbers
    pub accounting_comma: bool,
    /// Enable casual mode formatting
    pub casual_mode: bool,
    /// Replace common phrases with shorthand/slang equivalents
    pub shorthand_mode: bool,
    /// Capitalize first letter of sentences
    pub capitalize_sentences: bool,
    /// Use smart quotes
    pub smart_quotes: bool,
    /// Words to filter out
    pub filter_words: Vec<String>,
}

impl Default for ProcessingOptions {
    fn default() -> Self {
        Self {
            accounting_mode: false,
            accounting_comma: false,
            casual_mode: false,
            shorthand_mode: false,
            capitalize_sentences: true,
            smart_quotes: false,
            filter_words: Vec::new(),
        }
    }
}
