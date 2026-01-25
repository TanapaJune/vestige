//! Temporal-Aware Search
//!
//! Search that takes time into account:
//! - Filter by validity period
//! - Boost recent results
//! - Query historical states

use chrono::{DateTime, Duration, Utc};

// ============================================================================
// TEMPORAL SEARCH CONFIGURATION
// ============================================================================

/// Configuration for temporal search
#[derive(Debug, Clone)]
pub struct TemporalSearchConfig {
    /// Boost factor for recent memories (per day decay)
    pub recency_decay: f64,
    /// Maximum age for recency boost (days)
    pub recency_max_age_days: i64,
    /// Boost for currently valid memories
    pub validity_boost: f64,
}

impl Default for TemporalSearchConfig {
    fn default() -> Self {
        Self {
            recency_decay: 0.95, // 5% decay per day
            recency_max_age_days: 30,
            validity_boost: 1.5,
        }
    }
}

// ============================================================================
// TEMPORAL SEARCHER
// ============================================================================

/// Temporal-aware search enhancer
pub struct TemporalSearcher {
    config: TemporalSearchConfig,
}

impl Default for TemporalSearcher {
    fn default() -> Self {
        Self::new()
    }
}

impl TemporalSearcher {
    /// Create a new temporal searcher
    pub fn new() -> Self {
        Self {
            config: TemporalSearchConfig::default(),
        }
    }

    /// Create with custom config
    pub fn with_config(config: TemporalSearchConfig) -> Self {
        Self { config }
    }

    /// Calculate recency boost for a timestamp
    ///
    /// Returns a multiplier between 0.0 and 1.0
    /// Recent items get higher values
    pub fn recency_boost(&self, timestamp: DateTime<Utc>) -> f64 {
        let now = Utc::now();
        let age_days = (now - timestamp).num_days();

        if age_days < 0 {
            // Future timestamp, no boost
            return 1.0;
        }

        if age_days > self.config.recency_max_age_days {
            // Beyond max age, minimum boost
            return self
                .config
                .recency_decay
                .powi(self.config.recency_max_age_days as i32);
        }

        self.config.recency_decay.powi(age_days as i32)
    }

    /// Calculate validity boost
    ///
    /// Returns validity_boost if the memory is currently valid
    /// Returns 1.0 if validity is uncertain
    /// Returns 0.0 if definitely invalid
    pub fn validity_boost(
        &self,
        valid_from: Option<DateTime<Utc>>,
        valid_until: Option<DateTime<Utc>>,
        at_time: Option<DateTime<Utc>>,
    ) -> f64 {
        let check_time = at_time.unwrap_or_else(Utc::now);

        let is_valid = match (valid_from, valid_until) {
            (None, None) => true, // Always valid
            (Some(from), None) => check_time >= from,
            (None, Some(until)) => check_time <= until,
            (Some(from), Some(until)) => check_time >= from && check_time <= until,
        };

        if is_valid {
            self.config.validity_boost
        } else {
            0.0 // Exclude invalid results
        }
    }

    /// Apply temporal scoring to search results
    ///
    /// Combines base score with recency and validity boosts
    pub fn apply_temporal_scoring(
        &self,
        base_score: f64,
        created_at: DateTime<Utc>,
        valid_from: Option<DateTime<Utc>>,
        valid_until: Option<DateTime<Utc>>,
        at_time: Option<DateTime<Utc>>,
    ) -> f64 {
        let recency = self.recency_boost(created_at);
        let validity = self.validity_boost(valid_from, valid_until, at_time);

        // If invalid, score is 0
        if validity == 0.0 {
            return 0.0;
        }

        base_score * recency * validity
    }

    /// Generate time-based query filters
    pub fn time_filter(&self, range: TemporalRange) -> TemporalFilter {
        TemporalFilter {
            start: range.start,
            end: range.end,
            require_valid: true,
        }
    }

    /// Calculate how many days until a memory expires
    pub fn days_until_expiry(&self, valid_until: Option<DateTime<Utc>>) -> Option<i64> {
        valid_until.map(|until| {
            let now = Utc::now();
            (until - now).num_days()
        })
    }

    /// Check if a memory is about to expire (within N days)
    pub fn is_expiring_soon(&self, valid_until: Option<DateTime<Utc>>, days: i64) -> bool {
        match self.days_until_expiry(valid_until) {
            Some(remaining) => remaining >= 0 && remaining <= days,
            None => false,
        }
    }
}

