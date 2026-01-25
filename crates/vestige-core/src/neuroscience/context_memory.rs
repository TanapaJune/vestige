//! # Context-Dependent Memory - Encoding Specificity Principle
//!
//! Memory retrieval is best when the retrieval context MATCHES the encoding context.
//! This is one of the most robust findings in memory science, established by Tulving
//! and Thomson (1973).
//!
//! ## Scientific Background
//!
//! The Encoding Specificity Principle states that memory is most accessible when
//! the retrieval cues match the encoding conditions. This has been demonstrated
//! across multiple domains:
//!
//! - **State-Dependent Memory**: Information learned in one state (e.g., emotional,
//!   physiological) is better recalled in the same state
//! - **Context-Dependent Memory**: Environmental context during learning affects
//!   subsequent retrieval
//! - **Mood Congruence**: Emotional content is better remembered when current mood
//!   matches the emotion of the content
//!
//! ## Implementation Strategy
//!
//! We capture rich context at encoding time including:
//! - **Temporal Context**: Time of day, day of week, recency
//! - **Topical Context**: Active topics, recent queries, conversation thread
//! - **Session Context**: Session ID, activity type, project
//! - **Emotional Context**: Sentiment polarity and magnitude
//!
//! At retrieval time, we compute context similarity and use it to boost
//! relevance scores for memories encoded in similar contexts.
//!
//! ## Example
//!
//! ```rust,ignore
//! use vestige_core::neuroscience::{
//!     ContextMatcher, EncodingContext, TemporalContext, TopicalContext,
//! };
//!
//! let matcher = ContextMatcher::default();
//!
//! // Compare encoding and retrieval contexts
//! let encoding_ctx = memory.encoding_context();
//! let current_ctx = EncodingContext::capture_current();
//!
//! let similarity = matcher.match_contexts(&encoding_ctx, &current_ctx);
//! println!("Context match: {:.2}", similarity); // 0.0 to 1.0
//!
//! // Boost retrieval scores based on context match
//! let boosted = matcher.boost_retrieval(memories, &current_ctx);
//! ```
//!
//! ## References
//!
//! - Tulving, E., & Thomson, D. M. (1973). Encoding specificity and retrieval
//!   processes in episodic memory. Psychological Review, 80(5), 352-373.
//! - Godden, D. R., & Baddeley, A. D. (1975). Context-dependent memory in two
//!   natural environments: On land and underwater. British Journal of Psychology.

use std::collections::HashSet;

use chrono::{DateTime, Datelike, Duration, Timelike, Utc, Weekday};
use serde::{Deserialize, Serialize};

// ============================================================================
// TIME OF DAY
// ============================================================================

/// Time of day categories for temporal context matching
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeOfDay {
    /// 5:00 AM - 11:59 AM
    Morning,
    /// 12:00 PM - 4:59 PM
    Afternoon,
    /// 5:00 PM - 8:59 PM
    Evening,
    /// 9:00 PM - 4:59 AM
    Night,
}

impl TimeOfDay {
    /// Determine time of day from a timestamp
    pub fn from_datetime(dt: DateTime<Utc>) -> Self {
        let hour = dt.hour();
        match hour {
            5..=11 => Self::Morning,
            12..=16 => Self::Afternoon,
            17..=20 => Self::Evening,
            _ => Self::Night,
        }
    }

    /// Get the current time of day
    pub fn now() -> Self {
        Self::from_datetime(Utc::now())
    }

    /// Check if two time-of-day values are adjacent (within one period)
    pub fn is_adjacent(&self, other: &Self) -> bool {
        use TimeOfDay::*;
        matches!(
            (self, other),
            (Morning, Afternoon)
                | (Afternoon, Morning)
                | (Afternoon, Evening)
                | (Evening, Afternoon)
                | (Evening, Night)
                | (Night, Evening)
                | (Night, Morning)
                | (Morning, Night)
        )
    }

    /// Human-readable name
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Morning => "morning",
            Self::Afternoon => "afternoon",
            Self::Evening => "evening",
            Self::Night => "night",
        }
    }
}

// ============================================================================
// RECENCY BUCKET
// ============================================================================

