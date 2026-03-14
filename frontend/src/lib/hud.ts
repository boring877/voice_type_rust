import type { CSSProperties } from "react";
import { normalizeTheme, normalizedColor } from "./appearance";
import type { Config, RuntimeSnapshot } from "../types";
import defaultHudBackground from "../assets/default-hud-background.jpg";

function hexToRgba(hex: string, alpha: number): string {
  const normalized = normalizedColor(hex, "#101419").slice(1);
  const red = Number.parseInt(normalized.slice(0, 2), 16);
  const green = Number.parseInt(normalized.slice(2, 4), 16);
  const blue = Number.parseInt(normalized.slice(4, 6), 16);
  return `rgba(${red}, ${green}, ${blue}, ${alpha})`;
}

export function normalizeHudBackgroundMode(mode: string): "glass" | "image" {
  return mode === "image" ? "image" : "glass";
}

export function hudHeadline(state: RuntimeSnapshot["appState"]) {
  switch (state) {
    case "recording":
      return "Listening now";
    case "processing":
      return "Turning speech into text";
    case "done":
      return "Finished";
    case "error":
      return "Needs attention";
    default:
      return "Standing by";
  }
}

function firstMeaningfulLine(text: string) {
  return (
    text
      .split(/\r?\n/u)
      .map((line) => line.trim())
      .find((line) => line.length > 0) ?? ""
  );
}

export function hudDescription(snapshot: RuntimeSnapshot) {
  if (snapshot.appState === "recording") {
    return `Release ${snapshot.hotkeyLabel} when you are done speaking.`;
  }

  if (snapshot.appState === "processing") {
    return firstMeaningfulLine(snapshot.statusText) || "Cleaning up your final transcript.";
  }

  return firstMeaningfulLine(snapshot.statusText) || "Voice Type is ready.";
}

export function buildHudCardStyle(config: Config, backgroundUrl: string | null): CSSProperties {
  const theme = normalizeTheme(config.theme);
  const tint = normalizedColor(
    config.hud_background_color,
    theme === "light" ? "#ffffff" : "#101419"
  );
  const base = hexToRgba(tint, theme === "light" ? 0.76 : 0.84);

  if (normalizeHudBackgroundMode(config.hud_background_mode) === "image") {
    const resolvedImage = backgroundUrl ?? defaultHudBackground;
    const topOverlay = hexToRgba(tint, theme === "light" ? 0.48 : 0.44);
    const bottomOverlay = hexToRgba(tint, theme === "light" ? 0.72 : 0.82);

    return {
      backgroundColor: base,
      backgroundImage: `linear-gradient(180deg, ${topOverlay}, ${bottomOverlay}), url("${resolvedImage}")`,
      backgroundPosition: "center",
      backgroundRepeat: "no-repeat",
      backgroundSize: "cover"
    };
  }

  return {
    backgroundColor: base,
    backgroundImage: `linear-gradient(160deg, ${hexToRgba(tint, theme === "light" ? 0.14 : 0.22)}, ${hexToRgba(tint, 0.02)})`
  };
}
