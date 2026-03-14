import type { CSSProperties } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { BackgroundMode, Config, RuntimeState, ThemeMode } from "../types";
import defaultBackground from "../assets/default-background.jpg";

export function normalizeTheme(theme: string): ThemeMode {
  return theme === "light" ? "light" : "dark";
}

export function normalizeBackgroundMode(mode: string): BackgroundMode {
  if (mode === "gradient" || mode === "image") {
    return mode;
  }

  return "solid";
}

export function runtimeBadge(state: RuntimeState) {
  switch (state) {
    case "recording":
      return { label: "Recording", tone: "state-pill recording" };
    case "processing":
      return { label: "Transcribing", tone: "state-pill processing" };
    case "done":
      return { label: "Ready to type", tone: "state-pill done" };
    case "error":
      return { label: "Needs attention", tone: "state-pill error" };
    default:
      return { label: "Standing by", tone: "state-pill ready" };
  }
}

export function safeBackgroundUrl(path: string): string | null {
  const trimmed = path.trim();
  if (!trimmed) {
    return null;
  }

  try {
    return convertFileSrc(trimmed);
  } catch {
    return null;
  }
}

function isHexColor(value: string): boolean {
  return /^#[0-9a-fA-F]{6}$/.test(value.trim());
}

export function normalizedColor(value: string, fallback: string): string {
  const trimmed = value.trim();
  return isHexColor(trimmed) ? trimmed : fallback;
}

export function buildBackgroundStyle(config: Config, backgroundUrl: string | null): CSSProperties {
  const mode = normalizeBackgroundMode(config.background_mode);
  const solid = normalizedColor(config.background_color, "#111315");
  const gradientStart = normalizedColor(config.background_gradient_start, "#202938");
  const gradientEnd = normalizedColor(config.background_gradient_end, "#0f1117");

  if (mode === "image") {
    const resolvedImage = backgroundUrl ?? defaultBackground;
    return {
      backgroundColor: solid,
      backgroundImage: `url("${resolvedImage}")`,
      backgroundPosition: "center",
      backgroundSize: "cover",
      backgroundRepeat: "no-repeat"
    };
  }

  if (mode === "gradient") {
    return {
      backgroundColor: solid,
      backgroundImage: `linear-gradient(145deg, ${gradientStart}, ${gradientEnd})`
    };
  }

  return {
    backgroundColor: solid
  };
}

function hexToRgba(hex: string, alpha: number): string {
  const normalized = normalizedColor(hex, "#111315").slice(1);
  const red = Number.parseInt(normalized.slice(0, 2), 16);
  const green = Number.parseInt(normalized.slice(2, 4), 16);
  const blue = Number.parseInt(normalized.slice(4, 6), 16);
  return `rgba(${red}, ${green}, ${blue}, ${alpha})`;
}

function mixHex(base: string, target: string, amount: number): string {
  const normalizedBase = normalizedColor(base, "#111315").slice(1);
  const normalizedTarget = normalizedColor(target, "#ffffff").slice(1);
  const ratio = Math.max(0, Math.min(1, amount));

  const mixChannel = (startIndex: number) => {
    const start = Number.parseInt(normalizedBase.slice(startIndex, startIndex + 2), 16);
    const end = Number.parseInt(normalizedTarget.slice(startIndex, startIndex + 2), 16);
    const mixed = Math.round(start + (end - start) * ratio);
    return mixed.toString(16).padStart(2, "0");
  };

  return `#${mixChannel(0)}${mixChannel(2)}${mixChannel(4)}`;
}

export function buildShellStyle(config: Config, theme: ThemeMode): CSSProperties {
  const defaults =
    theme === "light"
      ? {
          panel: "#ffffff",
          section: "#f7f8fa",
          field: "#eff2f6",
          meterStart: "#e88bad",
          meterEnd: "#6f96da"
        }
      : {
          panel: "#111418",
          section: "#161a1f",
          field: "#0b0e12",
          meterStart: "#ff99c4",
          meterEnd: "#7faef6"
        };

  const panel = normalizedColor(config.panel_color, defaults.panel);
  const section = normalizedColor(config.section_color, defaults.section);
  const field = normalizedColor(config.section_color, defaults.field);
  const meterStart = normalizedColor(config.meter_color_start, defaults.meterStart);
  const meterEnd = normalizedColor(config.meter_color_end, defaults.meterEnd);
  const meterTop = mixHex(meterStart, "#ffffff", theme === "light" ? 0.28 : 0.42);

  return {
    "--panel-bg": hexToRgba(panel, theme === "light" ? 0.8 : 0.82),
    "--card-bg": hexToRgba(panel, theme === "light" ? 0.72 : 0.78),
    "--section-bg": hexToRgba(section, theme === "light" ? 0.84 : 0.84),
    "--field": hexToRgba(field, theme === "light" ? 0.92 : 0.68),
    "--meter-color-top": meterTop,
    "--meter-color-start": meterStart,
    "--meter-color-end": meterEnd
  } as CSSProperties;
}