/// Recency categories for temporal context matching
///
/// Based on memory research showing that temporal context decays over time
/// but in discrete "chunks" rather than continuously.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum RecencyBucket {
    /// Within the last hour
    VeryRecent,
    /// Within the last day (1-24 hours)
    Today,
    /// Within the last week (1-7 days)
    ThisWeek,
    /// Within the last month (1-4 weeks)
    ThisMonth,
    /// Within the last quarter (1-3 months)
    ThisQuarter,
    /// Within the last year (3-12 months)
    ThisYear,
    /// Older than a year
    Older,
}

impl RecencyBucket {
    /// Determine recency bucket from a timestamp
    pub fn from_datetime(dt: DateTime<Utc>) -> Self {
        let now = Utc::now();
        let age = now.signed_duration_since(dt);

        if age < Duration::hours(1) {
            Self::VeryRecent
        } else if age < Duration::hours(24) {
            Self::Today
        } else if age < Duration::days(7) {
            Self::ThisWeek
        } else if age < Duration::days(30) {
            Self::ThisMonth
        } else if age < Duration::days(90) {
            Self::ThisQuarter
        } else if age < Duration::days(365) {
            Self::ThisYear
        } else {
            Self::Older
        }
    }

    /// Check if two recency buckets are within one step of each other
    pub fn is_adjacent(&self, other: &Self) -> bool {
        let self_ord = *self as i32;
        let other_ord = *other as i32;
        (self_ord - other_ord).abs() <= 1
    }

    /// Human-readable description
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::VeryRecent => "very recent (< 1 hour)",
            Self::Today => "today",
            Self::ThisWeek => "this week",
            Self::ThisMonth => "this month",
            Self::ThisQuarter => "this quarter",
            Self::ThisYear => "this year",
            Self::Older => "older than a year",
        }
    }
}

// ============================================================================
// TEMPORAL CONTEXT
// ============================================================================

/// Temporal context captures WHEN a memory was encoded
///
/// Research shows that temporal context is a powerful retrieval cue.
/// Memories encoded at the same time of day, day of week, or in the
/// same temporal "neighborhood" are more likely to be recalled together.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemporalContext {
    /// Exact timestamp of encoding
    pub timestamp: DateTime<Utc>,
    /// Categorized time of day
    pub time_of_day: TimeOfDay,
    /// Day of the week
    pub day_of_week: Weekday,
    /// Recency bucket (computed dynamically at retrieval)
    pub recency_bucket: RecencyBucket,
}

impl TemporalContext {
    /// Create a new temporal context from a timestamp
    pub fn new(timestamp: DateTime<Utc>) -> Self {
        Self {
            timestamp,
            time_of_day: TimeOfDay::from_datetime(timestamp),
            day_of_week: timestamp.weekday(),
            recency_bucket: RecencyBucket::from_datetime(timestamp),
        }
    }

    /// Capture the current temporal context
    pub fn now() -> Self {
        Self::new(Utc::now())
    }

    /// Update the recency bucket (should be called at retrieval time)
    pub fn refresh_recency(&mut self) {
        self.recency_bucket = RecencyBucket::from_datetime(self.timestamp);
    }

    /// Check if this is a weekday
    pub fn is_weekday(&self) -> bool {
        !matches!(self.day_of_week, Weekday::Sat | Weekday::Sun)
    }

    /// Check if this is a weekend
    pub fn is_weekend(&self) -> bool {
        matches!(self.day_of_week, Weekday::Sat | Weekday::Sun)
    }
}

impl Default for TemporalContext {
    fn default() -> Self {
        Self::now()
    }
}

// ============================================================================
// TOPICAL CONTEXT
// ============================================================================

/// Topical context captures WHAT topics were active during encoding
///
/// This is the cognitive context - what the user was thinking about,
/// what topics were being discussed, and what the recent query history was.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopicalContext {
    /// Currently active topics (extracted from recent interactions)
    pub active_topics: Vec<String>,
    /// Recent queries (for query-based context matching)
    pub recent_queries: Vec<String>,
    /// Current conversation thread ID (if applicable)
    pub conversation_thread: Option<String>,
    /// Keywords extracted from the current context
    pub keywords: Vec<String>,
    /// Tags that were active at encoding time
    pub active_tags: Vec<String>,
}

impl TopicalContext {
    /// Create a new topical context
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with active topics
    pub fn with_topics(topics: Vec<String>) -> Self {
        Self {
            active_topics: topics,
            ..Default::default()
        }
    }

