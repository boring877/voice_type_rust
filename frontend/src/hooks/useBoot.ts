import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { fallbackInfo } from "../lib/options";
import type { AppInfo, HotkeyCapturePayload, RuntimeSnapshot } from "../types";

export function useBoot(
  setError: (e: string | null) => void,
  applyRuntimeSnapshot: (snapshot: RuntimeSnapshot) => void,
  applyCapturedHotkey: (payload: HotkeyCapturePayload) => void
) {
  const [loading, setLoading] = useState(true);
  const [appInfo, setAppInfo] = useState<AppInfo>(fallbackInfo);
  const [micNames, setMicNames] = useState<string[]>([]);

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

  return { loading, appInfo, micNames };
}
