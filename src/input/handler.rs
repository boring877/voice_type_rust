//! Input handling functionality
//!
//! Contains functions for global hotkeys and text typing simulation.

use anyhow::{Context, Result};
use rdev::{Button, EventType, Key, SimulateError, listen, simulate};
use std::sync::Arc;

use super::{HotkeyState, InputBinding};

/// Parse a hotkey string into an input binding.
///
/// Supports keyboard names like `shift`, mouse bindings like `mouse:left`,
/// and persisted OS-specific codes such as `keycode:123`.
pub fn parse_hotkey(hotkey_str: &str) -> Result<InputBinding> {
    let normalized = hotkey_str.to_lowercase().trim().to_string();

    if let Some(code) = normalized.strip_prefix("keycode:") {
        let parsed = code
            .parse::<u32>()
            .context("Keycode hotkeys must use a numeric value")?;
        return Ok(InputBinding::Key(Key::Unknown(parsed)));
    }

    if let Some(mouse_name) = normalized.strip_prefix("mouse:") {
        let button = match mouse_name {
            "left" => Button::Left,
            "right" => Button::Right,
            "middle" => Button::Middle,
            value if value.starts_with("button-") => {
                let raw = value
                    .trim_start_matches("button-")
                    .parse::<u8>()
                    .context("Mouse button hotkeys must use a numeric suffix")?;
                Button::Unknown(raw)
            }
            _ => anyhow::bail!("Unknown mouse hotkey: {}", hotkey_str),
        };
        return Ok(InputBinding::Mouse(button));
    }

    let key = match normalized.as_str() {
        "shift" | "shiftleft" => Key::ShiftLeft,
        "shiftright" => Key::ShiftRight,
        "ctrl" | "control" | "ctrlleft" => Key::ControlLeft,
        "ctrlright" | "controlright" => Key::ControlRight,
        "alt" | "altleft" => Key::Alt,
        "altright" => Key::AltGr,
        "meta" | "win" | "command" | "super" => Key::MetaLeft,
        "metaright" | "winright" => Key::MetaRight,
        "space" => Key::Space,
        "tab" => Key::Tab,
        "enter" | "return" => Key::Return,
        "escape" | "esc" => Key::Escape,
        "backspace" => Key::Backspace,
        "delete" | "del" => Key::Delete,
        "up" => Key::UpArrow,
        "down" => Key::DownArrow,
        "left" => Key::LeftArrow,
        "right" => Key::RightArrow,
        "home" => Key::Home,
        "end" => Key::End,
        "pageup" => Key::PageUp,
        "pagedown" => Key::PageDown,
        "capslock" => Key::CapsLock,
        "printscreen" => Key::PrintScreen,
        "scrolllock" => Key::ScrollLock,
        "pause" => Key::Pause,
        "numlock" => Key::NumLock,
        "insert" => Key::Insert,
        "`" | "backquote" => Key::BackQuote,
        "-" | "minus" => Key::Minus,
        "=" | "equal" => Key::Equal,
        "[" | "leftbracket" => Key::LeftBracket,
        "]" | "rightbracket" => Key::RightBracket,
        ";" | "semicolon" => Key::SemiColon,
        "'" | "quote" => Key::Quote,
        "\\" | "backslash" => Key::BackSlash,
        "intlbackslash" => Key::IntlBackslash,
        "," | "comma" => Key::Comma,
        "." | "dot" | "period" => Key::Dot,
        "/" | "slash" => Key::Slash,
        "kpplus" | "numpad+" => Key::KpPlus,
        "kpmultiply" | "numpad*" => Key::KpMultiply,
        "kpdivide" | "numpad/" => Key::KpDivide,
        "kp0" => Key::Kp0,
        "kp1" => Key::Kp1,
        "kp2" => Key::Kp2,
        "kp3" => Key::Kp3,
        "kp4" => Key::Kp4,
        "kp5" => Key::Kp5,
        "kp6" => Key::Kp6,
        "kp7" => Key::Kp7,
        "kp8" => Key::Kp8,
        "kp9" => Key::Kp9,
        "fn" => Key::Function,
        "f1" => Key::F1,
        "f2" => Key::F2,
        "f3" => Key::F3,
        "f4" => Key::F4,
        "f5" => Key::F5,
        "f6" => Key::F6,
        "f7" => Key::F7,
        "f8" => Key::F8,
        "f9" => Key::F9,
        "f10" => Key::F10,
        "f11" => Key::F11,
        "f12" => Key::F12,
        "0" => Key::Num0,
        "1" => Key::Num1,
        "2" => Key::Num2,
        "3" => Key::Num3,
        "4" => Key::Num4,
        "5" => Key::Num5,
        "6" => Key::Num6,
        "7" => Key::Num7,
        "8" => Key::Num8,
        "9" => Key::Num9,
        "a" => Key::KeyA,
        "b" => Key::KeyB,
        "c" => Key::KeyC,
        "d" => Key::KeyD,
        "e" => Key::KeyE,
        "f" => Key::KeyF,
        "g" => Key::KeyG,
        "h" => Key::KeyH,
        "i" => Key::KeyI,
        "j" => Key::KeyJ,
        "k" => Key::KeyK,
        "l" => Key::KeyL,
        "m" => Key::KeyM,
        "n" => Key::KeyN,
        "o" => Key::KeyO,
        "p" => Key::KeyP,
        "q" => Key::KeyQ,
        "r" => Key::KeyR,
        "s" => Key::KeyS,
        "t" => Key::KeyT,
        "u" => Key::KeyU,
        "v" => Key::KeyV,
        "w" => Key::KeyW,
        "x" => Key::KeyX,
        "y" => Key::KeyY,
        "z" => Key::KeyZ,
        s if s.len() == 1 => {
            let c = s.chars().next().unwrap();
            Key::Unknown(c as u32)
        }
        _ => anyhow::bail!("Unknown hotkey: {}", hotkey_str),
    };

    Ok(InputBinding::Key(key))
}

