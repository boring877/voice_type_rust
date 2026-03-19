import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export function useBackgroundImage(
  path: string,
  command: string,
  setError: (e: string | null) => void
): string | null {
  const [dataUrl, setDataUrl] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    const trimmed = path.trim();

    if (!trimmed) {
      setDataUrl(null);
      return;
    }

    void invoke<string>(command, { path: trimmed })
      .then((url) => {
        if (!cancelled) {
          setDataUrl(url);
        }
      })
      .catch((loadError) => {
        if (!cancelled) {
          setDataUrl(null);
          setError(String(loadError));
        }
      });

    return () => {
      cancelled = true;
    };
  }, [path, command]);

  return dataUrl;
}