    /// Add a topic to the context
    pub fn add_topic(&mut self, topic: impl Into<String>) {
        let topic = topic.into();
        if !self.active_topics.contains(&topic) {
            self.active_topics.push(topic);
        }
    }

    /// Add a recent query
    pub fn add_query(&mut self, query: impl Into<String>) {
        self.recent_queries.push(query.into());
        // Keep only the last 10 queries
        if self.recent_queries.len() > 10 {
            self.recent_queries.remove(0);
        }
    }

    /// Set the conversation thread
    pub fn set_thread(&mut self, thread_id: impl Into<String>) {
        self.conversation_thread = Some(thread_id.into());
    }

    /// Extract keywords from text and add them
    pub fn extract_keywords_from(&mut self, text: &str) {
        // Simple keyword extraction (in production, use NLP)
        let words: Vec<String> = text
            .split_whitespace()
            .filter(|w| w.len() > 3)
            .map(|w| w.to_lowercase())
            .filter(|w| !is_stop_word(w))
            .collect();

        for word in words {
            if !self.keywords.contains(&word) {
                self.keywords.push(word);
            }
        }

        // Keep only top 20 keywords
        self.keywords.truncate(20);
    }

    /// Get all context terms (topics + keywords + tags)
    pub fn all_terms(&self) -> HashSet<String> {
        let mut terms = HashSet::new();
        terms.extend(self.active_topics.iter().cloned());
        terms.extend(self.keywords.iter().cloned());
        terms.extend(self.active_tags.iter().cloned());
        terms
    }
}

/// Simple stop word check (expand for production)
fn is_stop_word(word: &str) -> bool {
    const STOP_WORDS: &[&str] = &[
        "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by",
        "from", "as", "is", "was", "are", "were", "been", "be", "have", "has", "had", "do", "does",
        "did", "will", "would", "could", "should", "may", "might", "must", "shall", "can", "this",
        "that", "these", "those", "it", "its", "they", "them", "their", "we", "our", "you", "your",
        "he", "she", "his", "her", "what", "which", "who", "whom", "when", "where", "why", "how",
    ];
    STOP_WORDS.contains(&word)
}

// ============================================================================
// SESSION CONTEXT
// ============================================================================

/// Session context captures the SESSION in which encoding occurred
///
/// This helps distinguish memories from different work sessions,
/// even if they occurred on the same day or with similar topics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionContext {
    /// Unique session identifier
    pub session_id: Option<String>,
    /// Type of activity (coding, research, debugging, etc.)
    pub activity_type: Option<String>,
    /// Current project or workspace
    pub project: Option<String>,
    /// Current file or document being worked on
    pub active_file: Option<String>,
    /// Git branch (for code-related sessions)
    pub git_branch: Option<String>,
    /// Duration of the session so far (in minutes)
    pub session_duration_minutes: Option<u32>,
}

impl SessionContext {
    /// Create a new session context
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with a session ID
    pub fn with_id(id: impl Into<String>) -> Self {
        Self {
            session_id: Some(id.into()),
            ..Default::default()
        }
    }

    /// Set the activity type
    pub fn set_activity(&mut self, activity: impl Into<String>) {
        self.activity_type = Some(activity.into());
    }

    /// Set the project
    pub fn set_project(&mut self, project: impl Into<String>) {
        self.project = Some(project.into());
    }

    /// Set the active file
    pub fn set_active_file(&mut self, file: impl Into<String>) {
        self.active_file = Some(file.into());
    }

    /// Set the git branch
    pub fn set_branch(&mut self, branch: impl Into<String>) {
        self.git_branch = Some(branch.into());
    }
}

// ============================================================================
// EMOTIONAL CONTEXT
// ============================================================================

/// Emotional context captures the emotional state during encoding
///
/// Based on mood-congruent memory research, emotional context
/// significantly affects memory encoding and retrieval.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmotionalContext {
    /// Emotional valence (-1.0 = negative, 0.0 = neutral, 1.0 = positive)
    pub valence: f64,
    /// Arousal level (0.0 = calm, 1.0 = excited/agitated)
    pub arousal: f64,
    /// Dominance (0.0 = submissive, 1.0 = dominant/in control)
    pub dominance: f64,
    /// Primary emotion label (optional)
    pub primary_emotion: Option<String>,
    /// Confidence in the emotional assessment (0.0 to 1.0)
    pub confidence: f64,
}

