//! Input handling module
//!
//! Handles global hotkeys (push-to-talk) and text typing simulation.
//! Uses the `rdev` crate for cross-platform input simulation.

// Re-export HotkeyState from types module
pub use crate::types::input::{HotkeyState, InputBinding};

// Declare submodules
mod handler;

// Re-export functions from handler
pub use handler::{
    describe_hotkey, is_recording, normalize_browser_key_code, normalize_browser_mouse_button,
    parse_hotkey, start_listener, type_text,
};
