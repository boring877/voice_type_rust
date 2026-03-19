import type { AppInfo, Config, RuntimeSnapshot } from "../types";

export const modelOptions = [
  { value: "whisper-large-v3", label: "Whisper Large v3" },
  { value: "whisper-large-v3-turbo", label: "Whisper Large v3 Turbo" }
];

export const styleOptions = [
  { value: "none", label: "None" },
  { value: "japanese_emojis", label: "Japanese emojis" },
  { value: "niko_style", label: "Niko style" },
  { value: "agent", label: "Agent Prompt" }
];

export const languageOptions = [
  "auto",
  "en",
  "es",
  "ar",
  "el",
  "ja",
  "zh",
  "fr",
  "de",
  "pt",
  "hi",
  "ko",
  "it",
  "ru"
];

export const fallbackConfig: Config = {
  api_key: "",
  mic_index: null,
  hotkey: "shift",
  accounting_mode: false,
  accounting_comma: false,
  casual_mode: false,
  shorthand_mode: false,
  style: "none",
  filter_words: [],
  theme: "dark",
  background_mode: "image",
  background_color: "#111315",
  background_gradient_start: "#202938",
  background_gradient_end: "#0f1117",
  panel_color: "",
  section_color: "",
  background_image_path: "",
  meter_color_start: "",
  meter_color_end: "",
  auto_stop: false,
  silence_threshold: 2,
  noise_threshold: 0.01,
  language: "auto",
  transcription_model: "whisper-large-v3-turbo",
  always_on_top: false,
  hud_enabled: true,
  hud_side: "right",
  hud_show_state: true,
  hud_show_app_name: true,
  hud_show_description: true,
  hud_show_meter: true,
  hud_background_mode: "image",
  hud_background_path: "",
  hud_background_color: "",
  auto_copy: false,
  provider: "groq"
};

export const fallbackInfo: AppInfo = {
  appName: "Voice Type",
  version: "0.2.0",
  backend: "Rust + Tokio"
};

export const fallbackRuntime: RuntimeSnapshot = {
  appState: "ready",
  audioLevel: 0,
  config: fallbackConfig,
  hotkeyLabel: "Shift",
  statusText: "Hold Shift to speak..."
};