impl EmotionalContext {
    /// Create a neutral emotional context
    pub fn neutral() -> Self {
        Self {
            valence: 0.0,
            arousal: 0.5,
            dominance: 0.5,
            primary_emotion: None,
            confidence: 0.5,
        }
    }

    /// Create from sentiment scores (maps to valence)
    pub fn from_sentiment(score: f64, magnitude: f64) -> Self {
        Self {
            valence: score,
            arousal: magnitude,
            dominance: 0.5,
            primary_emotion: Self::infer_emotion(score, magnitude),
            confidence: magnitude.min(1.0),
        }
    }

    /// Infer primary emotion from valence and arousal
    fn infer_emotion(valence: f64, arousal: f64) -> Option<String> {
        let emotion = match (valence > 0.3, valence < -0.3, arousal > 0.6) {
            (true, false, true) => "excited",
            (true, false, false) => "content",
            (false, true, true) => "angry",
            (false, true, false) => "sad",
            (false, false, true) => "anxious",
            (false, false, false) => "neutral",
            // Edge case: both conditions true (shouldn't happen with proper thresholds)
            (true, true, _) => "conflicted",
        };
        Some(emotion.to_string())
    }

    /// Check if this is a positive emotional state
    pub fn is_positive(&self) -> bool {
        self.valence > 0.2
    }

    /// Check if this is a negative emotional state
    pub fn is_negative(&self) -> bool {
        self.valence < -0.2
    }

    /// Check if this is a high-arousal state
    pub fn is_high_arousal(&self) -> bool {
        self.arousal > 0.6
    }
}

// ============================================================================
// ENCODING CONTEXT (COMBINED)
// ============================================================================

/// Complete encoding context capturing all dimensions
///
/// This is the full context snapshot taken when a memory is encoded.
/// It combines temporal, topical, session, and emotional contexts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncodingContext {
    /// When the memory was encoded
    pub temporal: TemporalContext,
    /// What topics were active
    pub topical: TopicalContext,
    /// What session/activity was occurring
    pub session: SessionContext,
    /// Emotional state during encoding
    pub emotional: EmotionalContext,
}

impl EncodingContext {
    /// Create a new encoding context with current temporal context
    pub fn new() -> Self {
        Self {
            temporal: TemporalContext::now(),
            topical: TopicalContext::default(),
            session: SessionContext::default(),
            emotional: EmotionalContext::neutral(),
        }
    }

    /// Capture the current context (minimal version)
    pub fn capture_current() -> Self {
        Self::new()
    }

    /// Create with all components
    pub fn with_all(
        temporal: TemporalContext,
        topical: TopicalContext,
        session: SessionContext,
        emotional: EmotionalContext,
    ) -> Self {
        Self {
            temporal,
            topical,
            session,
            emotional,
        }
    }

    /// Builder: set temporal context
    pub fn with_temporal(mut self, temporal: TemporalContext) -> Self {
        self.temporal = temporal;
        self
    }

    /// Builder: set topical context
    pub fn with_topical(mut self, topical: TopicalContext) -> Self {
        self.topical = topical;
        self
    }

    /// Builder: set session context
    pub fn with_session(mut self, session: SessionContext) -> Self {
        self.session = session;
        self
    }

    /// Builder: set emotional context
    pub fn with_emotional(mut self, emotional: EmotionalContext) -> Self {
        self.emotional = emotional;
        self
    }

    /// Add a topic to the topical context
    pub fn add_topic(&mut self, topic: impl Into<String>) {
        self.topical.add_topic(topic);
    }

    /// Set the project in session context
    pub fn set_project(&mut self, project: impl Into<String>) {
        self.session.set_project(project);
    }

    /// Refresh dynamic fields (like recency)
    pub fn refresh(&mut self) {
        self.temporal.refresh_recency();
    }
}

impl Default for EncodingContext {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// CONTEXT WEIGHTS
// ============================================================================

/// Weights for different context dimensions in matching
///
/// These can be tuned based on the application domain or user preferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextWeights {
    /// Weight for temporal context match (0.0 to 1.0)
    pub temporal: f64,
    /// Weight for topical context match (0.0 to 1.0)
    pub topical: f64,
    /// Weight for session context match (0.0 to 1.0)
    pub session: f64,
    /// Weight for emotional context match (0.0 to 1.0)
    pub emotional: f64,
}

