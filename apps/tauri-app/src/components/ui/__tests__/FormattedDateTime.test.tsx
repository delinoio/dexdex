import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { FormattedDateTime, formatDateTime } from "../FormattedDateTime";

describe("formatDateTime", () => {
  it("formats date string with time by default", () => {
    const result = formatDateTime("2024-01-15T10:30:00Z");
    // Should contain date parts
    expect(result).toMatch(/Jan/);
    expect(result).toMatch(/15/);
    expect(result).toMatch(/2024/);
    // Should contain time (hour and minute) - exact format varies by locale
    expect(result).toMatch(/\d{1,2}:\d{2}/);
  });

  it("formats Date object with time by default", () => {
    const date = new Date("2024-01-15T10:30:00Z");
    const result = formatDateTime(date);
    // Should contain date parts
    expect(result).toMatch(/Jan/);
    expect(result).toMatch(/15/);
    expect(result).toMatch(/2024/);
    // Should contain time
    expect(result).toMatch(/\d{1,2}:\d{2}/);
  });

  it("formats without time when includeTime is false", () => {
    const result = formatDateTime("2024-01-15T10:30:00Z", false);
    // Should contain date parts
    expect(result).toMatch(/Jan/);
    expect(result).toMatch(/15/);
    expect(result).toMatch(/2024/);
    // Should NOT contain the time pattern with colon separator
    // The format without time should just show "Jan 15, 2024" or similar
    const timePattern = /\d{1,2}:\d{2}/;
    expect(result).not.toMatch(timePattern);
  });
});

describe("FormattedDateTime", () => {
  it("renders formatted date from string", () => {
    render(<FormattedDateTime date="2024-01-15T10:30:00Z" />);
    const timeElement = screen.getByRole("time");
    expect(timeElement).toBeInTheDocument();
    expect(timeElement.textContent).toMatch(/Jan/);
    expect(timeElement.textContent).toMatch(/15/);
    expect(timeElement.textContent).toMatch(/2024/);
    expect(timeElement.textContent).toMatch(/\d{1,2}:\d{2}/);
  });

  it("renders formatted date from Date object", () => {
    const date = new Date("2024-01-15T10:30:00Z");
    render(<FormattedDateTime date={date} />);
    const timeElement = screen.getByRole("time");
    expect(timeElement).toBeInTheDocument();
    expect(timeElement.textContent).toMatch(/Jan/);
  });

  it("sets datetime attribute to ISO string", () => {
    render(<FormattedDateTime date="2024-01-15T10:30:00Z" />);
    const timeElement = screen.getByRole("time");
    expect(timeElement).toHaveAttribute("datetime", "2024-01-15T10:30:00.000Z");
  });

  it("renders without time when includeTime is false", () => {
    render(<FormattedDateTime date="2024-01-15T10:30:00Z" includeTime={false} />);
    const timeElement = screen.getByRole("time");
    expect(timeElement).toBeInTheDocument();
    expect(timeElement.textContent).toMatch(/Jan/);
    expect(timeElement.textContent).toMatch(/15/);
    expect(timeElement.textContent).toMatch(/2024/);
    expect(timeElement.textContent).not.toMatch(/\d{1,2}:\d{2}/);
  });

  it("accepts additional className", () => {
    render(<FormattedDateTime date="2024-01-15T10:30:00Z" className="custom-class" />);
    const timeElement = screen.getByRole("time");
    expect(timeElement).toHaveClass("custom-class");
  });

  it("forwards HTML attributes", () => {
    render(<FormattedDateTime date="2024-01-15T10:30:00Z" data-testid="test-datetime" />);
    expect(screen.getByTestId("test-datetime")).toBeInTheDocument();
  });
});