/// Convert a hotkey string into a user-facing label.
pub fn describe_hotkey(hotkey_str: &str) -> String {
    parse_hotkey(hotkey_str)
        .map(|binding| binding.label())
        .unwrap_or_else(|_| hotkey_str.to_uppercase())
}

fn browser_code_to_config_value(code: &str) -> Option<String> {
    let trimmed = code.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(letter) = trimmed.strip_prefix("Key") {
        if letter.len() == 1 && letter.chars().all(|character| character.is_ascii_alphabetic()) {
            return Some(letter.to_ascii_lowercase());
        }
    }

    if let Some(digit) = trimmed.strip_prefix("Digit") {
        if digit.len() == 1 && digit.chars().all(|character| character.is_ascii_digit()) {
            return Some(digit.to_string());
        }
    }

    if let Some(number) = trimmed.strip_prefix('F') {
        if !number.is_empty() && number.chars().all(|character| character.is_ascii_digit()) {
            return Some(format!("f{}", number));
        }
    }

    if let Some(number) = trimmed.strip_prefix("Numpad") {
        if number.len() == 1 && number.chars().all(|character| character.is_ascii_digit()) {
            return Some(format!("kp{}", number));
        }
    }

    let normalized = match trimmed {
        "ShiftLeft" => "shift",
        "ShiftRight" => "shiftright",
        "ControlLeft" => "ctrl",
        "ControlRight" => "ctrlright",
        "AltLeft" => "alt",
        "AltRight" => "altright",
        "MetaLeft" => "meta",
        "MetaRight" => "metaright",
        "Space" => "space",
        "Tab" => "tab",
        "Enter" | "NumpadEnter" => "enter",
        "Escape" => "escape",
        "Backspace" => "backspace",
        "Delete" => "delete",
        "Insert" => "insert",
        "Home" => "home",
        "End" => "end",
        "PageUp" => "pageup",
        "PageDown" => "pagedown",
        "ArrowUp" => "up",
        "ArrowDown" => "down",
        "ArrowLeft" => "left",
        "ArrowRight" => "right",
        "CapsLock" => "capslock",
        "Backquote" => "`",
        "Minus" => "-",
        "Equal" => "=",
        "BracketLeft" => "[",
        "BracketRight" => "]",
        "Semicolon" => ";",
        "Quote" => "'",
        "Backslash" => "\\",
        "IntlBackslash" => "intlbackslash",
        "Comma" => ",",
        "Period" => ".",
        "Slash" => "/",
        "NumpadAdd" => "kpplus",
        "NumpadSubtract" => "-",
        "NumpadMultiply" => "kpmultiply",
        "NumpadDivide" => "kpdivide",
        _ => return None,
    };

    Some(normalized.to_string())
}

