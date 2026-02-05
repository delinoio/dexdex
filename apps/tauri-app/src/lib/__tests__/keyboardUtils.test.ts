// keyboardUtils tests
import { describe, it, expect } from "vitest";
import { getLatinKeyFromCode } from "../keyboardUtils";

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
