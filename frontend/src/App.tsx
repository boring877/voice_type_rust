import { useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import {
  buildBackgroundStyle,
  buildShellStyle,
  normalizeBackgroundMode,
  normalizeTheme
} from "./lib/appearance";
import { configsEqual, parseFilterWords } from "./lib/config";
import { fallbackConfig, fallbackRuntime } from "./lib/options";
import {
  AdvancedSection,
  QuickStartSection,
  SettingsSection,
  SetupSection
} from "./components/sections";
import { HistorySection } from "./components/HistorySection";
import { HudStageSection } from "./components/HudStageSection";
import { OnboardingOverlay } from "./components/OnboardingOverlay";
import { useAutosave } from "./hooks/useAutosave";
import { useBackgroundImage } from "./hooks/useBackgroundImage";
import { useBoot } from "./hooks/useBoot";
import { useHotkeyCapture } from "./hooks/useHotkeyCapture";
import { useRuntimePolling } from "./hooks/useRuntimePolling";
import type { Config, RuntimeSnapshot } from "./types";

export default function App() {
  const [config, setConfig] = useState<Config>(fallbackConfig);
  const [savedConfig, setSavedConfig] = useState<Config>(fallbackConfig);
  const [runtime, setRuntime] = useState<RuntimeSnapshot>(fallbackRuntime);
  const [hotkeyLabel, setHotkeyLabel] = useState(fallbackRuntime.hotkeyLabel);
  const [error, setError] = useState<string | null>(null);
  const [advancedMode, setAdvancedMode] = useState(false);

  const configRef = useRef(config);
  const savedConfigRef = useRef(savedConfig);
  const prevAppStateRef = useRef(runtime.appState);

  useEffect(() => {
    configRef.current = config;
  }, [config]);

  useEffect(() => {
    savedConfigRef.current = savedConfig;
  }, [savedConfig]);

  useEffect(() => {
    const prev = prevAppStateRef.current;
    const curr = runtime.appState;

    if (curr === "recording" && prev !== "recording") {
      void invoke("play_beep", { frequency: 800, durationMs: 100 });
    } else if (prev === "recording" && curr !== "recording") {
      void invoke("play_beep", { frequency: 600, durationMs: 80 });
    }

    prevAppStateRef.current = curr;
  }, [runtime.appState]);

  function applyRuntimeSnapshot(snapshot: RuntimeSnapshot) {
    const shouldAdoptDraft = configsEqual(configRef.current, savedConfigRef.current);

    setRuntime(snapshot);
    savedConfigRef.current = snapshot.config;
    setSavedConfig(snapshot.config);

    if (shouldAdoptDraft) {
      configRef.current = snapshot.config;
      setConfig(snapshot.config);
      setHotkeyLabel(snapshot.hotkeyLabel);
    }
  }

  const {
    capturingHotkey,
    captureMessage,
    applyCapturedHotkey,
    startHotkeyCapture,
    stopHotkeyCapture
  } = useHotkeyCapture(setError, setConfig, setHotkeyLabel);

  const { loading, appInfo, micNames } = useBoot(
    setError,
    applyRuntimeSnapshot,
    applyCapturedHotkey
  );

  useRuntimePolling(loading, runtime.appState, applyRuntimeSnapshot);

  const backgroundUrl = useBackgroundImage(
    config.background_image_path,
    "load_background_image_data_url",
    setError
  );
  const hudBackgroundUrl = useBackgroundImage(
    config.hud_background_path,
    "load_background_image_data_url",
    setError
  );

  const theme = normalizeTheme(config.theme);
  const backgroundMode = normalizeBackgroundMode(config.background_mode);
  const dirty = !configsEqual(config, savedConfig);
  const filterWordsText = useMemo(() => config.filter_words.join(", "), [config.filter_words]);
  const backgroundStyle = useMemo(
    () => buildBackgroundStyle(config, backgroundUrl),
    [
      backgroundUrl,
      config.background_color,
      config.background_gradient_end,
      config.background_gradient_start,
      config.background_mode
    ]
  );
  const shellStyle = useMemo(() => buildShellStyle(config, theme), [config, theme]);

  const { autosaving } = useAutosave(
    loading,
    capturingHotkey,
    config,
    dirty,
    setError,
    setSavedConfig,
    savedConfigRef
  );

  function update<K extends keyof Config>(key: K, value: Config[K]) {
    setConfig((current) => ({
      ...current,
      [key]: value
    }));
  }

  function updateFilterWords(value: string) {
    update("filter_words", parseFilterWords(value));
  }

  function resetAppearance() {
    setConfig((current) => ({
      ...current,
      theme: fallbackConfig.theme,
      background_mode: fallbackConfig.background_mode,
      background_color: fallbackConfig.background_color,
      background_gradient_start: fallbackConfig.background_gradient_start,
      background_gradient_end: fallbackConfig.background_gradient_end,
      panel_color: fallbackConfig.panel_color,
      section_color: fallbackConfig.section_color,
      background_image_path: fallbackConfig.background_image_path,
      meter_color_start: fallbackConfig.meter_color_start,
      meter_color_end: fallbackConfig.meter_color_end
    }));
  }

  async function openApiKeyPage() {
    setError(null);

    try {
      await invoke("open_external_url", {
        url: "https://console.groq.com/keys"
      });
    } catch (openError) {
      setError(String(openError));
    }
  }

  async function pickBackgroundImage() {
    setError(null);

    try {
      const selected = await open({
        multiple: false,
        directory: false,
        filters: [
          {
            name: "Images",
            extensions: ["png", "jpg", "jpeg", "webp", "bmp", "gif"]
          }
        ]
      });

      if (typeof selected === "string") {
        const importedPath = await invoke<string>("import_background_image", {
          path: selected
        });

        setConfig((current) => ({
          ...current,
          background_mode: "image",
          background_image_path: importedPath
        }));
      }
    } catch (dialogError) {
      setError(String(dialogError));
    }
  }

  async function pickHudBackgroundImage() {
    setError(null);

    try {
      const selected = await open({
        multiple: false,
        directory: false,
        filters: [
          {
            name: "Images and GIFs",
            extensions: ["png", "jpg", "jpeg", "webp", "bmp", "gif"]
          }
        ]
      });

      if (typeof selected === "string") {
        const importedPath = await invoke<string>("import_hud_background_image", {
          path: selected
        });

        setConfig((current) => ({
          ...current,
          hud_background_mode: "image",
          hud_background_path: importedPath
        }));
      }
    } catch (dialogError) {
      setError(String(dialogError));
    }
  }

  return (
    <div className={`app-shell theme-${theme} background-${backgroundMode}`} style={shellStyle}>
      {!loading && !config.api_key && <OnboardingOverlay />}
      <div className="background-layer" style={backgroundStyle} />
      <div className="background-scrim" />

      <main className="shell-layout">
        <section className="stage-area">
          <div className="stage-brand">
            <span className="window-label">{appInfo.appName}</span>
            <span className="version-pill">v{appInfo.version}</span>
          </div>

          <div className="stage-focus">
            {advancedMode && (
              <HudStageSection
                backgroundUrl={hudBackgroundUrl}
                config={config}
                onPickHudBackgroundImage={pickHudBackgroundImage}
                onUpdate={update}
                runtime={runtime}
              />
            )}
          </div>
        </section>

        <aside className="control-dock card">
          <div className="dock-scroll">
            {error ? <p className="error-text">{error}</p> : null}
            {loading ? <div className="loading-banner">Loading...</div> : null}

            <div className="mode-toggle-row">
              <button
                className={!advancedMode ? "mode-pill active" : "mode-pill"}
                onClick={() => setAdvancedMode(false)}
                type="button"
              >
                Normal
              </button>
              <button
                className={advancedMode ? "mode-pill active" : "mode-pill"}
                onClick={() => setAdvancedMode(true)}
                type="button"
              >
                Advanced
              </button>
            </div>

            {!advancedMode && (
              <QuickStartSection
                config={config}
                hotkeyLabel={hotkeyLabel}
                capturingHotkey={capturingHotkey}
                captureMessage={captureMessage}
                onUpdate={update}
                onStartHotkeyCapture={startHotkeyCapture}
                onStopHotkeyCapture={stopHotkeyCapture}
                onOpenApiKeyPage={openApiKeyPage}
              />
            )}

            {advancedMode && (
              <>
                <SetupSection
                  config={config}
                  micNames={micNames}
                  hotkeyLabel={hotkeyLabel}
                  capturingHotkey={capturingHotkey}
                  captureMessage={captureMessage}
                  onUpdate={update}
                  onStartHotkeyCapture={startHotkeyCapture}
                  onStopHotkeyCapture={stopHotkeyCapture}
                  onOpenApiKeyPage={openApiKeyPage}
                />

                <SettingsSection
                  config={config}
                  filterWordsText={filterWordsText}
                  onUpdate={update}
                  onUpdateFilterWords={updateFilterWords}
                />

                <AdvancedSection
                  backgroundMode={backgroundMode}
                  config={config}
                  expanded={true}
                  onPickBackgroundImage={pickBackgroundImage}
                  onResetAppearance={resetAppearance}
                  onToggle={() => {}}
                  onUpdate={update}
                  theme={theme}
                />

                <HistorySection />
              </>
            )}
          </div>
        </aside>
      </main>
    </div>
  );
}
