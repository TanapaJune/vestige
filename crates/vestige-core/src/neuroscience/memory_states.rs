//! # Memory States System
//!
//! Implements the neuroscience concept that memories exist in different accessibility states.
//!
//! ## Background
//!
//! Modern memory science recognizes that memories don't simply "exist" or "not exist" -
//! they exist on a continuum of accessibility. A memory might be:
//!
//! - **Active**: Currently in working memory, immediately accessible
//! - **Dormant**: Easily retrievable with partial cues (like remembering a friend's name)
//! - **Silent**: Exists but requires strong/specific cues (like childhood memories)
//! - **Unavailable**: Temporarily blocked due to interference or suppression
//!
//! ## Key Phenomena Modeled
//!
//! 1. **State Decay**: Active memories naturally decay to Dormant, then Silent over time
//! 2. **Reactivation**: Strong cue matches can reactivate Silent memories
//! 3. **Retrieval-Induced Forgetting (RIF)**: Retrieving one memory can suppress related competitors
//! 4. **Interference**: Similar memories compete, with winners strengthening and losers weakening
//!
//! ## References
//!
//! - Bjork, R. A., & Bjork, E. L. (1992). A new theory of disuse and an old theory of stimulus fluctuation.
//! - Anderson, M. C., Bjork, R. A., & Bjork, E. L. (1994). Remembering can cause forgetting.
//! - Tulving, E. (1974). Cue-dependent forgetting. American Scientist.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

// ============================================================================
// CONSTANTS
// ============================================================================

/// Default time (in hours) before Active memories decay to Dormant
pub const DEFAULT_ACTIVE_DECAY_HOURS: i64 = 4;

/// Default time (in days) before Dormant memories decay to Silent
pub const DEFAULT_DORMANT_DECAY_DAYS: i64 = 30;

/// Base accessibility multiplier for Active state
pub const ACCESSIBILITY_ACTIVE: f64 = 1.0;

/// Base accessibility multiplier for Dormant state
pub const ACCESSIBILITY_DORMANT: f64 = 0.7;

/// Base accessibility multiplier for Silent state
pub const ACCESSIBILITY_SILENT: f64 = 0.3;

/// Base accessibility multiplier for Unavailable state
pub const ACCESSIBILITY_UNAVAILABLE: f64 = 0.05;

/// Minimum similarity threshold for competition to occur
pub const COMPETITION_SIMILARITY_THRESHOLD: f64 = 0.6;

/// Suppression strength applied to losers in retrieval competition
pub const COMPETITION_SUPPRESSION_FACTOR: f64 = 0.15;

/// Maximum number of state transitions to keep in history
pub const MAX_STATE_HISTORY_SIZE: usize = 50;

/// Maximum number of competition events to track
pub const MAX_COMPETITION_HISTORY_SIZE: usize = 100;

// ============================================================================
// MEMORY STATE ENUM
// ============================================================================

/// The accessibility state of a memory.
///
/// Memories transition between these states based on:
/// - Time since last access
/// - Strength of retrieval cues
/// - Competition with similar memories
///
/// # State Accessibility
///
/// | State       | Multiplier | Description                          |
/// |-------------|------------|--------------------------------------|
/// | Active      | 1.0        | Currently being processed            |
/// | Dormant     | 0.7        | Easily retrievable with partial cues |
/// | Silent      | 0.3        | Requires strong/specific cues        |
/// | Unavailable | 0.05       | Temporarily blocked                  |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum MemoryState {
    /// Currently being processed, high accessibility.
    ///
    /// This is the state immediately after a memory is created or accessed.
    /// The memory is in "working memory" and immediately available.
    #[default]
    Active,

    /// Easily retrievable with partial cues, moderate accessibility.
    ///
    /// Like remembering a friend's name when you see their face.
    /// The memory is well-consolidated and doesn't require much effort to retrieve.
    Dormant,

    /// Exists but requires strong/specific cues to retrieve.
    ///
    /// Like childhood memories that only surface with specific triggers.
    /// The memory exists but needs substantial cue overlap to be activated.
    Silent,

    /// Temporarily inaccessible due to interference or suppression.
    ///
    /// The memory is blocked, often because:
    /// - A similar memory "won" a retrieval competition
    /// - The user actively suppressed the memory
    /// - Too many similar memories are competing
    ///
    /// This state is reversible - the memory can become accessible again
    /// once interference is resolved or suppression expires.
    Unavailable,
}

impl MemoryState {
    /// Get the base accessibility multiplier for this state.
    ///
    /// This multiplier should be factored into retrieval ranking:
    /// `effective_score = raw_score * accessibility_multiplier`
    ///
    /// # Returns
    ///
    /// A value between 0.0 and 1.0 representing the state's base accessibility.
    #[inline]
    pub fn accessibility_multiplier(&self) -> f64 {
        match self {
            MemoryState::Active => ACCESSIBILITY_ACTIVE,
            MemoryState::Dormant => ACCESSIBILITY_DORMANT,
            MemoryState::Silent => ACCESSIBILITY_SILENT,
            MemoryState::Unavailable => ACCESSIBILITY_UNAVAILABLE,
        }
    }

    /// Check if this state allows normal retrieval.
    ///
    /// Active and Dormant memories can be retrieved with normal cues.
    /// Silent memories require stronger cues (higher similarity threshold).
    /// Unavailable memories are blocked until suppression expires.
    #[inline]
    pub fn is_retrievable(&self) -> bool {
        matches!(self, MemoryState::Active | MemoryState::Dormant)
    }

    /// Check if this state requires strong cues for retrieval.
    #[inline]
    pub fn requires_strong_cue(&self) -> bool {
        matches!(self, MemoryState::Silent)
    }

    /// Check if this state blocks retrieval.
    #[inline]
    pub fn is_blocked(&self) -> bool {
        matches!(self, MemoryState::Unavailable)
    }

    /// Get a human-readable description of the state.
    pub fn description(&self) -> &'static str {
        match self {
            MemoryState::Active => "Currently in working memory, immediately accessible",
            MemoryState::Dormant => "Well-consolidated, easily retrievable with partial cues",
            MemoryState::Silent => "Exists but requires strong or specific cues to surface",
            MemoryState::Unavailable => "Temporarily blocked due to interference or suppression",
        }
    }

    /// Convert to string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            MemoryState::Active => "active",
            MemoryState::Dormant => "dormant",
            MemoryState::Silent => "silent",
            MemoryState::Unavailable => "unavailable",
        }
    }

    /// Parse from string representation.
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "active" => MemoryState::Active,
            "dormant" => MemoryState::Dormant,
            "silent" => MemoryState::Silent,
            "unavailable" => MemoryState::Unavailable,
            _ => MemoryState::Dormant, // Safe default
        }
    }
}

