//! Temporal Memory - Bi-temporal knowledge modeling
//!
//! Implements a bi-temporal model for time-sensitive knowledge:
//!
//! - **Transaction Time**: When the fact was recorded (created_at, updated_at)
//! - **Valid Time**: When the fact is/was actually true (valid_from, valid_until)
//!
//! This allows querying:
//! - "What did I know on date X?" (transaction time)
//! - "What was true on date X?" (valid time)
//! - "What did I believe was true on date X, as of date Y?" (bitemporal)

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

// ============================================================================
// TEMPORAL RANGE
// ============================================================================

/// A time range with optional start and end
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemporalRange {
    /// Start of the range (inclusive)
    pub start: Option<DateTime<Utc>>,
    /// End of the range (inclusive)
    pub end: Option<DateTime<Utc>>,
}

impl TemporalRange {
    /// Create a range with both bounds
    pub fn between(start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        Self {
            start: Some(start),
            end: Some(end),
        }
    }

    /// Create a range starting from a point
    pub fn from(start: DateTime<Utc>) -> Self {
        Self {
            start: Some(start),
            end: None,
        }
    }

    /// Create a range ending at a point
    pub fn until(end: DateTime<Utc>) -> Self {
        Self {
            start: None,
            end: Some(end),
        }
    }

    /// Create an unbounded range (all time)
    pub fn all() -> Self {
        Self {
            start: None,
            end: None,
        }
    }

    /// Check if a timestamp falls within this range
    pub fn contains(&self, time: DateTime<Utc>) -> bool {
        let after_start = self.start.map(|s| time >= s).unwrap_or(true);
        let before_end = self.end.map(|e| time <= e).unwrap_or(true);
        after_start && before_end
    }

    /// Check if this range overlaps with another
    pub fn overlaps(&self, other: &TemporalRange) -> bool {
        // Two ranges overlap unless one ends before the other starts
        let this_ends_before = match (self.end, other.start) {
            (Some(e), Some(s)) => e < s,
            _ => false,
        };
        let other_ends_before = match (other.end, self.start) {
            (Some(e), Some(s)) => e < s,
            _ => false,
        };
        !this_ends_before && !other_ends_before
    }

    /// Get the duration of the range (if bounded)
    pub fn duration(&self) -> Option<Duration> {
        match (self.start, self.end) {
            (Some(s), Some(e)) => Some(e - s),
            _ => None,
        }
    }
}

impl Default for TemporalRange {
    fn default() -> Self {
        Self::all()
    }
}

// ============================================================================
// TEMPORAL VALIDITY
// ============================================================================

/// Temporal validity state for a knowledge node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TemporalValidity {
    /// Always valid (no temporal bounds)
    Eternal,
    /// Currently valid (within bounds)
    Current,
    /// Was valid in the past (ended)
    Past,
    /// Will be valid in the future (not started)
    Future,
    /// Has both start and end bounds, currently within them
    Bounded,
}

impl TemporalValidity {
    /// Determine validity state from temporal bounds
    pub fn from_bounds(
        valid_from: Option<DateTime<Utc>>,
        valid_until: Option<DateTime<Utc>>,
    ) -> Self {
        Self::from_bounds_at(valid_from, valid_until, Utc::now())
    }

    /// Determine validity state at a specific time
    pub fn from_bounds_at(
        valid_from: Option<DateTime<Utc>>,
        valid_until: Option<DateTime<Utc>>,
        at_time: DateTime<Utc>,
    ) -> Self {
        match (valid_from, valid_until) {
            (None, None) => TemporalValidity::Eternal,
            (Some(from), None) => {
                if at_time >= from {
                    TemporalValidity::Current
                } else {
                    TemporalValidity::Future
                }
            }
            (None, Some(until)) => {
                if at_time <= until {
                    TemporalValidity::Current
                } else {
                    TemporalValidity::Past
                }
            }
            (Some(from), Some(until)) => {
                if at_time < from {
                    TemporalValidity::Future
                } else if at_time > until {
                    TemporalValidity::Past
                } else {
                    TemporalValidity::Bounded
                }
            }
        }
    }

    /// Check if this state represents currently valid knowledge
    pub fn is_valid(&self) -> bool {
        matches!(
            self,
            TemporalValidity::Eternal | TemporalValidity::Current | TemporalValidity::Bounded
        )
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temporal_range_contains() {
        let now = Utc::now();
        let yesterday = now - Duration::days(1);
        let tomorrow = now + Duration::days(1);

        let range = TemporalRange::between(yesterday, tomorrow);
        assert!(range.contains(now));
        assert!(range.contains(yesterday));
        assert!(range.contains(tomorrow));
        assert!(!range.contains(now - Duration::days(2)));
    }

    #[test]
    fn test_temporal_range_overlaps() {
        let now = Utc::now();
        let r1 = TemporalRange::between(now - Duration::days(2), now);
        let r2 = TemporalRange::between(now - Duration::days(1), now + Duration::days(1));
        let r3 = TemporalRange::between(now + Duration::days(2), now + Duration::days(3));

        assert!(r1.overlaps(&r2)); // They overlap
        assert!(!r1.overlaps(&r3)); // No overlap
    }

    #[test]
    fn test_temporal_validity() {
        let now = Utc::now();
        let yesterday = now - Duration::days(1);
        let tomorrow = now + Duration::days(1);

        // Eternal
        assert_eq!(
            TemporalValidity::from_bounds_at(None, None, now),
            TemporalValidity::Eternal
        );

        // Current (started, no end)
        assert_eq!(
            TemporalValidity::from_bounds_at(Some(yesterday), None, now),
            TemporalValidity::Current
        );

        // Future (not started yet)
        assert_eq!(
            TemporalValidity::from_bounds_at(Some(tomorrow), None, now),
            TemporalValidity::Future
        );

        // Past (ended)
        assert_eq!(
            TemporalValidity::from_bounds_at(None, Some(yesterday), now),
            TemporalValidity::Past
        );

        // Bounded (within range)
        assert_eq!(
            TemporalValidity::from_bounds_at(Some(yesterday), Some(tomorrow), now),
            TemporalValidity::Bounded
        );
    }

    #[test]
    fn test_validity_is_valid() {
        assert!(TemporalValidity::Eternal.is_valid());
        assert!(TemporalValidity::Current.is_valid());
        assert!(TemporalValidity::Bounded.is_valid());
        assert!(!TemporalValidity::Past.is_valid());
        assert!(!TemporalValidity::Future.is_valid());
    }
}
