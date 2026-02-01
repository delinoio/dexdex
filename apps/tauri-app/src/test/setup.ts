// Test setup for Vitest
import "@testing-library/jest-dom";

// Mock the Tauri API
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));
