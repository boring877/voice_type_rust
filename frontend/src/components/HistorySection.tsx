import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { SectionHeader } from "./controls";
import type { HistoryEntry } from "../types";

function timeAgo(timestamp: number): string {
  const seconds = Math.floor(Date.now() - timestamp) / 1000;

  if (seconds < 60) {
    return "just now";
  }

  const minutes = Math.floor(seconds / 60);

  if (minutes < 60) {
    return `${minutes}m ago`;
  }

  const hours = Math.floor(minutes / 60);

  if (hours < 24) {
    return `${hours}h ago`;
  }

  const days = Math.floor(hours / 24);
  return `${days}d ago`;
}

function truncate(text: string, max: number): string {
  if (text.length <= max) {
    return text;
  }
  return text.slice(0, max) + "...";
}

export function HistorySection() {
  const [entries, setEntries] = useState<HistoryEntry[]>([]);
  const [copiedIndex, setCopiedIndex] = useState<number | null>(null);

  useEffect(() => {
    invoke<HistoryEntry[]>("get_history")
      .then(setEntries)
      .catch(() => {});
  }, []);

  async function handleCopy(text: string, index: number) {
    try {
      await navigator.clipboard.writeText(text);
      setCopiedIndex(index);
      setTimeout(() => setCopiedIndex(null), 1500);
    } catch {
      // ignore
    }
  }

  async function handleClear() {
    try {
      await invoke("clear_history");
      setEntries([]);
    } catch {
      // ignore
    }
  }

  return (
    <section className="dock-section">
      <SectionHeader title="History" />

      {entries.length === 0 ? (
        <p className="hint-text">No transcriptions yet.</p>
      ) : (
        <ul className="history-list">
          {entries.map((entry, index) => (
            <li key={entry.timestamp} className="history-item">
              <button
                className="history-item-button"
                onClick={() => void handleCopy(entry.text, index)}
                type="button"
                title="Click to copy full text"
              >
                <span className="history-item-meta">
                  {timeAgo(entry.timestamp)}
                  {" · "}
                  {entry.wordCount}w
                </span>
                <span className="history-item-text">
                  {copiedIndex === index ? "Copied!" : truncate(entry.text, 60)}
                </span>
              </button>
            </li>
          ))}
        </ul>
      )}

      {entries.length > 0 ? (
        <div className="history-footer">
          <button className="ghost-button" onClick={() => void handleClear()} type="button">
            Clear history
          </button>
        </div>
      ) : null}
    </section>
  );
}
