import type { Config } from "../types";

export function configsEqual(left: Config, right: Config): boolean {
  return JSON.stringify(left) === JSON.stringify(right);
}

export function parseFilterWords(value: string): string[] {
  return value
    .split(",")
    .map((entry) => entry.trim())
    .filter(Boolean);
}