impl std::fmt::Display for MemoryState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ============================================================================
// STATE TRANSITION REASON
// ============================================================================

/// The reason for a state transition.
///
/// Tracking reasons provides transparency about why memories
/// change accessibility over time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StateTransitionReason {
    /// Memory was just created or accessed
    Access,
    /// Time-based decay (natural forgetting)
    TimeDecay,
    /// Strong cue reactivated a Silent memory
    CueReactivation {
        /// Similarity score of the cue that triggered reactivation
        cue_similarity: f64,
    },
    /// Lost a retrieval competition to another memory
    CompetitionLoss {
        /// ID of the winning memory
        winner_id: String,
        /// How similar the memories were
        similarity: f64,
    },
    /// Competition resolved, interference no longer blocking
    InterferenceResolved,
    /// User explicitly suppressed the memory
    UserSuppression {
        /// Optional reason provided by user
        reason: Option<String>,
    },
    /// Suppression period expired
    SuppressionExpired,
    /// Manual state override (e.g., admin action)
    ManualOverride {
        /// Who made the change
        actor: Option<String>,
    },
    /// System initialization or migration
    SystemInit,
}

impl StateTransitionReason {
    /// Get a human-readable description of the reason.
    pub fn description(&self) -> String {
        match self {
            StateTransitionReason::Access => "Memory was accessed or created".to_string(),
            StateTransitionReason::TimeDecay => "Natural decay over time".to_string(),
            StateTransitionReason::CueReactivation { cue_similarity } => {
                format!(
                    "Reactivated by strong cue (similarity: {:.2})",
                    cue_similarity
                )
            }
            StateTransitionReason::CompetitionLoss {
                winner_id,
                similarity,
            } => {
                format!(
                    "Lost retrieval competition to {} (similarity: {:.2})",
                    winner_id, similarity
                )
            }
            StateTransitionReason::InterferenceResolved => {
                "Interference from competing memories resolved".to_string()
            }
            StateTransitionReason::UserSuppression { reason } => match reason {
                Some(r) => format!("User suppressed: {}", r),
                None => "User suppressed memory".to_string(),
            },
            StateTransitionReason::SuppressionExpired => "Suppression period expired".to_string(),
            StateTransitionReason::ManualOverride { actor } => match actor {
                Some(a) => format!("Manual override by {}", a),
                None => "Manual state override".to_string(),
            },
            StateTransitionReason::SystemInit => "System initialization".to_string(),
        }
    }
}

// ============================================================================
// STATE TRANSITION
// ============================================================================

/// A recorded state transition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateTransition {
    /// The previous state
    pub from_state: MemoryState,
    /// The new state
    pub to_state: MemoryState,
    /// When the transition occurred
    pub timestamp: DateTime<Utc>,
    /// Why the transition happened
    pub reason: StateTransitionReason,
}

impl StateTransition {
    /// Create a new state transition record.
    pub fn new(
        from_state: MemoryState,
        to_state: MemoryState,
        reason: StateTransitionReason,
    ) -> Self {
        Self {
            from_state,
            to_state,
            timestamp: Utc::now(),
            reason,
        }
    }
}

// ============================================================================
// COMPETITION EVENT
// ============================================================================

/// Records a retrieval competition event.
///
/// When similar memories compete during retrieval, we track:
/// - Which memories competed
/// - Who won
/// - The suppression applied to losers
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompetitionEvent {
    /// ID of the query/cue that triggered competition
    pub query_id: Option<String>,
    /// The query/cue text (for debugging)
    pub query_text: Option<String>,
    /// ID of the winning memory
    pub winner_id: String,
    /// IDs of memories that lost (and were suppressed)
    pub loser_ids: Vec<String>,
    /// Similarity scores between winner and each loser
    pub loser_similarities: Vec<f64>,
    /// When the competition occurred
    pub timestamp: DateTime<Utc>,
    /// How long suppression lasts for losers
    pub suppression_duration: Duration,
}

impl CompetitionEvent {
    /// Create a new competition event.
    pub fn new(
        winner_id: String,
        loser_ids: Vec<String>,
        loser_similarities: Vec<f64>,
        suppression_duration: Duration,
    ) -> Self {
        Self {
            query_id: None,
            query_text: None,
            winner_id,
            loser_ids,
            loser_similarities,
            timestamp: Utc::now(),
            suppression_duration,
        }
    }

    /// Add query information to the event.
    pub fn with_query(mut self, query_id: Option<String>, query_text: Option<String>) -> Self {
        self.query_id = query_id;
        self.query_text = query_text;
        self
    }
}

// ============================================================================
// MEMORY LIFECYCLE
// ============================================================================

/// Tracks the complete lifecycle and state of a memory.
///
/// This struct should be embedded in or associated with each Memory
/// to track its accessibility state over time.
///
/// # Example
///
/// ```rust
/// use vestige_core::neuroscience::{MemoryLifecycle, MemoryState};
///
/// // Create a new lifecycle (starts Active)
/// let mut lifecycle = MemoryLifecycle::new();
/// assert_eq!(lifecycle.state, MemoryState::Active);
///
/// // Record an access
/// lifecycle.record_access();
///
/// // Check if memory should decay
/// let config = lifecycle.decay_config();
/// if lifecycle.should_decay_to_dormant(&config) {
///     lifecycle.transition_to(MemoryState::Dormant,
///         vestige_core::neuroscience::StateTransitionReason::TimeDecay);
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryLifecycle {
    /// Current accessibility state
    pub state: MemoryState,
    /// When the memory was last accessed
    pub last_access: DateTime<Utc>,
    /// Total number of times this memory has been accessed
    pub access_count: u32,
    /// History of state transitions (most recent last)
    pub state_history: VecDeque<StateTransition>,
    /// If Unavailable due to suppression, when it expires
    pub suppression_until: Option<DateTime<Utc>>,
    /// IDs of memories that have suppressed this one
    pub suppressed_by: Vec<String>,
    /// When the current state was entered
    pub state_entered_at: DateTime<Utc>,
    /// Total time spent in each state (for analytics)
    pub time_in_states: StateTimeAccumulator,
}

