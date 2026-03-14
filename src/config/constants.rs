//! Configuration-related constants

/// Human-friendly application name used in UI and logs.
pub const APP_DISPLAY_NAME: &str = "Voice Type";

/// Config directory name under the platform-specific config root.
pub const CONFIG_DIR_NAME: &str = "voice-type";

/// Process-wide single instance identifier.
pub const SINGLE_INSTANCE_ID: &str = "voice_type_2_5";

/// Default hotkey for push-to-talk
pub const DEFAULT_HOTKEY: &str = "shift";

/// Default silence threshold in seconds for auto-stop
pub const DEFAULT_SILENCE_THRESHOLD: f32 = 2.0;

/// Default noise threshold for audio level detection
pub const DEFAULT_NOISE_THRESHOLD: f32 = 0.01;

/// Theme identifiers stored in config.
pub const THEME_DARK: &str = "dark";
pub const THEME_LIGHT: &str = "light";

/// Background presentation modes stored in config.
pub const BACKGROUND_MODE_SOLID: &str = "solid";
pub const BACKGROUND_MODE_GRADIENT: &str = "gradient";
pub const BACKGROUND_MODE_IMAGE: &str = "image";

/// Default background colors for the desktop shell.
pub const DEFAULT_BACKGROUND_COLOR: &str = "#111315";
pub const DEFAULT_BACKGROUND_GRADIENT_START: &str = "#202938";
pub const DEFAULT_BACKGROUND_GRADIENT_END: &str = "#0f1117";

/// Default transcription language.
pub const DEFAULT_LANGUAGE: &str = "auto";

/// Supported provider identifiers stored in config.
pub const PROVIDER_GROQ: &str = "groq";

/// Grammar quality profile identifiers.
pub const GRAMMAR_PROFILE_FAST: &str = "fast";
pub const GRAMMAR_PROFILE_BALANCED: &str = "balanced";
pub const GRAMMAR_PROFILE_QUALITY: &str = "quality";

/// Floating HUD placement identifiers.
pub const HUD_SIDE_LEFT: &str = "left";
pub const HUD_SIDE_RIGHT: &str = "right";

/// Floating HUD background identifiers.
pub const HUD_BACKGROUND_MODE_GLASS: &str = "glass";
pub const HUD_BACKGROUND_MODE_IMAGE: &str = "image";