impl Default for ContextWeights {
    fn default() -> Self {
        Self {
            temporal: 0.2,   // Moderate weight for time
            topical: 0.4,    // Highest weight for topic match
            session: 0.25,   // Good weight for same session/project
            emotional: 0.15, // Lower weight for emotional match
        }
    }
}

impl ContextWeights {
    /// Create weights emphasizing topical match
    pub fn topic_focused() -> Self {
        Self {
            temporal: 0.1,
            topical: 0.6,
            session: 0.2,
            emotional: 0.1,
        }
    }

    /// Create weights emphasizing temporal match
    pub fn recency_focused() -> Self {
        Self {
            temporal: 0.4,
            topical: 0.3,
            session: 0.2,
            emotional: 0.1,
        }
    }

    /// Create weights emphasizing session match
    pub fn session_focused() -> Self {
        Self {
            temporal: 0.15,
            topical: 0.3,
            session: 0.45,
            emotional: 0.1,
        }
    }

    /// Normalize weights to sum to 1.0
    pub fn normalize(&mut self) {
        let sum = self.temporal + self.topical + self.session + self.emotional;
        if sum > 0.0 {
            self.temporal /= sum;
            self.topical /= sum;
            self.session /= sum;
            self.emotional /= sum;
        }
    }
}

// ============================================================================
// CONTEXT REINSTATEMENT
// ============================================================================

/// Hints for context reinstatement during retrieval
///
/// When a memory is retrieved, these hints help the user remember
/// the original context ("You were discussing X when this came up").
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextReinstatement {
    /// Memory ID this reinstatement is for
    pub memory_id: String,
    /// Temporal hint ("This was from last Tuesday morning")
    pub temporal_hint: Option<String>,
    /// Topical hint ("You were discussing authentication")
    pub topical_hint: Option<String>,
    /// Session hint ("This was during your work on the API refactor")
    pub session_hint: Option<String>,
    /// Related memories from the same context
    pub related_memories: Vec<String>,
}

impl ContextReinstatement {
    /// Create an empty reinstatement
    pub fn new(memory_id: impl Into<String>) -> Self {
        Self {
            memory_id: memory_id.into(),
            temporal_hint: None,
            topical_hint: None,
            session_hint: None,
            related_memories: vec![],
        }
    }

    /// Generate reinstatement hints from an encoding context
    pub fn from_context(memory_id: impl Into<String>, context: &EncodingContext) -> Self {
        let mut reinstatement = Self::new(memory_id);

        // Generate temporal hint
        let recency = context.temporal.recency_bucket.as_str();
        let time_of_day = context.temporal.time_of_day.as_str();
        let day = format!("{:?}", context.temporal.day_of_week);
        reinstatement.temporal_hint = Some(format!(
            "This memory is from {} ({} on {})",
            recency, time_of_day, day
        ));

        // Generate topical hint
        if !context.topical.active_topics.is_empty() {
            let topics = context.topical.active_topics.join(", ");
            reinstatement.topical_hint = Some(format!("You were discussing: {}", topics));
        }

        // Generate session hint
        if let Some(ref project) = context.session.project {
            reinstatement.session_hint = Some(format!("This was during work on '{}'", project));
        } else if let Some(ref activity) = context.session.activity_type {
            reinstatement.session_hint = Some(format!("This was during {}", activity));
        }

        reinstatement
    }

    /// Check if any hints are available
    pub fn has_hints(&self) -> bool {
        self.temporal_hint.is_some() || self.topical_hint.is_some() || self.session_hint.is_some()
    }

    /// Get a combined hint string
    pub fn combined_hint(&self) -> Option<String> {
        let mut hints = Vec::new();

        if let Some(ref hint) = self.topical_hint {
            hints.push(hint.clone());
        }
        if let Some(ref hint) = self.session_hint {
            hints.push(hint.clone());
        }
        if let Some(ref hint) = self.temporal_hint {
            hints.push(hint.clone());
        }

        if hints.is_empty() {
            None
        } else {
            Some(hints.join(". "))
        }
    }
}

// ============================================================================
// SCORED MEMORY
// ============================================================================

/// A memory with its context match score
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoredMemory<T> {
    /// The memory item
    pub memory: T,
    /// Original relevance score (from search)
    pub relevance_score: f64,
    /// Context match score (0.0 to 1.0)
    pub context_score: f64,
    /// Final combined score
    pub combined_score: f64,
    /// Context reinstatement hints
    pub reinstatement: Option<ContextReinstatement>,
}

