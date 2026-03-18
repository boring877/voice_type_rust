import { useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import {
  buildBackgroundStyle,
  buildShellStyle,
  normalizeBackgroundMode,
  normalizeTheme
} from "./lib/appearance";
import { configsEqual, parseFilterWords } from "./lib/config";
import { getBrowserKeyboardCode, getBrowserMouseButton } from "./lib/hotkeys";
import { fallbackConfig, fallbackInfo, fallbackRuntime } from "./lib/options";
import {
  AdvancedSection,
  SettingsSection,
  SetupSection
} from "./components/sections";
import { HudStageSection } from "./components/HudStageSection";
import type { AppInfo, Config, HotkeyCapturePayload, RuntimeSnapshot } from "./types";

export default function App() {
  const [config, setConfig] = useState<Config>(fallbackConfig);
  const [savedConfig, setSavedConfig] = useState<Config>(fallbackConfig);
  const [runtime, setRuntime] = useState<RuntimeSnapshot>(fallbackRuntime);
  const [appInfo, setAppInfo] = useState<AppInfo>(fallbackInfo);
  const [micNames, setMicNames] = useState<string[]>([]);
  const [hotkeyLabel, setHotkeyLabel] = useState(fallbackRuntime.hotkeyLabel);
  const [backgroundUrl, setBackgroundUrl] = useState<string | null>(null);
  const [hudBackgroundUrl, setHudBackgroundUrl] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [autosaving, setAutosaving] = useState(false);
  const [capturingHotkey, setCapturingHotkey] = useState(false);
  const [captureMessage, setCaptureMessage] = useState(
    "Press the key or button you want to use."
  );
  const [error, setError] = useState<string | null>(null);
  const [advancedMode, setAdvancedMode] = useState(false);

  const configRef = useRef(config);
  const savedConfigRef = useRef(savedConfig);

  useEffect(() => {
    configRef.current = config;
  }, [config]);

  useEffect(() => {
    savedConfigRef.current = savedConfig;
  }, [savedConfig]);

  useEffect(() => {
    let mounted = true;
    const unlisteners: Array<() => void | Promise<void>> = [];

    const boot = async () => {
      try {
        const runtimeUnlisten = await listen<RuntimeSnapshot>(
          "voice-type://runtime",
          (event) => {
            if (!mounted) {
              return;
            }
            applyRuntimeSnapshot(event.payload);
          }
        );

        const captureUnlisten = await listen<HotkeyCapturePayload>(
          "voice-type://hotkey-captured",
          (event) => {
            if (!mounted) {
              return;
            }

            applyCapturedHotkey(event.payload);
          }
        );

        unlisteners.push(runtimeUnlisten, captureUnlisten);

        const [snapshot, info, microphones] = await Promise.all([
          invoke<RuntimeSnapshot>("get_runtime_snapshot"),
          invoke<AppInfo>("get_app_info"),
          invoke<string[]>("list_microphones")
        ]);

        if (!mounted) {
          return;
        }

        applyRuntimeSnapshot(snapshot);
        setAppInfo(info);
        setMicNames(microphones);
        setError(null);
      } catch (bootError) {
        if (!mounted) {
          return;
        }

        setError(String(bootError));
      } finally {
        if (mounted) {
          setLoading(false);
        }
      }
    };

    void boot();

    return () => {
      mounted = false;
      for (const unlisten of unlisteners) {
        void unlisten();
      }
    };
  }, []);

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

  useEffect(() => {
    let cancelled = false;
    const path = config.background_image_path.trim();

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
      .catch((loadError) => {
        if (!cancelled) {
          setBackgroundUrl(null);
          setError(String(loadError));
        }
      });

    return () => {
      cancelled = true;
    };
  }, [config.background_image_path]);

  useEffect(() => {
    let cancelled = false;
    const path = config.hud_background_path.trim();

    if (!path) {
      setHudBackgroundUrl(null);
      return;
    }

    void invoke<string>("load_background_image_data_url", { path })
      .then((dataUrl) => {
        if (!cancelled) {
          setHudBackgroundUrl(dataUrl);
        }
      })
      .catch((loadError) => {
        if (!cancelled) {
          setHudBackgroundUrl(null);
          setError(String(loadError));
        }
      });

    return () => {
      cancelled = true;
    };
  }, [config.hud_background_path]);

  function applyCapturedHotkey(payload: HotkeyCapturePayload) {
    setConfig((current) => ({
      ...current,
      hotkey: payload.configValue
    }));
    setHotkeyLabel(payload.label);
    setCapturingHotkey(false);
    setCaptureMessage(`Captured ${payload.label}. It will save automatically.`);
  }

  useEffect(() => {
    if (!capturingHotkey) {
      return;
    }

    const handleKeyDown = (event: KeyboardEvent) => {
      const code = getBrowserKeyboardCode(event);
      if (!code) {
        return;
      }

      event.preventDefault();
      event.stopPropagation();

      void invoke<HotkeyCapturePayload | null>("normalize_keyboard_capture", { code })
        .then((payload) => {
          if (!payload) {
            return;
          }

          applyCapturedHotkey(payload);
          void invoke("cancel_hotkey_capture");
        })
        .catch((captureError) => {
          setCapturingHotkey(false);
          setError(String(captureError));
        });
    };

    const handleMouseDown = (event: MouseEvent) => {
      const button = getBrowserMouseButton(event);
      if (button === null) {
        return;
      }

      event.preventDefault();
      event.stopPropagation();

      void invoke<HotkeyCapturePayload | null>("normalize_mouse_capture", { button })
        .then((payload) => {
          if (!payload) {
            return;
          }

          applyCapturedHotkey(payload);
          void invoke("cancel_hotkey_capture");
        })
        .catch((captureError) => {
          setCapturingHotkey(false);
          setError(String(captureError));
        });
    };

    const suppressContextMenu = (event: MouseEvent) => {
      event.preventDefault();
    };

    window.addEventListener("keydown", handleKeyDown, true);
    window.addEventListener("mousedown", handleMouseDown, true);
    window.addEventListener("auxclick", handleMouseDown, true);
    window.addEventListener("contextmenu", suppressContextMenu, true);

    return () => {
      window.removeEventListener("keydown", handleKeyDown, true);
      window.removeEventListener("mousedown", handleMouseDown, true);
      window.removeEventListener("auxclick", handleMouseDown, true);
      window.removeEventListener("contextmenu", suppressContextMenu, true);
    };
  }, [capturingHotkey]);

  useEffect(() => {
    if (loading) {
      return;
    }

    const refreshSnapshot = async () => {
      try {
        const snapshot = await invoke<RuntimeSnapshot>("get_runtime_snapshot");
        applyRuntimeSnapshot(snapshot);
      } catch {
        // Keep the last known state if the polling fallback misses once.
      }
    };

    const handleVisibilityRefresh = () => {
      if (!document.hidden) {
        void refreshSnapshot();
      }
    };

    const intervalMs = runtime.appState === "recording" ? 140 : 1200;
    const intervalId = window.setInterval(() => {
      void refreshSnapshot();
    }, intervalMs);

    window.addEventListener("focus", handleVisibilityRefresh);
    document.addEventListener("visibilitychange", handleVisibilityRefresh);

    return () => {
      window.clearInterval(intervalId);
      window.removeEventListener("focus", handleVisibilityRefresh);
      document.removeEventListener("visibilitychange", handleVisibilityRefresh);
    };
  }, [loading, runtime.appState]);

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

  async function persistConfig(nextConfig: Config) {
    setAutosaving(true);
    setError(null);

    try {
      await invoke("save_config", { config: nextConfig });
      savedConfigRef.current = nextConfig;
      setSavedConfig(nextConfig);
    } catch (saveError) {
      setError(String(saveError));
    } finally {
      setAutosaving(false);
    }
  }

  useEffect(() => {
    if (loading || capturingHotkey || autosaving || !dirty) {
      return;
    }

    const timeoutId = window.setTimeout(() => {
      void persistConfig(config);
    }, 900);

    return () => window.clearTimeout(timeoutId);
  }, [autosaving, capturingHotkey, config, dirty, loading]);

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

  async function startHotkeyCapture() {
    setError(null);
    setCapturingHotkey(true);
    setCaptureMessage("Press the key or mouse button you want.");

    try {
      await invoke("begin_hotkey_capture");
    } catch (captureError) {
      setCapturingHotkey(false);
      setError(String(captureError));
    }
  }

  async function stopHotkeyCapture() {
    setCapturingHotkey(false);
    setCaptureMessage("Capture cancelled.");

    try {
      await invoke("cancel_hotkey_capture");
    } catch (captureError) {
      setError(String(captureError));
    }
  }

  return (
    <div className={`app-shell theme-${theme} background-${backgroundMode}`} style={shellStyle}>
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

            {advancedMode && (
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
            )}
          </div>
        </aside>
      </main>
    </div>
  );
}