/// Normalize a browser keyboard event code into the same input binding model used by the app.
pub fn normalize_browser_key_code(code: &str) -> Option<InputBinding> {
    let config_value = browser_code_to_config_value(code)?;
    parse_hotkey(&config_value).ok()
}

/// Normalize a browser mouse button index into the same input binding model used by the app.
pub fn normalize_browser_mouse_button(button: i16) -> Option<InputBinding> {
    match button {
        0 => Some(InputBinding::Mouse(Button::Left)),
        1 => Some(InputBinding::Mouse(Button::Middle)),
        2 => Some(InputBinding::Mouse(Button::Right)),
        value if (3..=i16::from(u8::MAX) + 2).contains(&value) => {
            Some(InputBinding::Mouse(Button::Unknown((value - 2) as u8)))
        }
        _ => None,
    }
}

/// Start the global hotkey listener
///
/// Spawns a thread that listens for global keyboard events.
/// Returns the hotkey state that can be queried for recording status.
///
/// # Arguments
/// * `hotkey_str` - The hotkey to use for push-to-talk (e.g., "shift")
pub fn start_listener(hotkey_str: &str) -> Result<Arc<HotkeyState>> {
    let target_binding = parse_hotkey(hotkey_str).context("Invalid hotkey configuration")?;

    let state = Arc::new(HotkeyState::new(target_binding));
    let state_clone = Arc::clone(&state);

    // Store the hotkey string for logging
    let hotkey_owned = hotkey_str.to_string();

    // Spawn listener thread
    std::thread::spawn(move || {
        tracing::info!("Starting global hotkey listener for: {}", hotkey_owned);

        let callback = move |event: rdev::Event| match event.event_type {
            EventType::KeyPress(key) => {
                state_clone.press_key(key);
            }
            EventType::KeyRelease(key) => {
                state_clone.release_key(key);
            }
            EventType::ButtonPress(button) => {
                state_clone.press_button(button);
            }
            EventType::ButtonRelease(button) => {
                state_clone.release_button(button);
            }
            _ => {}
        };

        if let Err(e) = listen(callback) {
            tracing::error!("Hotkey listener error: {:?}", e);
        }
    });

    Ok(state)
}

/// Check if push-to-talk is currently held
///
/// Non-blocking check of the hotkey state.
pub fn is_recording(state: &Arc<HotkeyState>) -> bool {
    state.is_recording()
}

/// Type text at the current cursor position
///
/// Simulates keystrokes to type the given text.
/// Uses clipboard paste for efficiency with longer text.
///
/// # Arguments
/// * `text` - The text to type
/// * `use_clipboard` - If true, uses Ctrl+V for longer text (faster)
///
/// # Platform Notes
/// - On Windows, uses Ctrl+V for clipboard paste
/// - Character-by-character typing is slower but works everywhere
pub fn type_text(text: &str, leave_in_clipboard: bool) -> Result<()> {
    if text.is_empty() {
        return Ok(());
    }

    // For short text, type character by character
    // For longer text, use clipboard paste
    if text.len() > 10 {
        type_via_clipboard(text, leave_in_clipboard)
    } else {
        type_character_by_character(text)?;
        if leave_in_clipboard {
            set_clipboard_text(text)?;
        }
        Ok(())
    }
}

