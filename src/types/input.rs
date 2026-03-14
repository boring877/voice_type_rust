//! Input types for Voice Type
//!
//! Contains types related to hotkey state and input handling.

use rdev::{Button, Key};
use std::collections::HashSet;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

/// A single push-to-talk binding.
///
/// We keep this broader than a plain keyboard key so the app can support
/// mouse buttons and vendor-specific "extra button" events when `rdev`
/// exposes them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputBinding {
    Key(Key),
    Mouse(Button),
}

impl InputBinding {
    /// User-facing label shown in the settings UI and status text.
    pub fn label(&self) -> String {
        match self {
            Self::Key(key) => key_label(*key),
            Self::Mouse(button) => match button {
                Button::Left => "Mouse Left".to_string(),
                Button::Right => "Mouse Right".to_string(),
                Button::Middle => "Mouse Middle".to_string(),
                Button::Unknown(1) => "Mouse Button 4".to_string(),
                Button::Unknown(2) => "Mouse Button 5".to_string(),
                Button::Unknown(code) => format!("Mouse Button {}", code),
            },
        }
    }

    /// Stable config value written to disk.
    pub fn config_value(&self) -> String {
        match self {
            Self::Key(key) => key_config_value(*key),
            Self::Mouse(button) => match button {
                Button::Left => "mouse:left".to_string(),
                Button::Right => "mouse:right".to_string(),
                Button::Middle => "mouse:middle".to_string(),
                Button::Unknown(code) => format!("mouse:button-{}", code),
            },
        }
    }
}

fn key_label(key: Key) -> String {
    match key {
        Key::ShiftLeft => "Shift".to_string(),
        Key::ShiftRight => "Right Shift".to_string(),
        Key::ControlLeft => "Ctrl".to_string(),
        Key::ControlRight => "Right Ctrl".to_string(),
        Key::Alt => "Alt".to_string(),
        Key::AltGr => "AltGr".to_string(),
        Key::MetaLeft => "Meta".to_string(),
        Key::MetaRight => "Right Meta".to_string(),
        Key::Space => "Space".to_string(),
        Key::Tab => "Tab".to_string(),
        Key::Return | Key::KpReturn => "Enter".to_string(),
        Key::Escape => "Escape".to_string(),
        Key::Backspace => "Backspace".to_string(),
        Key::Delete | Key::KpDelete => "Delete".to_string(),
        Key::UpArrow => "Arrow Up".to_string(),
        Key::DownArrow => "Arrow Down".to_string(),
        Key::LeftArrow => "Arrow Left".to_string(),
        Key::RightArrow => "Arrow Right".to_string(),
        Key::Home => "Home".to_string(),
        Key::End => "End".to_string(),
        Key::PageUp => "Page Up".to_string(),
        Key::PageDown => "Page Down".to_string(),
        Key::CapsLock => "Caps Lock".to_string(),
        Key::PrintScreen => "Print Screen".to_string(),
        Key::ScrollLock => "Scroll Lock".to_string(),
        Key::Pause => "Pause".to_string(),
        Key::NumLock => "Num Lock".to_string(),
        Key::BackQuote => "`".to_string(),
        Key::Minus | Key::KpMinus => "-".to_string(),
        Key::Equal => "=".to_string(),
        Key::LeftBracket => "[".to_string(),
        Key::RightBracket => "]".to_string(),
        Key::SemiColon => ";".to_string(),
        Key::Quote => "'".to_string(),
        Key::BackSlash => "\\".to_string(),
        Key::IntlBackslash => "Intl Backslash".to_string(),
        Key::Comma => ",".to_string(),
        Key::Dot => ".".to_string(),
        Key::Slash | Key::KpDivide => "/".to_string(),
        Key::Insert => "Insert".to_string(),
        Key::KpPlus => "Numpad +".to_string(),
        Key::KpMultiply => "Numpad *".to_string(),
        Key::Kp0 => "Numpad 0".to_string(),
        Key::Kp1 => "Numpad 1".to_string(),
        Key::Kp2 => "Numpad 2".to_string(),
        Key::Kp3 => "Numpad 3".to_string(),
        Key::Kp4 => "Numpad 4".to_string(),
        Key::Kp5 => "Numpad 5".to_string(),
        Key::Kp6 => "Numpad 6".to_string(),
        Key::Kp7 => "Numpad 7".to_string(),
        Key::Kp8 => "Numpad 8".to_string(),
        Key::Kp9 => "Numpad 9".to_string(),
        Key::Function => "Fn".to_string(),
        Key::Num0 => "0".to_string(),
        Key::Num1 => "1".to_string(),
        Key::Num2 => "2".to_string(),
        Key::Num3 => "3".to_string(),
        Key::Num4 => "4".to_string(),
        Key::Num5 => "5".to_string(),
        Key::Num6 => "6".to_string(),
        Key::Num7 => "7".to_string(),
        Key::Num8 => "8".to_string(),
        Key::Num9 => "9".to_string(),
        Key::KeyA => "A".to_string(),
        Key::KeyB => "B".to_string(),
        Key::KeyC => "C".to_string(),
        Key::KeyD => "D".to_string(),
        Key::KeyE => "E".to_string(),
        Key::KeyF => "F".to_string(),
        Key::KeyG => "G".to_string(),
        Key::KeyH => "H".to_string(),
        Key::KeyI => "I".to_string(),
        Key::KeyJ => "J".to_string(),
        Key::KeyK => "K".to_string(),
        Key::KeyL => "L".to_string(),
        Key::KeyM => "M".to_string(),
        Key::KeyN => "N".to_string(),
        Key::KeyO => "O".to_string(),
        Key::KeyP => "P".to_string(),
        Key::KeyQ => "Q".to_string(),
        Key::KeyR => "R".to_string(),
        Key::KeyS => "S".to_string(),
        Key::KeyT => "T".to_string(),
        Key::KeyU => "U".to_string(),
        Key::KeyV => "V".to_string(),
        Key::KeyW => "W".to_string(),
        Key::KeyX => "X".to_string(),
        Key::KeyY => "Y".to_string(),
        Key::KeyZ => "Z".to_string(),
        Key::F1 => "F1".to_string(),
        Key::F2 => "F2".to_string(),
        Key::F3 => "F3".to_string(),
        Key::F4 => "F4".to_string(),
        Key::F5 => "F5".to_string(),
        Key::F6 => "F6".to_string(),
        Key::F7 => "F7".to_string(),
        Key::F8 => "F8".to_string(),
        Key::F9 => "F9".to_string(),
        Key::F10 => "F10".to_string(),
        Key::F11 => "F11".to_string(),
        Key::F12 => "F12".to_string(),
        Key::Unknown(code) => {
            if let Some(character) = char::from_u32(code).filter(|c| c.is_ascii_graphic()) {
                character.to_ascii_uppercase().to_string()
            } else {
                format!("Custom key ({})", code)
            }
        }
    }
}