impl Default for MemoryLifecycle {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryLifecycle {
    /// Create a new lifecycle in the Active state.
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            state: MemoryState::Active,
            last_access: now,
            access_count: 1,
            state_history: VecDeque::with_capacity(MAX_STATE_HISTORY_SIZE),
            suppression_until: None,
            suppressed_by: Vec::new(),
            state_entered_at: now,
            time_in_states: StateTimeAccumulator::default(),
        }
    }

    /// Create a lifecycle with a specific initial state.
    pub fn with_state(state: MemoryState) -> Self {
        let mut lifecycle = Self::new();
        lifecycle.state = state;
        lifecycle.state_history.push_back(StateTransition::new(
            MemoryState::Active,
            state,
            StateTransitionReason::SystemInit,
        ));
        lifecycle
    }

    /// Record an access to this memory.
    ///
    /// Accessing a memory:
    /// 1. Resets it to Active state (if not suppressed)
    /// 2. Updates last_access timestamp
    /// 3. Increments access count
    ///
    /// # Returns
    ///
    /// Whether the state changed (i.e., memory was reactivated).
    pub fn record_access(&mut self) -> bool {
        self.last_access = Utc::now();
        self.access_count = self.access_count.saturating_add(1);

        // Can't reactivate if suppressed
        if self.state == MemoryState::Unavailable && !self.is_suppression_expired() {
            return false;
        }

        if self.state != MemoryState::Active {
            self.transition_to(MemoryState::Active, StateTransitionReason::Access);
            true
        } else {
            false
        }
    }

    /// Transition to a new state with a reason.
    pub fn transition_to(&mut self, new_state: MemoryState, reason: StateTransitionReason) {
        if self.state == new_state {
            return; // No change
        }

        // Update time accumulator
        let now = Utc::now();
        let time_in_current = now
            .signed_duration_since(self.state_entered_at)
            .num_seconds()
            .max(0) as u64;
        self.time_in_states.add(self.state, time_in_current);

        // Record transition
        let transition = StateTransition::new(self.state, new_state, reason);
        self.state_history.push_back(transition);

        // Trim history if needed
        while self.state_history.len() > MAX_STATE_HISTORY_SIZE {
            self.state_history.pop_front();
        }

        // Update state
        self.state = new_state;
        self.state_entered_at = now;

        // Clear suppression if leaving Unavailable
        if new_state != MemoryState::Unavailable {
            self.suppression_until = None;
            self.suppressed_by.clear();
        }
    }

    /// Suppress this memory due to competition loss.
    ///
    /// # Arguments
    ///
    /// * `winner_id` - ID of the memory that won the competition
    /// * `similarity` - How similar this memory was to the winner
    /// * `duration` - How long the suppression lasts
    pub fn suppress_from_competition(
        &mut self,
        winner_id: String,
        similarity: f64,
        duration: Duration,
    ) {
        let reason = StateTransitionReason::CompetitionLoss {
            winner_id: winner_id.clone(),
            similarity,
        };

        self.transition_to(MemoryState::Unavailable, reason);
        self.suppression_until = Some(Utc::now() + duration);
        self.suppressed_by.push(winner_id);
    }

    /// Suppress this memory due to user action.
    ///
    /// # Arguments
    ///
    /// * `duration` - How long the suppression lasts
    /// * `reason` - Optional reason from the user
    pub fn suppress_by_user(&mut self, duration: Duration, reason: Option<String>) {
        self.transition_to(
            MemoryState::Unavailable,
            StateTransitionReason::UserSuppression { reason },
        );
        self.suppression_until = Some(Utc::now() + duration);
    }

    /// Check if suppression has expired.
    pub fn is_suppression_expired(&self) -> bool {
        self.suppression_until
            .map(|until| Utc::now() >= until)
            .unwrap_or(true)
    }

    /// Get the default decay configuration.
    pub fn decay_config(&self) -> StateDecayConfig {
        StateDecayConfig::default()
    }

    /// Check if this memory should decay from Active to Dormant.
    pub fn should_decay_to_dormant(&self, config: &StateDecayConfig) -> bool {
        if self.state != MemoryState::Active {
            return false;
        }

        let hours_since_access = Utc::now()
            .signed_duration_since(self.last_access)
            .num_hours();
        hours_since_access >= config.active_decay_hours
    }

    /// Check if this memory should decay from Dormant to Silent.
    pub fn should_decay_to_silent(&self, config: &StateDecayConfig) -> bool {
        if self.state != MemoryState::Dormant {
            return false;
        }

        let days_since_access = Utc::now()
            .signed_duration_since(self.last_access)
            .num_days();
        days_since_access >= config.dormant_decay_days
    }

    /// Try to reactivate from Silent with a strong cue.
    ///
    /// # Arguments
    ///
    /// * `cue_similarity` - Similarity score of the retrieval cue
    /// * `threshold` - Minimum similarity required for reactivation
    ///
    /// # Returns
    ///
    /// Whether reactivation succeeded.
    pub fn try_reactivate_with_cue(&mut self, cue_similarity: f64, threshold: f64) -> bool {
        if self.state != MemoryState::Silent {
            return false;
        }

        if cue_similarity >= threshold {
            self.transition_to(
                MemoryState::Dormant,
                StateTransitionReason::CueReactivation { cue_similarity },
            );
            true
        } else {
            false
        }
    }

    /// Get the current accessibility multiplier.
    pub fn accessibility(&self) -> f64 {
        self.state.accessibility_multiplier()
    }

    /// Get a summary of this lifecycle for debugging/display.
    pub fn summary(&self) -> LifecycleSummary {
        LifecycleSummary {
            state: self.state,
            state_description: self.state.description().to_string(),
            accessibility: self.accessibility(),
            access_count: self.access_count,
            last_access: self.last_access,
            time_in_current_state: Utc::now()
                .signed_duration_since(self.state_entered_at)
                .num_seconds()
                .max(0) as u64,
            total_transitions: self.state_history.len(),
            is_suppressed: self.state == MemoryState::Unavailable && !self.is_suppression_expired(),
            suppression_expires: self.suppression_until,
        }
    }
}

// ============================================================================
// STATE TIME ACCUMULATOR
// ============================================================================