/// Type text character by character
fn type_character_by_character(text: &str) -> Result<()> {
    for c in text.chars() {
        // Convert char to key events
        if let Err(e) = send_char(c) {
            tracing::warn!("Failed to type character '{}': {:?}", c, e);
        }

        // Small delay between keystrokes
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    Ok(())
}

/// Type text via clipboard paste
///
/// Faster for longer text, but requires clipboard access.
fn type_via_clipboard(text: &str, leave_in_clipboard: bool) -> Result<()> {
    // Save current clipboard content
    let old_clipboard = arboard::Clipboard::new()
        .and_then(|mut cb| cb.get_text())
        .unwrap_or_default();

    // Copy new text to clipboard
    set_clipboard_text(text)?;

    // Small delay for clipboard to update
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Paste with Ctrl+V
    let paste_result = simulate_key_combination(&[Key::ControlLeft, Key::KeyV]);

    if !leave_in_clipboard {
        // Restore old clipboard content after paste has had time to land
        std::thread::sleep(std::time::Duration::from_millis(200));
        let _ = arboard::Clipboard::new().and_then(|mut cb| cb.set_text(old_clipboard));
    }

    paste_result
}

/// Simulate a key combination (e.g., Ctrl+V)
pub fn simulate_key_combination(keys: &[Key]) -> Result<()> {
    // Press all keys
    for &key in keys {
        simulate(&EventType::KeyPress(key))
            .map_err(|e| anyhow::anyhow!("Key press failed: {:?}", e))?;
    }

    // Release in reverse order
    for &key in keys.iter().rev() {
        simulate(&EventType::KeyRelease(key))
            .map_err(|e| anyhow::anyhow!("Key release failed: {:?}", e))?;
    }

    Ok(())
}

/// Send a single character as keystrokes
fn send_char(c: char) -> Result<(), SimulateError> {
    // Map common characters to keys
    let event = match c {
        'a'..='z' | 'A'..='Z' | '0'..='9' => {
            // Use Unknown key with Unicode code point
            EventType::KeyPress(Key::Unknown(c as u32))
        }
        ' ' => EventType::KeyPress(Key::Space),
        '\n' => EventType::KeyPress(Key::Return),
        '.' => EventType::KeyPress(Key::Unknown('.' as u32)),
        ',' => EventType::KeyPress(Key::Unknown(',' as u32)),
        '!' => EventType::KeyPress(Key::Unknown('!' as u32)),
        '?' => EventType::KeyPress(Key::Unknown('?' as u32)),
        _ => EventType::KeyPress(Key::Unknown(c as u32)),
    };

    simulate(&event)?;

    // Release the key
    let release_event = match event {
        EventType::KeyPress(k) => EventType::KeyRelease(k),
        _ => event,
    };
    simulate(&release_event)?;

    Ok(())
}

fn set_clipboard_text(text: &str) -> Result<()> {
    arboard::Clipboard::new()
        .and_then(|mut cb| cb.set_text(text.to_string()))
        .context("Failed to set clipboard")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hotkey_supports_alphanumeric_keys() {
        assert_eq!(parse_hotkey("a").unwrap(), InputBinding::Key(Key::KeyA));
        assert_eq!(parse_hotkey("z").unwrap(), InputBinding::Key(Key::KeyZ));
        assert_eq!(parse_hotkey("1").unwrap(), InputBinding::Key(Key::Num1));
        assert_eq!(parse_hotkey("0").unwrap(), InputBinding::Key(Key::Num0));
    }

    #[test]
    fn parse_hotkey_supports_common_punctuation_and_numpad_keys() {
        assert_eq!(parse_hotkey("/").unwrap(), InputBinding::Key(Key::Slash));
        assert_eq!(
            parse_hotkey("`").unwrap(),
            InputBinding::Key(Key::BackQuote)
        );
        assert_eq!(parse_hotkey("kp7").unwrap(), InputBinding::Key(Key::Kp7));
        assert_eq!(
            parse_hotkey("kpplus").unwrap(),
            InputBinding::Key(Key::KpPlus)
        );
    }

    #[test]
    fn parse_hotkey_supports_mouse_buttons_and_raw_keycodes() {
        assert_eq!(
            parse_hotkey("mouse:left").unwrap(),
            InputBinding::Mouse(Button::Left)
        );
        assert_eq!(
            parse_hotkey("mouse:button-2").unwrap(),
            InputBinding::Mouse(Button::Unknown(2))
        );
        assert_eq!(
            parse_hotkey("keycode:183").unwrap(),
            InputBinding::Key(Key::Unknown(183))
        );
    }

    #[test]
    fn browser_normalization_supports_pattern_based_codes() {
        assert_eq!(
            normalize_browser_key_code("KeyA").unwrap(),
            InputBinding::Key(Key::KeyA)
        );
        assert_eq!(
            normalize_browser_key_code("Digit4").unwrap(),
            InputBinding::Key(Key::Num4)
        );
        assert_eq!(
            normalize_browser_key_code("F12").unwrap(),
            InputBinding::Key(Key::F12)
        );
        assert_eq!(
            normalize_browser_key_code("Numpad7").unwrap(),
            InputBinding::Key(Key::Kp7)
        );
    }

    #[test]
    fn browser_normalization_supports_mouse_buttons() {
        assert_eq!(
            normalize_browser_mouse_button(0).unwrap(),
            InputBinding::Mouse(Button::Left)
        );
        assert_eq!(
            normalize_browser_mouse_button(4).unwrap(),
            InputBinding::Mouse(Button::Unknown(2))
        );
        assert!(normalize_browser_mouse_button(-1).is_none());
    }
}