fn key_config_value(key: Key) -> String {
    match key {
        Key::ShiftLeft => "shift".to_string(),
        Key::ShiftRight => "shiftright".to_string(),
        Key::ControlLeft => "ctrl".to_string(),
        Key::ControlRight => "ctrlright".to_string(),
        Key::Alt => "alt".to_string(),
        Key::AltGr => "altright".to_string(),
        Key::MetaLeft => "meta".to_string(),
        Key::MetaRight => "metaright".to_string(),
        Key::Space => "space".to_string(),
        Key::Tab => "tab".to_string(),
        Key::Return | Key::KpReturn => "enter".to_string(),
        Key::Escape => "escape".to_string(),
        Key::Backspace => "backspace".to_string(),
        Key::Delete | Key::KpDelete => "delete".to_string(),
        Key::UpArrow => "up".to_string(),
        Key::DownArrow => "down".to_string(),
        Key::LeftArrow => "left".to_string(),
        Key::RightArrow => "right".to_string(),
        Key::Home => "home".to_string(),
        Key::End => "end".to_string(),
        Key::PageUp => "pageup".to_string(),
        Key::PageDown => "pagedown".to_string(),
        Key::CapsLock => "capslock".to_string(),
        Key::PrintScreen => "printscreen".to_string(),
        Key::ScrollLock => "scrolllock".to_string(),
        Key::Pause => "pause".to_string(),
        Key::NumLock => "numlock".to_string(),
        Key::BackQuote => "`".to_string(),
        Key::Minus | Key::KpMinus => "-".to_string(),
        Key::Equal => "=".to_string(),
        Key::LeftBracket => "[".to_string(),
        Key::RightBracket => "]".to_string(),
        Key::SemiColon => ";".to_string(),
        Key::Quote => "'".to_string(),
        Key::BackSlash => "\\".to_string(),
        Key::IntlBackslash => "intlbackslash".to_string(),
        Key::Comma => ",".to_string(),
        Key::Dot => ".".to_string(),
        Key::Slash | Key::KpDivide => "/".to_string(),
        Key::Insert => "insert".to_string(),
        Key::KpPlus => "kpplus".to_string(),
        Key::KpMultiply => "kpmultiply".to_string(),
        Key::Kp0 => "kp0".to_string(),
        Key::Kp1 => "kp1".to_string(),
        Key::Kp2 => "kp2".to_string(),
        Key::Kp3 => "kp3".to_string(),
        Key::Kp4 => "kp4".to_string(),
        Key::Kp5 => "kp5".to_string(),
        Key::Kp6 => "kp6".to_string(),
        Key::Kp7 => "kp7".to_string(),
        Key::Kp8 => "kp8".to_string(),
        Key::Kp9 => "kp9".to_string(),
        Key::Function => "fn".to_string(),
        Key::Num0 => "0".to_string(),
        Key::Num1 => "1".to_string(),
        Key::Num2 => "2".to_string(),
        Key::Num3 => "3".to_string(),
        Key::Num4 => "4".to_string(),
        Key::Num5 => "5".to_string(),
        Key::Num6 => "6".to_string(),
        Key::Num7 => "7".to_string(),
        Key::Num8 => "8".to_string(),
        Key::Num9 => "9".to_string(),
        Key::KeyA => "a".to_string(),
        Key::KeyB => "b".to_string(),
        Key::KeyC => "c".to_string(),
        Key::KeyD => "d".to_string(),
        Key::KeyE => "e".to_string(),
        Key::KeyF => "f".to_string(),
        Key::KeyG => "g".to_string(),
        Key::KeyH => "h".to_string(),
        Key::KeyI => "i".to_string(),
        Key::KeyJ => "j".to_string(),
        Key::KeyK => "k".to_string(),
        Key::KeyL => "l".to_string(),
        Key::KeyM => "m".to_string(),
        Key::KeyN => "n".to_string(),
        Key::KeyO => "o".to_string(),
        Key::KeyP => "p".to_string(),
        Key::KeyQ => "q".to_string(),
        Key::KeyR => "r".to_string(),
        Key::KeyS => "s".to_string(),
        Key::KeyT => "t".to_string(),
        Key::KeyU => "u".to_string(),
        Key::KeyV => "v".to_string(),
        Key::KeyW => "w".to_string(),
        Key::KeyX => "x".to_string(),
        Key::KeyY => "y".to_string(),
        Key::KeyZ => "z".to_string(),
        Key::F1 => "f1".to_string(),
        Key::F2 => "f2".to_string(),
        Key::F3 => "f3".to_string(),
        Key::F4 => "f4".to_string(),
        Key::F5 => "f5".to_string(),
        Key::F6 => "f6".to_string(),
        Key::F7 => "f7".to_string(),
        Key::F8 => "f8".to_string(),
        Key::F9 => "f9".to_string(),
        Key::F10 => "f10".to_string(),
        Key::F11 => "f11".to_string(),
        Key::F12 => "f12".to_string(),
        Key::Unknown(code) => {
            if let Some(character) = char::from_u32(code).filter(|c| c.is_ascii_graphic()) {
                character.to_string().to_lowercase()
            } else {
                format!("keycode:{}", code)
            }
        }
    }
}