/// Accumulates time spent in each state.
///
/// Useful for analytics and understanding memory behavior over time.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateTimeAccumulator {
    /// Seconds spent in Active state
    pub active_seconds: u64,
    /// Seconds spent in Dormant state
    pub dormant_seconds: u64,
    /// Seconds spent in Silent state
    pub silent_seconds: u64,
    /// Seconds spent in Unavailable state
    pub unavailable_seconds: u64,
}

impl StateTimeAccumulator {
    /// Add time to the appropriate state counter.
    pub fn add(&mut self, state: MemoryState, seconds: u64) {
        match state {
            MemoryState::Active => self.active_seconds += seconds,
            MemoryState::Dormant => self.dormant_seconds += seconds,
            MemoryState::Silent => self.silent_seconds += seconds,
            MemoryState::Unavailable => self.unavailable_seconds += seconds,
        }
    }

    /// Get total tracked time across all states.
    pub fn total_seconds(&self) -> u64 {
        self.active_seconds + self.dormant_seconds + self.silent_seconds + self.unavailable_seconds
    }

    /// Get percentage of time spent in each state.
    pub fn percentages(&self) -> StatePercentages {
        let total = self.total_seconds().max(1) as f64;
        StatePercentages {
            active: (self.active_seconds as f64 / total) * 100.0,
            dormant: (self.dormant_seconds as f64 / total) * 100.0,
            silent: (self.silent_seconds as f64 / total) * 100.0,
            unavailable: (self.unavailable_seconds as f64 / total) * 100.0,
        }
    }
}

/// Percentage breakdown of time spent in each state.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatePercentages {
    pub active: f64,
    pub dormant: f64,
    pub silent: f64,
    pub unavailable: f64,
}

// ============================================================================
// LIFECYCLE SUMMARY
// ============================================================================

/// A summary of a memory's lifecycle for display/debugging.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleSummary {
    pub state: MemoryState,
    pub state_description: String,
    pub accessibility: f64,
    pub access_count: u32,
    pub last_access: DateTime<Utc>,
    pub time_in_current_state: u64,
    pub total_transitions: usize,
    pub is_suppressed: bool,
    pub suppression_expires: Option<DateTime<Utc>>,
}

// ============================================================================
// STATE DECAY CONFIGURATION
// ============================================================================

/// Configuration for automatic state decay.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateDecayConfig {
    /// Hours before Active decays to Dormant
    pub active_decay_hours: i64,
    /// Days before Dormant decays to Silent
    pub dormant_decay_days: i64,
    /// Similarity threshold for cue reactivation of Silent memories
    pub reactivation_threshold: f64,
    /// Default suppression duration for competition losses
    pub competition_suppression_duration: Duration,
    /// Whether to automatically resolve expired suppressions
    pub auto_resolve_suppression: bool,
}

impl Default for StateDecayConfig {
    fn default() -> Self {
        Self {
            active_decay_hours: DEFAULT_ACTIVE_DECAY_HOURS,
            dormant_decay_days: DEFAULT_DORMANT_DECAY_DAYS,
            reactivation_threshold: 0.8,
            competition_suppression_duration: Duration::hours(24),
            auto_resolve_suppression: true,
        }
    }
}

// ============================================================================
// RETRIEVAL COMPETITION MANAGER
// ============================================================================

/// Manages retrieval-induced forgetting (RIF) and memory competition.
///
/// When multiple similar memories compete during retrieval:
/// 1. The winner gets strengthened (moved to Active)
/// 2. The losers get suppressed (moved to Unavailable)
/// 3. This implements the neuroscience concept of retrieval-induced forgetting
///
/// # Example
///
/// ```rust
/// use vestige_core::neuroscience::{CompetitionManager, CompetitionCandidate};
///
/// let mut manager = CompetitionManager::new();
///
/// let candidates = vec![
///     CompetitionCandidate {
///         memory_id: "mem1".to_string(),
///         relevance_score: 0.95,
///         similarity_to_query: 0.9,
///     },
///     CompetitionCandidate {
///         memory_id: "mem2".to_string(),
///         relevance_score: 0.80,
///         similarity_to_query: 0.85,
///     },
/// ];
///
/// // Winner: mem1, Loser: mem2 (if similar enough)
/// if let Some(result) = manager.run_competition(&candidates, 0.6) {
///     println!("Winner: {}", result.winner_id);
///     println!("Suppressed: {:?}", result.suppressed_ids);
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompetitionManager {
    /// Configuration for competition behavior
    pub config: CompetitionConfig,
    /// History of competition events
    pub history: VecDeque<CompetitionEvent>,
}