// ============================================================================
// TEMPORAL RANGE
// ============================================================================

/// A time range for filtering
#[derive(Debug, Clone)]
pub struct TemporalRange {
    /// Start of range (inclusive)
    pub start: Option<DateTime<Utc>>,
    /// End of range (inclusive)
    pub end: Option<DateTime<Utc>>,
}

impl TemporalRange {
    /// Create an unbounded range
    pub fn all() -> Self {
        Self {
            start: None,
            end: None,
        }
    }

    /// Create a range from a start time
    pub fn from(start: DateTime<Utc>) -> Self {
        Self {
            start: Some(start),
            end: None,
        }
    }

    /// Create a range until an end time
    pub fn until(end: DateTime<Utc>) -> Self {
        Self {
            start: None,
            end: Some(end),
        }
    }

    /// Create a bounded range
    pub fn between(start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        Self {
            start: Some(start),
            end: Some(end),
        }
    }

    /// Last N days
    pub fn last_days(days: i64) -> Self {
        let now = Utc::now();
        Self {
            start: Some(now - Duration::days(days)),
            end: Some(now),
        }
    }

    /// Last week
    pub fn last_week() -> Self {
        Self::last_days(7)
    }

    /// Last month
    pub fn last_month() -> Self {
        Self::last_days(30)
    }
}

/// Filter for temporal queries
#[derive(Debug, Clone)]
pub struct TemporalFilter {
    /// Start of range
    pub start: Option<DateTime<Utc>>,
    /// End of range
    pub end: Option<DateTime<Utc>>,
    /// Require memories to be valid within range
    pub require_valid: bool,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recency_boost() {
        let searcher = TemporalSearcher::new();
        let now = Utc::now();

        // Today = full boost
        let today_boost = searcher.recency_boost(now);
        assert!((today_boost - 1.0).abs() < 0.01);

        // Yesterday = slightly less
        let yesterday_boost = searcher.recency_boost(now - Duration::days(1));
        assert!(yesterday_boost < today_boost);
        assert!(yesterday_boost > 0.9);

        // Week ago = more decay
        let week_ago_boost = searcher.recency_boost(now - Duration::days(7));
        assert!(week_ago_boost < yesterday_boost);
    }

    #[test]
    fn test_validity_boost() {
        let searcher = TemporalSearcher::new();
        let now = Utc::now();
        let yesterday = now - Duration::days(1);
        let tomorrow = now + Duration::days(1);

        // Currently valid
        let valid_boost = searcher.validity_boost(Some(yesterday), Some(tomorrow), None);
        assert!(valid_boost > 1.0);

        // Expired
        let expired_boost = searcher.validity_boost(None, Some(yesterday), None);
        assert_eq!(expired_boost, 0.0);

        // Not yet valid
        let future_boost = searcher.validity_boost(Some(tomorrow), None, None);
        assert_eq!(future_boost, 0.0);
    }

    #[test]
    fn test_temporal_scoring() {
        let searcher = TemporalSearcher::new();
        let now = Utc::now();
        let yesterday = now - Duration::days(1);

        // Valid and recent
        let score = searcher.apply_temporal_scoring(1.0, now, None, None, None);
        assert!(score > 1.0); // Should have validity boost

        // Valid but old
        let old_score =
            searcher.apply_temporal_scoring(1.0, now - Duration::days(10), None, None, None);
        assert!(old_score < score);

        // Invalid (expired)
        let invalid_score = searcher.apply_temporal_scoring(1.0, now, None, Some(yesterday), None);
        assert_eq!(invalid_score, 0.0);
    }

    #[test]
    fn test_is_expiring_soon() {
        let searcher = TemporalSearcher::new();
        let now = Utc::now();

        // Expires tomorrow
        assert!(searcher.is_expiring_soon(Some(now + Duration::days(1)), 7));

        // Expires next month
        assert!(!searcher.is_expiring_soon(Some(now + Duration::days(30)), 7));

        // Already expired
        assert!(!searcher.is_expiring_soon(Some(now - Duration::days(1)), 7));

        // No expiry
        assert!(!searcher.is_expiring_soon(None, 7));
    }

    #[test]
    fn test_temporal_range() {
        let last_week = TemporalRange::last_week();
        assert!(last_week.start.is_some());
        assert!(last_week.end.is_some());

        let all = TemporalRange::all();
        assert!(all.start.is_none());
        assert!(all.end.is_none());
    }
}