/// State for tracking hotkey presses
///
/// Thread-safe state shared between the global listener and the app.
/// This is public so the main app can query recording state.
#[derive(Debug)]
pub struct HotkeyState {
    /// Currently pressed keys
    pub pressed_keys: Mutex<HashSet<Key>>,
    /// Whether the push-to-talk key is currently held
    pub is_recording: AtomicBool,
    /// The configured push-to-talk input binding.
    pub target_binding: Mutex<InputBinding>,
    /// Whether the next low-level input press should be captured for settings.
    pub capture_next_input: AtomicBool,
    /// Whether capture mode is armed and ready to consume the next press.
    pub capture_ready: AtomicBool,
    /// Earliest point where a new capture can arm after the activation click.
    pub capture_arm_at: Mutex<Option<Instant>>,
    /// The most recent binding captured from the low-level listener.
    pub captured_binding: Mutex<Option<InputBinding>>,
}

impl HotkeyState {
    /// Create a new hotkey state with the given target binding.
    pub fn new(target_binding: InputBinding) -> Self {
        Self {
            pressed_keys: Mutex::new(HashSet::new()),
            is_recording: AtomicBool::new(false),
            target_binding: Mutex::new(target_binding),
            capture_next_input: AtomicBool::new(false),
            capture_ready: AtomicBool::new(false),
            capture_arm_at: Mutex::new(None),
            captured_binding: Mutex::new(None),
        }
    }