impl Default for CompetitionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CompetitionManager {
    /// Create a new competition manager with default config.
    pub fn new() -> Self {
        Self {
            config: CompetitionConfig::default(),
            history: VecDeque::with_capacity(MAX_COMPETITION_HISTORY_SIZE),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(config: CompetitionConfig) -> Self {
        Self {
            config,
            history: VecDeque::with_capacity(MAX_COMPETITION_HISTORY_SIZE),
        }
    }

    /// Run a competition among candidate memories.
    ///
    /// # Arguments
    ///
    /// * `candidates` - Memories competing for retrieval
    /// * `similarity_threshold` - Minimum similarity for memories to compete
    ///
    /// # Returns
    ///
    /// Competition result if competition occurred, None if not enough similar candidates.
    pub fn run_competition(
        &mut self,
        candidates: &[CompetitionCandidate],
        similarity_threshold: f64,
    ) -> Option<CompetitionResult> {
        if candidates.len() < 2 {
            return None;
        }

        // Sort by relevance score (highest first = winner)
        let mut sorted = candidates.to_vec();
        sorted.sort_by(|a, b| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let winner = &sorted[0];
        let mut suppressed_ids = Vec::new();
        let mut suppressed_similarities = Vec::new();

        // Check each other candidate for competition
        for loser in sorted.iter().skip(1) {
            // Calculate similarity between winner and loser
            // Using the simpler of: both were similar to the same query
            let similarity = (winner.similarity_to_query + loser.similarity_to_query) / 2.0;

            if similarity >= similarity_threshold {
                suppressed_ids.push(loser.memory_id.clone());
                suppressed_similarities.push(similarity);
            }
        }

        if suppressed_ids.is_empty() {
            return None; // No competition occurred
        }

        // Record the event
        let event = CompetitionEvent::new(
            winner.memory_id.clone(),
            suppressed_ids.clone(),
            suppressed_similarities.clone(),
            self.config.suppression_duration,
        );
        self.record_event(event);

        Some(CompetitionResult {
            winner_id: winner.memory_id.clone(),
            winner_boost: self.config.winner_boost,
            suppressed_ids,
            suppressed_similarities,
            suppression_duration: self.config.suppression_duration,
        })
    }

    /// Record a competition event.
    fn record_event(&mut self, event: CompetitionEvent) {
        self.history.push_back(event);
        while self.history.len() > MAX_COMPETITION_HISTORY_SIZE {
            self.history.pop_front();
        }
    }

    /// Get memories that have been suppressed by a specific winner.
    pub fn get_suppressed_by(&self, winner_id: &str) -> Vec<&CompetitionEvent> {
        self.history
            .iter()
            .filter(|e| e.winner_id == winner_id)
            .collect()
    }

    /// Get how many times a memory has been suppressed.
    pub fn suppression_count(&self, memory_id: &str) -> usize {
        self.history
            .iter()
            .filter(|e| e.loser_ids.contains(&memory_id.to_string()))
            .count()
    }

    /// Get how many times a memory has won competitions.
    pub fn win_count(&self, memory_id: &str) -> usize {
        self.history
            .iter()
            .filter(|e| e.winner_id == memory_id)
            .count()
    }

    /// Clear competition history.
    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

// ============================================================================
// COMPETITION TYPES
// ============================================================================

/// Configuration for retrieval competition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompetitionConfig {
    /// Minimum similarity for memories to compete
    pub similarity_threshold: f64,
    /// How much to boost the winner's strength
    pub winner_boost: f64,
    /// How long losers are suppressed
    pub suppression_duration: Duration,
    /// Whether to track competition history
    pub track_history: bool,
}

impl Default for CompetitionConfig {
    fn default() -> Self {
        Self {
            similarity_threshold: COMPETITION_SIMILARITY_THRESHOLD,
            winner_boost: 0.1,
            suppression_duration: Duration::hours(24),
            track_history: true,
        }
    }
}

/// A candidate in a retrieval competition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompetitionCandidate {
    /// Unique ID of the memory
    pub memory_id: String,
    /// How relevant this memory is to the query (higher = more likely to win)
    pub relevance_score: f64,
    /// How similar this memory is to the query
    pub similarity_to_query: f64,
}

/// Result of a retrieval competition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompetitionResult {
    /// ID of the winning memory
    pub winner_id: String,
    /// How much to boost the winner
    pub winner_boost: f64,
    /// IDs of suppressed (losing) memories
    pub suppressed_ids: Vec<String>,
    /// Similarity of each suppressed memory to winner
    pub suppressed_similarities: Vec<f64>,
    /// How long suppression lasts
    pub suppression_duration: Duration,
}

// ============================================================================
// STATE UPDATE SERVICE
// ============================================================================

/// Service for updating memory states based on time and access patterns.
///
/// This should be run periodically (e.g., as a background task) to:
/// 1. Decay Active memories to Dormant
/// 2. Decay Dormant memories to Silent
/// 3. Resolve expired suppressions
///
/// # Example
///
/// ```rust
/// use vestige_core::neuroscience::{StateUpdateService, MemoryLifecycle, MemoryState};
///
/// let service = StateUpdateService::new();
/// let mut lifecycle = MemoryLifecycle::new();
///
/// // Check and apply any needed transitions
/// let transitions = service.update_lifecycle(&mut lifecycle);
/// println!("Applied {} transitions", transitions.len());
/// ```
#[derive(Debug, Clone)]
pub struct StateUpdateService {
    config: StateDecayConfig,
}

impl Default for StateUpdateService {
    fn default() -> Self {
        Self::new()
    }
}

impl StateUpdateService {
    /// Create a new update service with default config.
    pub fn new() -> Self {
        Self {
            config: StateDecayConfig::default(),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(config: StateDecayConfig) -> Self {
        Self { config }
    }

    /// Get the configuration.
    pub fn config(&self) -> &StateDecayConfig {
        &self.config
    }

    /// Update a single lifecycle, applying any needed transitions.
    ///
    /// # Returns
    ///
    /// List of transitions that were applied.
    pub fn update_lifecycle(&self, lifecycle: &mut MemoryLifecycle) -> Vec<StateTransition> {
        let mut transitions = Vec::new();

        // Check for suppression expiry first
        if lifecycle.state == MemoryState::Unavailable
            && lifecycle.is_suppression_expired()
            && self.config.auto_resolve_suppression
        {
            let from = lifecycle.state;
            lifecycle.transition_to(
                MemoryState::Dormant,
                StateTransitionReason::SuppressionExpired,
            );
            transitions.push(StateTransition::new(
                from,
                MemoryState::Dormant,
                StateTransitionReason::SuppressionExpired,
            ));
        }

        // Check for Active -> Dormant decay
        if lifecycle.should_decay_to_dormant(&self.config) {
            let from = lifecycle.state;
            lifecycle.transition_to(MemoryState::Dormant, StateTransitionReason::TimeDecay);
            transitions.push(StateTransition::new(
                from,
                MemoryState::Dormant,
                StateTransitionReason::TimeDecay,
            ));
        }

        // Check for Dormant -> Silent decay
        if lifecycle.should_decay_to_silent(&self.config) {
            let from = lifecycle.state;
            lifecycle.transition_to(MemoryState::Silent, StateTransitionReason::TimeDecay);
            transitions.push(StateTransition::new(
                from,
                MemoryState::Silent,
                StateTransitionReason::TimeDecay,
            ));
        }

        transitions
    }

    /// Batch update multiple lifecycles.
    ///
    /// # Returns
    ///
    /// Total number of transitions applied.
    pub fn batch_update(&self, lifecycles: &mut [MemoryLifecycle]) -> BatchUpdateResult {
        let mut result = BatchUpdateResult::default();

        for lifecycle in lifecycles {
            let transitions = self.update_lifecycle(lifecycle);
            for t in transitions {
                match t.to_state {
                    MemoryState::Dormant => {
                        if matches!(t.reason, StateTransitionReason::SuppressionExpired) {
                            result.suppressions_resolved += 1;
                        } else {
                            result.active_to_dormant += 1;
                        }
                    }
                    MemoryState::Silent => result.dormant_to_silent += 1,
                    _ => {}
                }
                result.total_transitions += 1;
            }
        }

        result
    }
}

/// Result of a batch update operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchUpdateResult {
    /// Total transitions applied
    pub total_transitions: usize,
    /// Active -> Dormant transitions
    pub active_to_dormant: usize,
    /// Dormant -> Silent transitions
    pub dormant_to_silent: usize,
    /// Suppressions that were resolved
    pub suppressions_resolved: usize,
}

// ============================================================================
// ACCESSIBILITY CALCULATOR
// ============================================================================

/// Calculates effective accessibility scores for retrieval ranking.
///
/// Combines:
/// - State-based accessibility (Active: 1.0, Dormant: 0.7, Silent: 0.3, Unavailable: 0.05)
/// - Recency boost (recently accessed memories get a boost)
/// - Access frequency boost (frequently accessed memories get a boost)
#[derive(Debug, Clone)]
pub struct AccessibilityCalculator {
    /// Weight for recency in the final score (0.0-1.0)
    pub recency_weight: f64,
    /// Weight for access frequency in the final score (0.0-1.0)
    pub frequency_weight: f64,
    /// Half-life for recency decay in hours
    pub recency_half_life_hours: f64,
    /// Access count at which frequency bonus maxes out
    pub frequency_saturation_count: u32,
}

impl Default for AccessibilityCalculator {
    fn default() -> Self {
        Self {
            recency_weight: 0.15,
            frequency_weight: 0.1,
            recency_half_life_hours: 24.0,
            frequency_saturation_count: 10,
        }
    }
}

impl AccessibilityCalculator {
    /// Calculate the effective accessibility score for a memory.
    ///
    /// # Arguments
    ///
    /// * `lifecycle` - The memory's lifecycle state
    /// * `base_score` - The base relevance score from search (0.0-1.0)
    ///
    /// # Returns
    ///
    /// Adjusted score factoring in accessibility (0.0-1.0).
    pub fn calculate(&self, lifecycle: &MemoryLifecycle, base_score: f64) -> f64 {
        let state_multiplier = lifecycle.state.accessibility_multiplier();

        // Recency boost: exponential decay based on time since last access
        let hours_since_access = Utc::now()
            .signed_duration_since(lifecycle.last_access)
            .num_minutes() as f64
            / 60.0;
        let recency_factor = 0.5_f64.powf(hours_since_access / self.recency_half_life_hours);
        let recency_boost = recency_factor * self.recency_weight;

        // Frequency boost: logarithmic saturation
        let frequency_factor = (lifecycle.access_count as f64)
            .min(self.frequency_saturation_count as f64)
            / self.frequency_saturation_count as f64;
        let frequency_boost = frequency_factor * self.frequency_weight;

        // Combine: base * state_multiplier + boosts
        let raw_score = base_score * state_multiplier + recency_boost + frequency_boost;

        // Clamp to valid range
        raw_score.clamp(0.0, 1.0)
    }