impl<T> ScoredMemory<T> {
    /// Create a new scored memory
    pub fn new(memory: T, relevance_score: f64, context_score: f64) -> Self {
        let combined_score = Self::compute_combined(relevance_score, context_score);
        Self {
            memory,
            relevance_score,
            context_score,
            combined_score,
            reinstatement: None,
        }
    }

    /// Compute combined score (can be customized)
    fn compute_combined(relevance: f64, context: f64) -> f64 {
        // Context provides up to 30% boost to relevance
        relevance * (1.0 + 0.3 * context)
    }

    /// Add reinstatement hints
    pub fn with_reinstatement(mut self, reinstatement: ContextReinstatement) -> Self {
        self.reinstatement = Some(reinstatement);
        self
    }
}

// ============================================================================
// CONTEXT MATCHER
// ============================================================================

/// Matches encoding and retrieval contexts to compute similarity
///
/// This is the core component that implements the Encoding Specificity Principle.
#[derive(Debug, Clone)]
pub struct ContextMatcher {
    /// Weights for different context dimensions
    pub weights: ContextWeights,
}

impl Default for ContextMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextMatcher {
    /// Create a new context matcher with default weights
    pub fn new() -> Self {
        Self {
            weights: ContextWeights::default(),
        }
    }

    /// Create with custom weights
    pub fn with_weights(weights: ContextWeights) -> Self {
        Self { weights }
    }

    /// Compute similarity between encoding and retrieval contexts
    ///
    /// Returns a score from 0.0 (no match) to 1.0 (perfect match).
    pub fn match_contexts(&self, encoding: &EncodingContext, retrieval: &EncodingContext) -> f64 {
        let temporal_match = self.match_temporal(&encoding.temporal, &retrieval.temporal);
        let topical_match = self.match_topical(&encoding.topical, &retrieval.topical);
        let session_match = self.match_session(&encoding.session, &retrieval.session);
        let emotional_match = self.match_emotional(&encoding.emotional, &retrieval.emotional);

        // Weighted combination
        temporal_match * self.weights.temporal
            + topical_match * self.weights.topical
            + session_match * self.weights.session
            + emotional_match * self.weights.emotional
    }

    /// Match temporal contexts
    fn match_temporal(&self, encoding: &TemporalContext, retrieval: &TemporalContext) -> f64 {
        let mut score = 0.0;

        // Time of day match (0.3 weight)
        if encoding.time_of_day == retrieval.time_of_day {
            score += 0.3;
        } else if encoding.time_of_day.is_adjacent(&retrieval.time_of_day) {
            score += 0.15;
        }

        // Day of week match (0.2 weight)
        if encoding.day_of_week == retrieval.day_of_week {
            score += 0.2;
        } else if encoding.is_weekday() == retrieval.is_weekday() {
            score += 0.1;
        }

        // Recency match (0.5 weight) - most important temporal factor
        if encoding.recency_bucket == retrieval.recency_bucket {
            score += 0.5;
        } else if encoding
            .recency_bucket
            .is_adjacent(&retrieval.recency_bucket)
        {
            score += 0.25;
        }

        score
    }

    /// Match topical contexts
    fn match_topical(&self, encoding: &TopicalContext, retrieval: &TopicalContext) -> f64 {
        let encoding_terms = encoding.all_terms();
        let retrieval_terms = retrieval.all_terms();

        // If both are empty, they're identical (perfect match)
        if encoding_terms.is_empty() && retrieval_terms.is_empty() {
            return 1.0;
        }

        // If only one is empty, no match
        if encoding_terms.is_empty() || retrieval_terms.is_empty() {
            return 0.0;
        }

        // Jaccard similarity
        let intersection = encoding_terms.intersection(&retrieval_terms).count();
        let union = encoding_terms.union(&retrieval_terms).count();

        if union == 0 {
            0.0
        } else {
            (intersection as f64 / union as f64).min(1.0)
        }
    }

