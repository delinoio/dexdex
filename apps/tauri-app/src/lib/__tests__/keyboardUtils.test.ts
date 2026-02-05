// keyboardUtils tests
import { describe, it, expect } from "vitest";
import { getLatinKeyFromCode, getEffectiveKey } from "../keyboardUtils";

describe("getLatinKeyFromCode", () => {
  it("should map letter key codes to lowercase Latin characters", () => {
    expect(getLatinKeyFromCode("KeyA")).toBe("a");
    expect(getLatinKeyFromCode("KeyZ")).toBe("z");
    expect(getLatinKeyFromCode("KeyK")).toBe("k");
    expect(getLatinKeyFromCode("KeyC")).toBe("c");
  });

  it("should map digit key codes to digit characters", () => {
    expect(getLatinKeyFromCode("Digit0")).toBe("0");
    expect(getLatinKeyFromCode("Digit1")).toBe("1");
    expect(getLatinKeyFromCode("Digit9")).toBe("9");
  });

  it("should map punctuation key codes", () => {
    expect(getLatinKeyFromCode("Comma")).toBe(",");
    expect(getLatinKeyFromCode("Period")).toBe(".");
    expect(getLatinKeyFromCode("Slash")).toBe("/");
    expect(getLatinKeyFromCode("Semicolon")).toBe(";");
    expect(getLatinKeyFromCode("Minus")).toBe("-");
    expect(getLatinKeyFromCode("Equal")).toBe("=");
  });

  it("should return undefined for unknown codes", () => {
    expect(getLatinKeyFromCode("Escape")).toBeUndefined();
    expect(getLatinKeyFromCode("Tab")).toBeUndefined();
    expect(getLatinKeyFromCode("Enter")).toBeUndefined();
    expect(getLatinKeyFromCode("Space")).toBeUndefined();
    expect(getLatinKeyFromCode("UnknownCode")).toBeUndefined();
  });
});

describe("getEffectiveKey", () => {
  it("should return event.key for Latin characters", () => {
    expect(getEffectiveKey({ key: "c", code: "KeyC" })).toBe("c");
    expect(getEffectiveKey({ key: "k", code: "KeyK" })).toBe("k");
    expect(getEffectiveKey({ key: "1", code: "Digit1" })).toBe("1");
  });

  it("should return event.key lowercased for uppercase Latin characters", () => {
    expect(getEffectiveKey({ key: "C", code: "KeyC" })).toBe("c");
    expect(getEffectiveKey({ key: "K", code: "KeyK" })).toBe("k");
  });

  it("should fall back to physical key code for Korean layout", () => {
    expect(getEffectiveKey({ key: "ㅊ", code: "KeyC" })).toBe("c");
    expect(getEffectiveKey({ key: "ㅏ", code: "KeyK" })).toBe("k");
    expect(getEffectiveKey({ key: "ㅜ", code: "KeyN" })).toBe("n");
  });

  it("should fall back to physical key code for Russian layout", () => {
    expect(getEffectiveKey({ key: "с", code: "KeyC" })).toBe("c");
    expect(getEffectiveKey({ key: "т", code: "KeyN" })).toBe("n");
    expect(getEffectiveKey({ key: "л", code: "KeyK" })).toBe("k");
  });

  it("should return event.key for special keys (Enter, Escape, etc.)", () => {
    expect(getEffectiveKey({ key: "Enter", code: "Enter" })).toBe("enter");
    expect(getEffectiveKey({ key: "Escape", code: "Escape" })).toBe("escape");
    expect(getEffectiveKey({ key: "Tab", code: "Tab" })).toBe("tab");
  });

  it("should return event.key when code is unknown and key is non-Latin", () => {
    expect(getEffectiveKey({ key: "ㅊ", code: "UnknownKey" })).toBe("ㅊ");
  });

  it("should handle shifted keys on non-Latin layouts", () => {
    // Shift+KeyC on Russian layout might produce uppercase Cyrillic
    // event.code is still KeyC, so getEffectiveKey should return 'c'
    expect(getEffectiveKey({ key: "С", code: "KeyC" })).toBe("c");
  });
});
