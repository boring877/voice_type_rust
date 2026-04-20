export type ThemeMode = "dark" | "light";
export type BackgroundMode = "solid" | "gradient" | "image";
export type HudSide = "left" | "right";
export type HudBackgroundMode = "glass" | "image";
export type RuntimeState = "ready" | "recording" | "processing" | "done" | "error";

export interface Config {
  api_key: string;
  mic_index: number | null;
  hotkey: string;
  accounting_mode: boolean;
  accounting_comma: boolean;
  casual_mode: boolean;
  shorthand_mode: boolean;
  style: string;
  filter_words: string[];
  theme: string;
  background_mode: string;
  background_color: string;
  background_gradient_start: string;
  background_gradient_end: string;
  panel_color: string;
  section_color: string;
  background_image_path: string;
  meter_color_start: string;
  meter_color_end: string;
  auto_stop: boolean;
  silence_threshold: number;
  noise_threshold: number;
  language: string;
  transcription_model: string;
  always_on_top: boolean;
  hud_enabled: boolean;
  hud_side: HudSide;
  hud_show_state: boolean;
  hud_show_app_name: boolean;
  hud_show_description: boolean;
  hud_show_meter: boolean;
  hud_background_mode: HudBackgroundMode;
  hud_background_path: string;
  hud_background_color: string;
  auto_copy: boolean;
  provider: string;
}

export interface AppInfo {
  appName: string;
  version: string;
  backend: string;
}

export interface RuntimeSnapshot {
  appState: RuntimeState;
  audioLevel: number;
  config: Config;
  hotkeyLabel: string;
  statusText: string;
}

export interface HotkeyCapturePayload {
  configValue: string;
  label: string;
}

export interface HistoryEntry {
  text: string;
  timestamp: number;
  wordCount: number;
}

export interface UpdateInfo {
  currentVersion: string;
  latestVersion: string;
  releaseUrl: string;
  updateAvailable: boolean;
}