    /// Match session contexts
    fn match_session(&self, encoding: &SessionContext, retrieval: &SessionContext) -> f64 {
        let mut score = 0.0;

        // Same session is a very strong match
        if let (Some(e_id), Some(r_id)) = (&encoding.session_id, &retrieval.session_id) {
            if e_id == r_id {
                return 1.0;
            }
        }

        // Project match (0.4 weight)
        if let (Some(e_proj), Some(r_proj)) = (&encoding.project, &retrieval.project) {
            if e_proj == r_proj {
                score += 0.4;
            }
        }

        // Activity type match (0.3 weight)
        if let (Some(e_act), Some(r_act)) = (&encoding.activity_type, &retrieval.activity_type) {
            if e_act == r_act {
                score += 0.3;
            }
        }

        // Git branch match (0.2 weight)
        if let (Some(e_br), Some(r_br)) = (&encoding.git_branch, &retrieval.git_branch) {
            if e_br == r_br {
                score += 0.2;
            }
        }

        // Active file match (0.1 weight)
        if let (Some(e_file), Some(r_file)) = (&encoding.active_file, &retrieval.active_file) {
            if e_file == r_file {
                score += 0.1;
            }
        }

        score
    }

    /// Match emotional contexts
    fn match_emotional(&self, encoding: &EmotionalContext, retrieval: &EmotionalContext) -> f64 {
        // Emotional match based on VAD (Valence-Arousal-Dominance) distance
        let valence_diff = (encoding.valence - retrieval.valence).abs();
        let arousal_diff = (encoding.arousal - retrieval.arousal).abs();
        let dominance_diff = (encoding.dominance - retrieval.dominance).abs();

        // Convert distances to similarity (max distance is 2.0 per dimension)
        let valence_sim = 1.0 - (valence_diff / 2.0);
        let arousal_sim = 1.0 - arousal_diff;
        let dominance_sim = 1.0 - dominance_diff;

        // Weighted average (valence is most important for mood-congruence)
        valence_sim * 0.5 + arousal_sim * 0.3 + dominance_sim * 0.2
    }

    /// Boost retrieval results based on context match
    ///
    /// Takes a vector of memories with their encoding contexts and the current
    /// retrieval context, returns memories with boosted scores.
    pub fn boost_retrieval<T, F>(
        &self,
        memories: Vec<T>,
        current_context: &EncodingContext,
        get_context: F,
        get_relevance: impl Fn(&T) -> f64,
    ) -> Vec<ScoredMemory<T>>
    where
        F: Fn(&T) -> Option<&EncodingContext>,
    {
        let mut scored: Vec<ScoredMemory<T>> = memories
            .into_iter()
            .map(|memory| {
                let relevance = get_relevance(&memory);
                let context_score = get_context(&memory)
                    .map(|ctx| self.match_contexts(ctx, current_context))
                    .unwrap_or(0.0);

                ScoredMemory::new(memory, relevance, context_score)
            })
            .collect();

        // Sort by combined score (descending)
        scored.sort_by(|a, b| b.combined_score.partial_cmp(&a.combined_score).unwrap_or(std::cmp::Ordering::Equal));

        scored
    }

    /// Generate context reinstatement hints for a memory
    pub fn reinstate_context(
        &self,
        memory_id: &str,
        context: &EncodingContext,
    ) -> ContextReinstatement {
        ContextReinstatement::from_context(memory_id, context)
    }

