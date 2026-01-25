//! Time Travel Environment for Testing Decay
//!
//! Enables testing of time-dependent memory behavior:
//! - FSRS-6 scheduling and intervals
//! - Memory decay (retrieval strength degradation)
//! - Temporal validity periods
//! - Consolidation timing
//!
//! Uses a virtual clock that can be advanced without waiting.

use chrono::{DateTime, Duration, Utc};
use std::cell::RefCell;

/// Environment for testing time-dependent memory behavior
///
/// Provides a virtual clock that can be advanced to test:
/// - Memory decay over time
/// - FSRS-6 scheduling calculations
/// - Temporal validity windows
/// - Consolidation cycles
///
/// # Example
///
/// ```rust,ignore
/// let mut env = TimeTravelEnvironment::new();
///
/// // Start at a known time
/// env.set_time(Utc::now());
///
/// // Advance 30 days to test decay
/// env.advance_days(30);
///
/// // Check retrievability at this point
/// let elapsed = env.days_since(original_time);
/// ```
pub struct TimeTravelEnvironment {
    /// Current virtual time
    current_time: RefCell<DateTime<Utc>>,
    /// Original start time for reference
    start_time: DateTime<Utc>,
    /// History of time jumps for debugging
    time_history: RefCell<Vec<TimeJump>>,
}

/// Record of a time jump for debugging
#[derive(Debug, Clone)]
pub struct TimeJump {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
    pub reason: String,
}

