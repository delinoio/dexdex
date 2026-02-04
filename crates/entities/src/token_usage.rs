//! Token usage tracking for AI coding agent sessions.
//!
//! This module provides structures for tracking token consumption,
//! costs, and duration metrics from AI coding agents like Claude Code.

use serde::{Deserialize, Serialize};

/// Token usage statistics from an AI coding agent session.
///
/// This structure captures detailed metrics about token consumption
/// and costs from agents like Claude Code that report usage in their
/// stream-json output format.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TokenUsage {
    /// Number of input tokens (excluding cache).
    pub input_tokens: u64,
    /// Number of output tokens generated.
    pub output_tokens: u64,
    /// Number of tokens read from cache.
    pub cache_read_input_tokens: u64,
    /// Number of tokens written to cache.
    pub cache_creation_input_tokens: u64,
    /// Total cost in USD for this session.
    pub total_cost_usd: f64,
    /// Duration of the session in milliseconds.
    pub duration_ms: u64,
    /// Number of conversation turns (API round-trips).
    pub num_turns: u32,
}

impl TokenUsage {
    /// Creates a new empty TokenUsage instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the total number of input tokens (including cache reads).
    pub fn total_input_tokens(&self) -> u64 {
        self.input_tokens + self.cache_read_input_tokens + self.cache_creation_input_tokens
    }

    /// Returns the total number of tokens (input + output).
    pub fn total_tokens(&self) -> u64 {
        self.total_input_tokens() + self.output_tokens
    }

    /// Adds another TokenUsage to this one (for aggregation).
    pub fn add(&mut self, other: &TokenUsage) {
        self.input_tokens += other.input_tokens;
        self.output_tokens += other.output_tokens;
        self.cache_read_input_tokens += other.cache_read_input_tokens;
        self.cache_creation_input_tokens += other.cache_creation_input_tokens;
        self.total_cost_usd += other.total_cost_usd;
        self.duration_ms += other.duration_ms;
        self.num_turns += other.num_turns;
    }

    /// Creates a new TokenUsage that is the sum of this and another.
    pub fn merged(&self, other: &TokenUsage) -> TokenUsage {
        let mut result = self.clone();
        result.add(other);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_usage_default() {
        let usage = TokenUsage::default();
        assert_eq!(usage.input_tokens, 0);
        assert_eq!(usage.output_tokens, 0);
        assert_eq!(usage.cache_read_input_tokens, 0);
        assert_eq!(usage.cache_creation_input_tokens, 0);
        assert_eq!(usage.total_cost_usd, 0.0);
        assert_eq!(usage.duration_ms, 0);
        assert_eq!(usage.num_turns, 0);
    }

    #[test]
    fn test_total_input_tokens() {
        let usage = TokenUsage {
            input_tokens: 100,
            output_tokens: 50,
            cache_read_input_tokens: 200,
            cache_creation_input_tokens: 150,
            total_cost_usd: 0.05,
            duration_ms: 5000,
            num_turns: 2,
        };
        assert_eq!(usage.total_input_tokens(), 450); // 100 + 200 + 150
    }

    #[test]
    fn test_total_tokens() {
        let usage = TokenUsage {
            input_tokens: 100,
            output_tokens: 50,
            cache_read_input_tokens: 200,
            cache_creation_input_tokens: 150,
            total_cost_usd: 0.05,
            duration_ms: 5000,
            num_turns: 2,
        };
        assert_eq!(usage.total_tokens(), 500); // 450 + 50
    }

    #[test]
    fn test_add() {
        let mut usage1 = TokenUsage {
            input_tokens: 100,
            output_tokens: 50,
            cache_read_input_tokens: 200,
            cache_creation_input_tokens: 150,
            total_cost_usd: 0.05,
            duration_ms: 5000,
            num_turns: 2,
        };
        let usage2 = TokenUsage {
            input_tokens: 50,
            output_tokens: 25,
            cache_read_input_tokens: 100,
            cache_creation_input_tokens: 75,
            total_cost_usd: 0.03,
            duration_ms: 3000,
            num_turns: 1,
        };
        usage1.add(&usage2);

        assert_eq!(usage1.input_tokens, 150);
        assert_eq!(usage1.output_tokens, 75);
        assert_eq!(usage1.cache_read_input_tokens, 300);
        assert_eq!(usage1.cache_creation_input_tokens, 225);
        assert_eq!(usage1.total_cost_usd, 0.08);
        assert_eq!(usage1.duration_ms, 8000);
        assert_eq!(usage1.num_turns, 3);
    }

    #[test]
    fn test_merged() {
        let usage1 = TokenUsage {
            input_tokens: 100,
            output_tokens: 50,
            cache_read_input_tokens: 0,
            cache_creation_input_tokens: 0,
            total_cost_usd: 0.05,
            duration_ms: 5000,
            num_turns: 2,
        };
        let usage2 = TokenUsage {
            input_tokens: 50,
            output_tokens: 25,
            cache_read_input_tokens: 0,
            cache_creation_input_tokens: 0,
            total_cost_usd: 0.03,
            duration_ms: 3000,
            num_turns: 1,
        };
        let merged = usage1.merged(&usage2);

        // Original should be unchanged
        assert_eq!(usage1.input_tokens, 100);
        // Merged should have combined values
        assert_eq!(merged.input_tokens, 150);
        assert_eq!(merged.output_tokens, 75);
    }

    #[test]
    fn test_serialization() {
        let usage = TokenUsage {
            input_tokens: 100,
            output_tokens: 50,
            cache_read_input_tokens: 200,
            cache_creation_input_tokens: 150,
            total_cost_usd: 0.05,
            duration_ms: 5000,
            num_turns: 2,
        };

        let json = serde_json::to_string(&usage).unwrap();
        let deserialized: TokenUsage = serde_json::from_str(&json).unwrap();

        assert_eq!(usage, deserialized);
    }
}