    /// Disambiguate same content in different contexts
    ///
    /// When the same content appears in multiple memories with different contexts,
    /// this function helps identify which one is most relevant to the current context.
    pub fn disambiguate<T, F>(
        &self,
        memories: &[T],
        current_context: &EncodingContext,
        get_context: F,
    ) -> Vec<(usize, f64)>
    where
        F: Fn(&T) -> Option<&EncodingContext>,
    {
        let mut scores: Vec<(usize, f64)> = memories
            .iter()
            .enumerate()
            .map(|(idx, memory)| {
                let score = get_context(memory)
                    .map(|ctx| self.match_contexts(ctx, current_context))
                    .unwrap_or(0.0);
                (idx, score)
            })
            .collect();

        // Sort by context match (descending)
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        scores
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_of_day() {
        // Test morning
        assert_eq!(
            TimeOfDay::from_datetime(Utc::now().with_hour(8).unwrap()),
            TimeOfDay::Morning
        );

        // Test afternoon
        assert_eq!(
            TimeOfDay::from_datetime(Utc::now().with_hour(14).unwrap()),
            TimeOfDay::Afternoon
        );

        // Test evening
        assert_eq!(
            TimeOfDay::from_datetime(Utc::now().with_hour(18).unwrap()),
            TimeOfDay::Evening
        );

        // Test night
        assert_eq!(
            TimeOfDay::from_datetime(Utc::now().with_hour(23).unwrap()),
            TimeOfDay::Night
        );

        // Test adjacency
        assert!(TimeOfDay::Morning.is_adjacent(&TimeOfDay::Afternoon));
        assert!(!TimeOfDay::Morning.is_adjacent(&TimeOfDay::Evening));
    }

    #[test]
    fn test_recency_bucket() {
        let now = Utc::now();

        // Test very recent
        assert_eq!(
            RecencyBucket::from_datetime(now - Duration::minutes(30)),
            RecencyBucket::VeryRecent
        );

        // Test today
        assert_eq!(
            RecencyBucket::from_datetime(now - Duration::hours(5)),
            RecencyBucket::Today
        );

        // Test this week
        assert_eq!(
            RecencyBucket::from_datetime(now - Duration::days(3)),
            RecencyBucket::ThisWeek
        );

        // Test adjacency
        assert!(RecencyBucket::VeryRecent.is_adjacent(&RecencyBucket::Today));
        assert!(!RecencyBucket::VeryRecent.is_adjacent(&RecencyBucket::ThisMonth));
    }

    #[test]
    fn test_topical_context() {
        let mut topical = TopicalContext::new();
        topical.add_topic("authentication");
        topical.add_topic("security");
        topical.extract_keywords_from("implementing OAuth2 authentication flow");

        assert!(topical
            .active_topics
            .contains(&"authentication".to_string()));
        assert!(topical.keywords.contains(&"oauth2".to_string()));

        let terms = topical.all_terms();
        assert!(terms.contains("authentication"));
    }

    #[test]
    fn test_encoding_context() {
        let mut ctx = EncodingContext::new();
        ctx.add_topic("api-design");
        ctx.set_project("vestige");

        assert!(ctx
            .topical
            .active_topics
            .contains(&"api-design".to_string()));
        assert_eq!(ctx.session.project, Some("vestige".to_string()));
    }

    #[test]
    fn test_context_matcher_same_context() {
        let matcher = ContextMatcher::new();

        // Create contexts with actual content to match
        let mut ctx1 = EncodingContext::new();
        ctx1.add_topic("authentication");
        ctx1.session.project = Some("test-project".to_string());

        let ctx2 = ctx1.clone();

        let similarity = matcher.match_contexts(&ctx1, &ctx2);
        assert!(similarity > 0.8, "Same context should have high similarity, got {}", similarity);
    }

    #[test]
    fn test_context_matcher_different_topics() {
        let matcher = ContextMatcher::new();

        let mut ctx1 = EncodingContext::new();
        ctx1.add_topic("authentication");
        ctx1.add_topic("security");

        let mut ctx2 = EncodingContext::new();
        ctx2.add_topic("database");
        ctx2.add_topic("performance");

        let similarity = matcher.match_contexts(&ctx1, &ctx2);
        assert!(
            similarity < 0.5,
            "Different topics should have low similarity"
        );
    }

    #[test]
    fn test_context_reinstatement() {
        let mut ctx = EncodingContext::new();
        ctx.topical.active_topics = vec!["authentication".to_string()];
        ctx.session.project = Some("vestige".to_string());

        let reinstatement = ContextReinstatement::from_context("mem-123", &ctx);

        assert!(reinstatement.has_hints());
        assert!(reinstatement.topical_hint.is_some());
        assert!(reinstatement.session_hint.is_some());

        let hint = reinstatement.combined_hint().unwrap();
        assert!(hint.contains("authentication"));
        assert!(hint.contains("vestige"));
    }

    #[test]
    fn test_emotional_context() {
        let positive = EmotionalContext::from_sentiment(0.7, 0.8);
        assert!(positive.is_positive());
        assert!(positive.is_high_arousal());

        let negative = EmotionalContext::from_sentiment(-0.5, 0.3);
        assert!(negative.is_negative());
        assert!(!negative.is_high_arousal());
    }

    #[test]
    fn test_context_weights_normalization() {
        let mut weights = ContextWeights {
            temporal: 1.0,
            topical: 2.0,
            session: 1.0,
            emotional: 0.0,
        };
        weights.normalize();

        let sum = weights.temporal + weights.topical + weights.session + weights.emotional;
        assert!((sum - 1.0).abs() < 0.001, "Weights should sum to 1.0");
    }
}