    /// Calculate minimum similarity threshold for a given state.
    ///
    /// Silent memories require higher similarity to be retrieved.
    pub fn minimum_similarity_for_state(&self, state: MemoryState, base_threshold: f64) -> f64 {
        match state {
            MemoryState::Active => base_threshold * 0.8, // Lower threshold
            MemoryState::Dormant => base_threshold,
            MemoryState::Silent => base_threshold * 1.5, // Higher threshold
            MemoryState::Unavailable => 1.1,             // Effectively unreachable
        }
    }
}

// ============================================================================
// MEMORY STATE QUERY RESULT
// ============================================================================

/// Extended information about a memory's state for user queries.
///
/// This provides transparency about why a memory might be harder to access.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryStateInfo {
    /// Current state
    pub state: MemoryState,
    /// Human-readable explanation of current state
    pub explanation: String,
    /// Current accessibility (0.0-1.0)
    pub accessibility: f64,
    /// How many times accessed
    pub access_count: u32,
    /// Last access time
    pub last_access: DateTime<Utc>,
    /// Time since last access in human-readable format
    pub time_since_access: String,
    /// If applicable, why the memory is in this state
    pub state_reason: Option<String>,
    /// If suppressed, when it will be accessible again
    pub accessible_after: Option<DateTime<Utc>>,
    /// Recent state transitions
    pub recent_transitions: Vec<StateTransition>,
    /// Recommendations for improving accessibility
    pub recommendations: Vec<String>,
}

impl MemoryStateInfo {
    /// Create state info from a lifecycle.
    pub fn from_lifecycle(lifecycle: &MemoryLifecycle) -> Self {
        let now = Utc::now();
        let duration_since_access = now.signed_duration_since(lifecycle.last_access);

        // Format time since access
        let time_since_access = if duration_since_access.num_days() > 0 {
            format!("{} days ago", duration_since_access.num_days())
        } else if duration_since_access.num_hours() > 0 {
            format!("{} hours ago", duration_since_access.num_hours())
        } else if duration_since_access.num_minutes() > 0 {
            format!("{} minutes ago", duration_since_access.num_minutes())
        } else {
            "just now".to_string()
        };

        // Get state reason from most recent transition
        let state_reason = lifecycle
            .state_history
            .back()
            .map(|t| t.reason.description());

        // Generate recommendations
        let mut recommendations = Vec::new();
        match lifecycle.state {
            MemoryState::Silent => {
                recommendations.push(
                    "This memory needs a strong, specific cue to be retrieved. \
                     Try using more detailed search terms."
                        .to_string(),
                );
            }
            MemoryState::Unavailable => {
                if let Some(until) = lifecycle.suppression_until {
                    if until > now {
                        recommendations.push(format!(
                            "This memory is temporarily suppressed. \
                             It will become accessible again after {}.",
                            until.format("%Y-%m-%d %H:%M UTC")
                        ));
                    }
                }
            }
            MemoryState::Dormant => {
                if duration_since_access.num_days() > 20 {
                    recommendations.push(
                        "Consider accessing this memory soon to prevent it from \
                         becoming harder to retrieve."
                            .to_string(),
                    );
                }
            }
            _ => {}
        }

        // Get recent transitions (last 5)
        let recent_transitions: Vec<_> = lifecycle
            .state_history
            .iter()
            .rev()
            .take(5)
            .cloned()
            .collect();

        Self {
            state: lifecycle.state,
            explanation: lifecycle.state.description().to_string(),
            accessibility: lifecycle.accessibility(),
            access_count: lifecycle.access_count,
            last_access: lifecycle.last_access,
            time_since_access,
            state_reason,
            accessible_after: if lifecycle.state == MemoryState::Unavailable {
                lifecycle.suppression_until
            } else {
                None
            },
            recent_transitions,
            recommendations,
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f64, b: f64, epsilon: f64) -> bool {
        (a - b).abs() < epsilon
    }

    // ==================== MemoryState Tests ====================

    #[test]
    fn test_memory_state_accessibility() {
        assert!(approx_eq(
            MemoryState::Active.accessibility_multiplier(),
            1.0,
            0.001
        ));
        assert!(approx_eq(
            MemoryState::Dormant.accessibility_multiplier(),
            0.7,
            0.001
        ));
        assert!(approx_eq(
            MemoryState::Silent.accessibility_multiplier(),
            0.3,
            0.001
        ));
        assert!(approx_eq(
            MemoryState::Unavailable.accessibility_multiplier(),
            0.05,
            0.001
        ));
    }

    #[test]
    fn test_memory_state_retrievability() {
        assert!(MemoryState::Active.is_retrievable());
        assert!(MemoryState::Dormant.is_retrievable());
        assert!(!MemoryState::Silent.is_retrievable());
        assert!(!MemoryState::Unavailable.is_retrievable());

        assert!(MemoryState::Silent.requires_strong_cue());
        assert!(MemoryState::Unavailable.is_blocked());
    }

    #[test]
    fn test_memory_state_roundtrip() {
        for state in [
            MemoryState::Active,
            MemoryState::Dormant,
            MemoryState::Silent,
            MemoryState::Unavailable,
        ] {
            assert_eq!(MemoryState::from_str(state.as_str()), state);
        }
    }

    // ==================== MemoryLifecycle Tests ====================

    #[test]
    fn test_lifecycle_creation() {
        let lifecycle = MemoryLifecycle::new();
        assert_eq!(lifecycle.state, MemoryState::Active);
        assert_eq!(lifecycle.access_count, 1);
        assert!(lifecycle.state_history.is_empty());
    }

    #[test]
    fn test_lifecycle_access_reactivates() {
        let mut lifecycle = MemoryLifecycle::with_state(MemoryState::Dormant);
        assert_eq!(lifecycle.state, MemoryState::Dormant);

        let changed = lifecycle.record_access();
        assert!(changed);
        assert_eq!(lifecycle.state, MemoryState::Active);
        assert_eq!(lifecycle.access_count, 2);
    }

    #[test]
    fn test_lifecycle_suppression() {
        let mut lifecycle = MemoryLifecycle::new();

        lifecycle.suppress_from_competition("winner123".to_string(), 0.85, Duration::hours(2));

        assert_eq!(lifecycle.state, MemoryState::Unavailable);
        assert!(!lifecycle.is_suppression_expired());
        assert!(lifecycle.suppressed_by.contains(&"winner123".to_string()));

        // Access should not reactivate while suppressed
        let changed = lifecycle.record_access();
        assert!(!changed);
        assert_eq!(lifecycle.state, MemoryState::Unavailable);
    }

    #[test]
    fn test_lifecycle_decay_detection() {
        let mut lifecycle = MemoryLifecycle::new();
        let config = StateDecayConfig {
            active_decay_hours: 0, // Immediate decay
            dormant_decay_days: 0, // Immediate decay
            ..Default::default()
        };

        // Should decay immediately
        assert!(lifecycle.should_decay_to_dormant(&config));

        lifecycle.transition_to(MemoryState::Dormant, StateTransitionReason::TimeDecay);
        assert!(lifecycle.should_decay_to_silent(&config));
    }

    #[test]
    fn test_lifecycle_cue_reactivation() {
        let mut lifecycle = MemoryLifecycle::with_state(MemoryState::Silent);

        // Weak cue should fail
        let reactivated = lifecycle.try_reactivate_with_cue(0.5, 0.8);
        assert!(!reactivated);
        assert_eq!(lifecycle.state, MemoryState::Silent);

        // Strong cue should succeed
        let reactivated = lifecycle.try_reactivate_with_cue(0.9, 0.8);
        assert!(reactivated);
        assert_eq!(lifecycle.state, MemoryState::Dormant);
    }

    #[test]
    fn test_lifecycle_state_history_limit() {
        let mut lifecycle = MemoryLifecycle::new();

        // Add many transitions
        for i in 0..100 {
            lifecycle.transition_to(
                if i % 2 == 0 {
                    MemoryState::Dormant
                } else {
                    MemoryState::Active
                },
                StateTransitionReason::Access,
            );
        }

        // History should be capped
        assert!(lifecycle.state_history.len() <= MAX_STATE_HISTORY_SIZE);
    }

    // ==================== Competition Tests ====================

    #[test]
    fn test_competition_manager() {
        let mut manager = CompetitionManager::new();

        let candidates = vec![
            CompetitionCandidate {
                memory_id: "mem1".to_string(),
                relevance_score: 0.95,
                similarity_to_query: 0.9,
            },
            CompetitionCandidate {
                memory_id: "mem2".to_string(),
                relevance_score: 0.80,
                similarity_to_query: 0.85,
            },
            CompetitionCandidate {
                memory_id: "mem3".to_string(),
                relevance_score: 0.70,
                similarity_to_query: 0.88,
            },
        ];

        let result = manager.run_competition(&candidates, 0.6);
        assert!(result.is_some());

        let result = result.unwrap();
        assert_eq!(result.winner_id, "mem1");
        assert!(result.suppressed_ids.contains(&"mem2".to_string()));
        assert!(result.suppressed_ids.contains(&"mem3".to_string()));
    }

    #[test]
    fn test_competition_no_similar_candidates() {
        let mut manager = CompetitionManager::new();

        let candidates = vec![
            CompetitionCandidate {
                memory_id: "mem1".to_string(),
                relevance_score: 0.95,
                similarity_to_query: 0.9,
            },
            CompetitionCandidate {
                memory_id: "mem2".to_string(),
                relevance_score: 0.80,
                similarity_to_query: 0.2, // Very different
            },
        ];

        // High threshold means no competition
        let result = manager.run_competition(&candidates, 0.9);
        assert!(result.is_none());
    }

    #[test]
    fn test_competition_win_count() {
        let mut manager = CompetitionManager::new();

        // Run two competitions with same winner
        for _ in 0..2 {
            let candidates = vec![
                CompetitionCandidate {
                    memory_id: "winner".to_string(),
                    relevance_score: 0.95,
                    similarity_to_query: 0.9,
                },
                CompetitionCandidate {
                    memory_id: "loser".to_string(),
                    relevance_score: 0.80,
                    similarity_to_query: 0.85,
                },
            ];
            manager.run_competition(&candidates, 0.5);
        }

        assert_eq!(manager.win_count("winner"), 2);
        assert_eq!(manager.suppression_count("loser"), 2);
    }

    // ==================== State Update Service Tests ====================

    #[test]
    fn test_state_update_service() {
        let service = StateUpdateService::with_config(StateDecayConfig {
            active_decay_hours: 0,
            dormant_decay_days: 0,
            auto_resolve_suppression: true,
            ..Default::default()
        });

        let mut lifecycle = MemoryLifecycle::new();
        let transitions = service.update_lifecycle(&mut lifecycle);

        // Should have decayed: Active -> Dormant -> Silent
        assert_eq!(lifecycle.state, MemoryState::Silent);
        assert_eq!(transitions.len(), 2);
    }

    #[test]
    fn test_state_update_resolves_suppression() {
        let service = StateUpdateService::with_config(StateDecayConfig {
            auto_resolve_suppression: true,
            ..Default::default()
        });

        let mut lifecycle = MemoryLifecycle::new();
        lifecycle.transition_to(
            MemoryState::Unavailable,
            StateTransitionReason::CompetitionLoss {
                winner_id: "test".to_string(),
                similarity: 0.8,
            },
        );
        // Set suppression to already expired
        lifecycle.suppression_until = Some(Utc::now() - Duration::hours(1));

        let transitions = service.update_lifecycle(&mut lifecycle);

        assert_eq!(lifecycle.state, MemoryState::Dormant);
        assert_eq!(transitions.len(), 1);
        assert!(matches!(
            transitions[0].reason,
            StateTransitionReason::SuppressionExpired
        ));
    }

    #[test]
    fn test_batch_update() {
        let service = StateUpdateService::with_config(StateDecayConfig {
            active_decay_hours: 0,
            dormant_decay_days: 1000, // Won't decay
            ..Default::default()
        });

        let mut lifecycles = vec![
            MemoryLifecycle::new(),
            MemoryLifecycle::new(),
            MemoryLifecycle::with_state(MemoryState::Dormant),
        ];

        let result = service.batch_update(&mut lifecycles);

        assert_eq!(result.active_to_dormant, 2);
        assert_eq!(result.dormant_to_silent, 0);
        assert_eq!(result.total_transitions, 2);
    }

    // ==================== Accessibility Calculator Tests ====================

    #[test]
    fn test_accessibility_calculator() {
        let calc = AccessibilityCalculator::default();
        let lifecycle = MemoryLifecycle::new();

        // Active memory just accessed should have high accessibility
        let score = calc.calculate(&lifecycle, 0.8);
        assert!(score > 0.8);
        assert!(score <= 1.0);
    }

    #[test]
    fn test_accessibility_state_multipliers() {
        let calc = AccessibilityCalculator {
            recency_weight: 0.0,
            frequency_weight: 0.0,
            ..Default::default()
        };

        let mut lifecycle = MemoryLifecycle::new();
        let base_score = 1.0;

        // Active: full score
        let active_score = calc.calculate(&lifecycle, base_score);
        assert!(approx_eq(active_score, 1.0, 0.01));

        // Dormant: 0.7x
        lifecycle.state = MemoryState::Dormant;
        let dormant_score = calc.calculate(&lifecycle, base_score);
        assert!(approx_eq(dormant_score, 0.7, 0.01));

        // Silent: 0.3x
        lifecycle.state = MemoryState::Silent;
        let silent_score = calc.calculate(&lifecycle, base_score);
        assert!(approx_eq(silent_score, 0.3, 0.01));

        // Unavailable: 0.05x
        lifecycle.state = MemoryState::Unavailable;
        let unavailable_score = calc.calculate(&lifecycle, base_score);
        assert!(approx_eq(unavailable_score, 0.05, 0.01));
    }

    // ==================== State Time Accumulator Tests ====================

    #[test]
    fn test_state_time_accumulator() {
        let mut acc = StateTimeAccumulator::default();

        acc.add(MemoryState::Active, 3600);
        acc.add(MemoryState::Dormant, 7200);

        assert_eq!(acc.active_seconds, 3600);
        assert_eq!(acc.dormant_seconds, 7200);
        assert_eq!(acc.total_seconds(), 10800);

        let pct = acc.percentages();
        assert!(approx_eq(pct.active, 33.33, 0.1));
        assert!(approx_eq(pct.dormant, 66.67, 0.1));
    }

    // ==================== Memory State Info Tests ====================

    #[test]
    fn test_memory_state_info() {
        let lifecycle = MemoryLifecycle::new();
        let info = MemoryStateInfo::from_lifecycle(&lifecycle);

        assert_eq!(info.state, MemoryState::Active);
        assert_eq!(info.accessibility, 1.0);
        assert_eq!(info.access_count, 1);
        assert!(
            info.time_since_access.contains("just now")
                || info.time_since_access.contains("minute")
        );
    }

    #[test]
    fn test_memory_state_info_suppressed() {
        let mut lifecycle = MemoryLifecycle::new();
        lifecycle.suppress_by_user(Duration::hours(2), Some("test reason".to_string()));

        let info = MemoryStateInfo::from_lifecycle(&lifecycle);

        assert_eq!(info.state, MemoryState::Unavailable);
        assert!(info.accessible_after.is_some());
        assert!(!info.recommendations.is_empty());
    }

    // ==================== Serialization Tests ====================

    #[test]
    fn test_memory_state_serialization() {
        let state = MemoryState::Dormant;
        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, "\"dormant\"");

        let parsed: MemoryState = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, state);
    }

    #[test]
    fn test_lifecycle_serialization() {
        let lifecycle = MemoryLifecycle::new();
        let json = serde_json::to_string(&lifecycle).unwrap();
        let parsed: MemoryLifecycle = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.state, lifecycle.state);
        assert_eq!(parsed.access_count, lifecycle.access_count);
    }
}