impl Default for TimeTravelEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeTravelEnvironment {
    /// Create a new time travel environment starting at the current time
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            current_time: RefCell::new(now),
            start_time: now,
            time_history: RefCell::new(Vec::new()),
        }
    }

    /// Create environment at a specific starting time
    pub fn at(time: DateTime<Utc>) -> Self {
        Self {
            current_time: RefCell::new(time),
            start_time: time,
            time_history: RefCell::new(Vec::new()),
        }
    }

    /// Get the current virtual time
    pub fn now(&self) -> DateTime<Utc> {
        *self.current_time.borrow()
    }

    /// Get the original start time
    pub fn start_time(&self) -> DateTime<Utc> {
        self.start_time
    }

    /// Set the current time to a specific point
    pub fn set_time(&self, time: DateTime<Utc>) {
        let from = *self.current_time.borrow();
        self.time_history.borrow_mut().push(TimeJump {
            from,
            to: time,
            reason: "set_time".to_string(),
        });
        *self.current_time.borrow_mut() = time;
    }

    /// Advance time by a duration
    pub fn advance(&self, duration: Duration) {
        let from = *self.current_time.borrow();
        let to = from + duration;
        self.time_history.borrow_mut().push(TimeJump {
            from,
            to,
            reason: format!("advance {:?}", duration),
        });
        *self.current_time.borrow_mut() = to;
    }

    /// Advance time by the specified number of days
    pub fn advance_days(&self, days: i64) {
        self.advance(Duration::days(days));
    }

    /// Advance time by the specified number of hours
    pub fn advance_hours(&self, hours: i64) {
        self.advance(Duration::hours(hours));
    }

    /// Advance time by the specified number of minutes
    pub fn advance_minutes(&self, minutes: i64) {
        self.advance(Duration::minutes(minutes));
    }

    /// Advance time by the specified number of seconds
    pub fn advance_seconds(&self, seconds: i64) {
        self.advance(Duration::seconds(seconds));
    }

    /// Calculate days elapsed since a reference time
    pub fn days_since(&self, reference: DateTime<Utc>) -> f64 {
        let current = *self.current_time.borrow();
        (current - reference).num_seconds() as f64 / 86400.0
    }

    /// Calculate days elapsed since the start time
    pub fn days_since_start(&self) -> f64 {
        self.days_since(self.start_time)
    }

    /// Calculate hours elapsed since a reference time
    pub fn hours_since(&self, reference: DateTime<Utc>) -> f64 {
        let current = *self.current_time.borrow();
        (current - reference).num_seconds() as f64 / 3600.0
    }

    /// Get time history for debugging
    pub fn get_history(&self) -> Vec<TimeJump> {
        self.time_history.borrow().clone()
    }

    /// Clear time history
    pub fn clear_history(&self) {
        self.time_history.borrow_mut().clear();
    }

    /// Reset to start time
    pub fn reset(&self) {
        let from = *self.current_time.borrow();
        self.time_history.borrow_mut().push(TimeJump {
            from,
            to: self.start_time,
            reason: "reset".to_string(),
        });
        *self.current_time.borrow_mut() = self.start_time;
    }

    // ========================================================================
    // DECAY TESTING HELPERS
    // ========================================================================

    /// Calculate expected retrievability at current time
    ///
    /// Uses FSRS-6 power forgetting curve:
    /// R = (1 + factor * t / S)^(-w20)
    pub fn expected_retrievability(&self, stability: f64, last_review: DateTime<Utc>) -> f64 {
        let elapsed_days = self.days_since(last_review);
        vestige_core::retrievability(stability, elapsed_days)
    }

    /// Calculate expected retrievability with custom decay
    pub fn expected_retrievability_with_decay(
        &self,
        stability: f64,
        last_review: DateTime<Utc>,
        w20: f64,
    ) -> f64 {
        let elapsed_days = self.days_since(last_review);
        vestige_core::retrievability_with_decay(stability, elapsed_days, w20)
    }

    /// Check if a memory would be due for review at current time
    pub fn is_due(&self, next_review: DateTime<Utc>) -> bool {
        *self.current_time.borrow() >= next_review
    }

    /// Calculate how overdue a memory is (negative if not yet due)
    pub fn days_overdue(&self, next_review: DateTime<Utc>) -> f64 {
        self.days_since(next_review)
    }

    // ========================================================================
    // SCHEDULING HELPERS
    // ========================================================================

    /// Advance to when a memory would be due
    pub fn advance_to_due(&self, next_review: DateTime<Utc>) {
        let from = *self.current_time.borrow();
        self.time_history.borrow_mut().push(TimeJump {
            from,
            to: next_review,
            reason: "advance_to_due".to_string(),
        });
        *self.current_time.borrow_mut() = next_review;
    }

    /// Advance past due date by specified days
    pub fn advance_past_due(&self, next_review: DateTime<Utc>, days_overdue: i64) {
        let target = next_review + Duration::days(days_overdue);
        let from = *self.current_time.borrow();
        self.time_history.borrow_mut().push(TimeJump {
            from,
            to: target,
            reason: format!("advance_past_due +{} days", days_overdue),
        });
        *self.current_time.borrow_mut() = target;
    }

    // ========================================================================
    // TEMPORAL VALIDITY HELPERS
    // ========================================================================

    /// Check if a time is within a validity window
    pub fn is_within_validity(
        &self,
        valid_from: Option<DateTime<Utc>>,
        valid_until: Option<DateTime<Utc>>,
    ) -> bool {
        let current = *self.current_time.borrow();
        let after_start = valid_from.map(|t| current >= t).unwrap_or(true);
        let before_end = valid_until.map(|t| current <= t).unwrap_or(true);
        after_start && before_end
    }

    /// Advance to just before validity starts
    pub fn advance_to_before_validity(&self, valid_from: DateTime<Utc>) {
        let target = valid_from - Duration::seconds(1);
        self.set_time(target);
    }

    /// Advance to just after validity ends
    pub fn advance_to_after_validity(&self, valid_until: DateTime<Utc>) {
        let target = valid_until + Duration::seconds(1);
        self.set_time(target);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_travel_basic() {
        let env = TimeTravelEnvironment::new();
        let start = env.now();

        env.advance_days(10);

        assert!(env.days_since(start) >= 9.99);
        assert!(env.days_since(start) <= 10.01);
    }

    #[test]
    fn test_time_travel_reset() {
        let env = TimeTravelEnvironment::new();
        let start = env.start_time();

        env.advance_days(100);
        env.reset();

        assert_eq!(env.now(), start);
    }

    #[test]
    fn test_retrievability_decay() {
        let env = TimeTravelEnvironment::new();
        let stability = 10.0;
        let last_review = env.now();

        // At t=0, retrievability should be ~1.0
        let r0 = env.expected_retrievability(stability, last_review);
        assert!(r0 > 0.99);

        // After 10 days with stability=10, retrievability should be ~0.9
        env.advance_days(10);
        let r10 = env.expected_retrievability(stability, last_review);
        assert!(r10 < r0);
        assert!(r10 > 0.85 && r10 < 0.95);

        // After 30 days, retrievability should be much lower
        env.advance_days(20);
        let r30 = env.expected_retrievability(stability, last_review);
        assert!(r30 < r10);
    }

    #[test]
    fn test_due_date_checking() {
        let env = TimeTravelEnvironment::new();
        let next_review = env.now() + Duration::days(5);

        // Not due yet
        assert!(!env.is_due(next_review));
        assert!(env.days_overdue(next_review) < 0.0);

        // Advance to due
        env.advance_to_due(next_review);
        assert!(env.is_due(next_review));
        assert!(env.days_overdue(next_review).abs() < 0.01);

        // Advance past due
        env.advance_days(3);
        assert!(env.is_due(next_review));
        assert!(env.days_overdue(next_review) > 2.99);
    }

    #[test]
    fn test_history_tracking() {
        let env = TimeTravelEnvironment::new();

        env.advance_days(1);
        env.advance_hours(12);
        env.advance_minutes(30);

        let history = env.get_history();
        assert_eq!(history.len(), 3);

        env.clear_history();
        assert!(env.get_history().is_empty());
    }
}
