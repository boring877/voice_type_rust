import type { AppInfo, Config, RuntimeSnapshot } from "../types";

export const modelOptions = [
  { value: "whisper-large-v3", label: "Whisper Large v3" }
];

export const styleOptions = [
  { value: "none", label: "None" },
  { value: "japanese_emojis", label: "Japanese emojis" },
  { value: "niko_style", label: "Niko style" },
];

export const languageLabels: Record<string, string> = {
  auto: "Auto-detect",
  en: "English",
  es: "Spanish",
  ar: "Arabic",
  el: "Greek",
  ja: "Japanese",
  zh: "Chinese",
  fr: "French",
  de: "German",
  pt: "Portuguese",
  hi: "Hindi",
  ko: "Korean",
  it: "Italian",
  ru: "Russian"
};

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

export function languageLabel(value: string): string {
  return languageLabels[value] ?? value.toUpperCase();
}

export function languageHint(value: string): string {
  if (value === "auto" || value === "") {
    return "Detects the language you speak each time — you can freely mix languages.";
  }
  return `Locks transcription to ${languageLabel(value)}. Speaking another language may give unexpected results.`;
}

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
  hud_pinned: false,
  hud_position_x: null,
  hud_position_y: null,
  auto_copy: false,
  provider: "groq"
};

/** App display name — must match APP_DISPLAY_NAME in src/config/constants.rs */
export const APP_DISPLAY_NAME = "Voice Type";

export const fallbackInfo: AppInfo = {
  appName: "Voice Type",
  version: "0.7.2",
  backend: "Rust + Tokio"
};

export const fallbackRuntime: RuntimeSnapshot = {
  appState: "ready",
  audioLevel: 0,
  config: fallbackConfig,
  hotkeyLabel: "Shift",
  statusText: "Hold Shift to speak..."
};
