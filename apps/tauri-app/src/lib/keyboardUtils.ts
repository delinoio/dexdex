// Keyboard layout utilities
// Maps physical key codes (KeyboardEvent.code) to their Latin character equivalents.
// This allows shortcuts to work regardless of the active keyboard layout (e.g., Korean, Russian, etc.).

// Mapping from KeyboardEvent.code to the Latin character on a standard QWERTY layout
const CODE_TO_LATIN_KEY: Readonly<Record<string, string>> = Object.freeze({
  KeyA: "a",
  KeyB: "b",
  KeyC: "c",
  KeyD: "d",
  KeyE: "e",
  KeyF: "f",
  KeyG: "g",
  KeyH: "h",
  KeyI: "i",
  KeyJ: "j",
  KeyK: "k",
  KeyL: "l",
  KeyM: "m",
  KeyN: "n",
  KeyO: "o",
  KeyP: "p",
  KeyQ: "q",
  KeyR: "r",
  KeyS: "s",
  KeyT: "t",
  KeyU: "u",
  KeyV: "v",
  KeyW: "w",
  KeyX: "x",
  KeyY: "y",
  KeyZ: "z",
  Digit0: "0",
  Digit1: "1",
  Digit2: "2",
  Digit3: "3",
  Digit4: "4",
  Digit5: "5",
  Digit6: "6",
  Digit7: "7",
  Digit8: "8",
  Digit9: "9",
  Comma: ",",
  Period: ".",
  Slash: "/",
  Backslash: "\\",
  BracketLeft: "[",
  BracketRight: "]",
  Semicolon: ";",
  Quote: "'",
  Backquote: "`",
  Minus: "-",
  Equal: "=",
});

/**
 * Returns the Latin key character for a given KeyboardEvent.code.
 * Returns undefined if the code doesn't map to a known Latin key.
 *
 * This is used as a fallback when event.key returns a non-Latin character
 * (e.g., Korean ㅊ instead of 'c') due to the active keyboard layout.
 *
 * @example
 * getLatinKeyFromCode('KeyC') // returns 'c'
 * getLatinKeyFromCode('Digit1') // returns '1'
 * getLatinKeyFromCode('Escape') // returns undefined
 */
export function getLatinKeyFromCode(code: string): string | undefined {
  return CODE_TO_LATIN_KEY[code];
}

/**
 * Returns the effective Latin key for a keyboard event, accounting for
 * non-Latin keyboard layouts. If event.key is a non-Latin single character,
 * falls back to the physical key code mapping.
 *
 * @example
 * // Korean layout: physical 'C' key produces 'ㅊ'
 * getEffectiveKey({ key: 'ㅊ', code: 'KeyC' }) // returns 'c'
 * // English layout: physical 'C' key produces 'c'
 * getEffectiveKey({ key: 'c', code: 'KeyC' }) // returns 'c'
 */
export function getEffectiveKey(event: Pick<KeyboardEvent, "key" | "code">): string {
  const key = event.key.toLowerCase();
  const latinKey = getLatinKeyFromCode(event.code);
  return key.length === 1 && !/[a-z0-9]/.test(key) && latinKey ? latinKey : key;
}
