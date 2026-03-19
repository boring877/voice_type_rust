import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { RuntimeSnapshot, RuntimeState } from "../types";

export function useRuntimePolling(
  loading: boolean,
  appState: RuntimeState,
  applyRuntimeSnapshot: (snapshot: RuntimeSnapshot) => void
) {
  useEffect(() => {
    if (loading) {
      return;
    }

    const refreshSnapshot = async () => {
      try {
        const snapshot = await invoke<RuntimeSnapshot>("get_runtime_snapshot");
        applyRuntimeSnapshot(snapshot);
      } catch {
      }
    };

    const handleVisibilityRefresh = () => {
      if (!document.hidden) {
        void refreshSnapshot();
      }
    };

    const intervalMs = appState === "recording" ? 140 : 1200;
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
  }, [loading, appState]);
}
