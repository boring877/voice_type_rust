import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getBrowserKeyboardCode, getBrowserMouseButton } from "../lib/hotkeys";
import type { Config, HotkeyCapturePayload } from "../types";

export function useHotkeyCapture(
  setError: (e: string | null) => void,
  setConfig: (updater: Config | ((prev: Config) => Config)) => void,
  setHotkeyLabel: (label: string) => void
) {
  const [capturingHotkey, setCapturingHotkey] = useState(false);
  const [captureMessage, setCaptureMessage] = useState(
    "Press the key or button you want to use."
  );

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

  return { capturingHotkey, captureMessage, applyCapturedHotkey, startHotkeyCapture, stopHotkeyCapture };
}
