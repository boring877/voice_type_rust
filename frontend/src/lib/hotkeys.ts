export function getBrowserKeyboardCode(event: KeyboardEvent): string | null {
  if (event.repeat) {
    return null;
  }

  const code = event.code.trim();
  return code.length > 0 ? code : null;
}

export function getBrowserMouseButton(event: MouseEvent): number | null {
  return Number.isInteger(event.button) && event.button >= 0 ? event.button : null;
}
