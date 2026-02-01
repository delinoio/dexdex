// Test setup for Vitest
import { vi } from "vitest";
import "@testing-library/jest-dom";

// Mock the Tauri API
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));
