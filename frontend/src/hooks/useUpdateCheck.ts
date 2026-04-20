import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { UpdateInfo } from "../types";

const STORAGE_KEY = "voice-type-update-dismissed";

export function useUpdateCheck(loading: boolean) {
  const [update, setUpdate] = useState<UpdateInfo | null>(null);
  const [dismissed, setDismissed] = useState(() => {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (!saved) return false;
    try {
      const parsed = JSON.parse(saved) as { version: string; dismissedAt: number };
      return parsed.version === saved;
    } catch {
      return false;
    }
  });

  useEffect(() => {
    if (loading) return;

    let mounted = true;

    void (async () => {
      try {
        const result = await invoke<UpdateInfo | null>("check_for_updates");
        if (!mounted || !result) return;

        const saved = localStorage.getItem(STORAGE_KEY);
        if (saved) {
          try {
            const parsed = JSON.parse(saved) as { version: string };
            if (parsed.version === result.latestVersion) return;
          } catch { /* ignore */ }
        }

        setUpdate(result);
      } catch { /* silent - network error, etc. */ }
    })();

    return () => { mounted = false; };
  }, [loading]);

  function dismissUpdate() {
    if (update) {
      localStorage.setItem(STORAGE_KEY, JSON.stringify({ version: update.latestVersion }));
    }
    setUpdate(null);
    setDismissed(true);
  }

  return { update, dismissUpdate };
}