    /// Get the current target binding.
    pub fn get_target_binding(&self) -> InputBinding {
        self.target_binding
            .lock()
            .map(|binding| *binding)
            .unwrap_or(InputBinding::Key(Key::ShiftLeft))
    }

    /// Update the configured push-to-talk binding at runtime.
    pub fn set_target_binding(&self, binding: InputBinding) {
        if let Ok(mut target_binding) = self.target_binding.lock() {
            *target_binding = binding;
        }

        // Reset the pressed state so switching bindings never leaves recording stuck on.
        self.is_recording.store(false, Ordering::Relaxed);
        if let Ok(mut keys) = self.pressed_keys.lock() {
            keys.clear();
        }
    }

    /// Enter low-level capture mode for the next pressed input.
    pub fn begin_capture(&self) {
        self.capture_next_input.store(true, Ordering::Relaxed);
        self.capture_ready.store(false, Ordering::Relaxed);
        if let Ok(mut capture_arm_at) = self.capture_arm_at.lock() {
            *capture_arm_at = Some(Instant::now() + Duration::from_millis(180));
        }
        if let Ok(mut captured) = self.captured_binding.lock() {
            *captured = None;
        }
    }

    /// Arm capture mode after the UI has finished the activation click.
    pub fn arm_capture(&self) {
        if self.capture_next_input.load(Ordering::Relaxed) {
            let can_arm = self
                .capture_arm_at
                .lock()
                .map(|mut capture_arm_at| match *capture_arm_at {
                    Some(arm_at) if Instant::now() < arm_at => false,
                    _ => {
                        *capture_arm_at = None;
                        true
                    }
                })
                .unwrap_or(true);

            if can_arm {
                self.capture_ready.store(true, Ordering::Relaxed);
            }
        }
    }

    /// Cancel any pending low-level input capture.
    pub fn cancel_capture(&self) {
        self.capture_next_input.store(false, Ordering::Relaxed);
        self.capture_ready.store(false, Ordering::Relaxed);
        if let Ok(mut capture_arm_at) = self.capture_arm_at.lock() {
            *capture_arm_at = None;
        }
    }

    /// Returns true while the listener is waiting for the next pressed input.
    pub fn is_capturing(&self) -> bool {
        self.capture_next_input.load(Ordering::Relaxed)
    }

    /// Returns true when capture mode is armed and the next press will be saved.
    pub fn is_capture_ready(&self) -> bool {
        self.capture_ready.load(Ordering::Relaxed)
    }

    /// Take the last binding captured by the low-level listener.
    pub fn take_captured_binding(&self) -> Option<InputBinding> {
        self.captured_binding
            .lock()
            .ok()
            .and_then(|mut captured| captured.take())
    }

    fn try_capture_binding(&self, binding: InputBinding) -> bool {
        if self.capture_next_input.load(Ordering::Relaxed)
            && self.capture_ready.load(Ordering::Relaxed)
        {
            self.capture_next_input.store(false, Ordering::Relaxed);
            self.capture_ready.store(false, Ordering::Relaxed);
            if let Ok(mut captured) = self.captured_binding.lock() {
                *captured = Some(binding);
            }
            self.is_recording.store(false, Ordering::Relaxed);
            return true;
        }

        false
    }

    /// Record a key press.
    pub fn press_key(&self, key: Key) {
        if self.try_capture_binding(InputBinding::Key(key)) {
            return;
        }

        if let Ok(mut keys) = self.pressed_keys.lock() {
            keys.insert(key);
        }

        if self.get_target_binding() == InputBinding::Key(key) {
            self.is_recording.store(true, Ordering::Relaxed);
        }
    }

    /// Record a key release.
    pub fn release_key(&self, key: Key) {
        if let Ok(mut keys) = self.pressed_keys.lock() {
            keys.remove(&key);
        }

        if self.get_target_binding() == InputBinding::Key(key) {
            self.is_recording.store(false, Ordering::Relaxed);
        }
    }

    /// Record a mouse button press.
    pub fn press_button(&self, button: Button) {
        if self.try_capture_binding(InputBinding::Mouse(button)) {
            return;
        }

        if self.get_target_binding() == InputBinding::Mouse(button) {
            self.is_recording.store(true, Ordering::Relaxed);
        }
    }

    /// Record a mouse button release.
    pub fn release_button(&self, button: Button) {
        if self.get_target_binding() == InputBinding::Mouse(button) {
            self.is_recording.store(false, Ordering::Relaxed);
        }
    }

    /// Check if push-to-talk is currently active.
    pub fn is_recording(&self) -> bool {
        self.is_recording.load(Ordering::Relaxed)
    }
}
