import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { buildShellStyle, normalizeTheme } from "./lib/appearance";
import { fallbackRuntime } from "./lib/options";
import type { RuntimeSnapshot } from "./types";
import { HudCard } from "./components/HudCard";

function clampUnit(value: number) {
  return Math.max(0, Math.min(1, value));
}

function smoothingFactor(deltaMs: number, durationMs: number) {
  return 1 - Math.exp(-deltaMs / durationMs);
}

export default function HudApp() {
  const [runtime, setRuntime] = useState<RuntimeSnapshot>(fallbackRuntime);
  const [meter, setMeter] = useState({ level: 0, peak: 0 });
  const [backgroundUrl, setBackgroundUrl] = useState<string | null>(null);

  const audioLevelTargetRef = useRef(runtime.audioLevel);
  const runtimeStateRef = useRef(runtime.appState);

  useEffect(() => {
    audioLevelTargetRef.current = runtime.audioLevel;
  }, [runtime.audioLevel]);

  useEffect(() => {
    runtimeStateRef.current = runtime.appState;
  }, [runtime.appState]);

  useEffect(() => {
    let frameId = 0;
    let lastFrame = performance.now();

    const animate = (now: number) => {
      const deltaMs = Math.min(64, Math.max(8, now - lastFrame));
      lastFrame = now;

      setMeter((current) => {
        const recording = runtimeStateRef.current === "recording";
        const target = recording ? clampUnit(audioLevelTargetRef.current) : 0;
        const levelFactor = smoothingFactor(deltaMs, target > current.level ? 110 : 320);
        const nextLevel = current.level + (target - current.level) * levelFactor;

        let nextPeak = current.peak;
        if (recording) {
          const peakTarget = Math.max(target, nextLevel);
          const peakFactor = smoothingFactor(deltaMs, peakTarget >= current.peak ? 64 : 520);
          nextPeak = current.peak + (peakTarget - current.peak) * peakFactor;
        } else {
          nextPeak = current.peak * Math.exp(-deltaMs / 180);
        }

        return {
          level: nextLevel < 0.002 ? 0 : clampUnit(nextLevel),
          peak: nextPeak < 0.006 ? 0 : clampUnit(nextPeak)
        };
      });

      frameId = window.requestAnimationFrame(animate);
    };

    frameId = window.requestAnimationFrame(animate);
    return () => window.cancelAnimationFrame(frameId);
  }, []);

  useEffect(() => {
    let mounted = true;
    let unlistenRuntime: (() => void | Promise<void>) | null = null;

    const boot = async () => {
      unlistenRuntime = await listen<RuntimeSnapshot>("voice-type://runtime", (event) => {
        if (mounted) {
          setRuntime(event.payload);
        }
      });

      const snapshot = await invoke<RuntimeSnapshot>("get_runtime_snapshot");
      if (mounted) {
        setRuntime(snapshot);
      }
    };

    void boot();

    return () => {
      mounted = false;
      if (unlistenRuntime) {
        void unlistenRuntime();
      }
    };
  }, []);

  useEffect(() => {
    let cancelled = false;
    const path = runtime.config.hud_background_path.trim();

    if (!path) {
      setBackgroundUrl(null);
      return;
    }

    void invoke<string>("load_background_image_data_url", { path })
      .then((dataUrl) => {
        if (!cancelled) {
          setBackgroundUrl(dataUrl);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setBackgroundUrl(null);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [runtime.config.hud_background_path]);

  const theme = normalizeTheme(runtime.config.theme);
  const shellStyle = buildShellStyle(runtime.config, theme);
  const animatedMeter = runtime.appState === "recording" || runtime.appState === "processing";
  const meterLevel = runtime.appState === "processing" ? 0.22 : meter.level;
  const meterPeak = runtime.appState === "processing" ? 0.5 : meter.peak;

  return (
    <div className={`hud-shell theme-${theme}`} style={shellStyle}>
      <HudCard
        active={animatedMeter}
        backgroundUrl={backgroundUrl}
        config={runtime.config}
        level={meterLevel}
        peak={meterPeak}
        snapshot={runtime}
      />
    </div>
  );
}
