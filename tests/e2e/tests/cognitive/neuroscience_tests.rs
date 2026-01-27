//! # Neuroscience Validation E2E Tests
//!
//! Comprehensive tests validating Vestige's neuroscience-inspired memory features.
//!
//! ## Test Categories
//!
//! 1. **Synaptic Tagging and Capture (STC)** - 10 tests
//!    Based on Redondo & Morris (2011): memories can become important RETROACTIVELY
//!
//! 2. **Memory Reconsolidation** - 5 tests
//!    Based on Nader (2000): memories become modifiable when retrieved
//!
//! 3. **FSRS-6 Forgetting Curves** - 8 tests
//!    Based on FSRS-6 algorithm: power forgetting curve with personalization
//!
//! 4. **Memory States** - 7 tests
//!    Based on Bjork (1992): memories exist in different accessibility states
//!
//! 5. **Multi-Channel Importance** - 5 tests
//!    Based on neuromodulator systems: dopamine, norepinephrine, acetylcholine

use chrono::{Duration, Utc};
use vestige_core::{
    // Advanced reconsolidation
    AccessContext, AccessTrigger, Modification, ReconsolidationManager, RelationshipType,
    // FSRS
    Rating, retrievability, retrievability_with_decay, initial_difficulty,
    next_interval, FSRSScheduler,
    // Neuroscience - Synaptic Tagging
    SynapticTaggingSystem, ImportanceEvent, ImportanceEventType,
    CaptureWindow, DecayFunction,
    // Neuroscience - Memory States
    MemoryState, MemoryLifecycle, AccessibilityCalculator,
    CompetitionManager, CompetitionCandidate,
    // Neuroscience - Importance Signals
    ImportanceSignals, NoveltySignal, ArousalSignal, RewardSignal, AttentionSignal,
    ImportanceContext, AttentionSession, OutcomeType,
};

// ============================================================================
// SYNAPTIC TAGGING AND CAPTURE (STC) TESTS - 10 tests
// ============================================================================
// Based on Redondo & Morris (2011): Synaptic tagging allows memories to be
// consolidated retroactively when a later important event occurs.

/// Test that synaptic tags are created correctly.
///
/// When a memory is encoded, it should receive a synaptic tag that marks
/// it as eligible for later consolidation.
#[test]
fn test_stc_tag_creation() {
    let mut stc = SynapticTaggingSystem::new();

    let tag = stc.tag_memory("mem-123");

    assert_eq!(tag.memory_id, "mem-123");
    assert_eq!(tag.initial_strength, 1.0);
    assert!(!tag.captured);
    assert!(tag.capture_event.is_none());
    assert!(stc.has_active_tag("mem-123"));
}

/// Test that tags with custom strength are created correctly.
///
/// Some memories may have initial importance signals (e.g., emotional content)
/// that warrant a higher initial tag strength.
#[test]
fn test_stc_tag_with_custom_strength() {
    let mut stc = SynapticTaggingSystem::new();

    let tag = stc.tag_memory_with_strength("mem-456", 0.7);

    assert_eq!(tag.initial_strength, 0.7);
    assert_eq!(tag.tag_strength, 0.7);
}

/// Test that importance events trigger PRP production and capture.
///
/// When a strong importance event occurs (e.g., user flags something as important),
/// PRPs are produced and can capture nearby tagged memories.
#[test]
fn test_stc_prp_trigger_captures_memories() {
    let mut stc = SynapticTaggingSystem::new();

    // Tag a memory
    stc.tag_memory("mem-background");

    // Later, trigger an importance event
    let event = ImportanceEvent::user_flag("mem-trigger", Some("Remember this!"));
    let result = stc.trigger_prp(event);

    // The tagged memory should be captured
    assert!(result.has_captures());
    assert!(result.captured_memories.iter().any(|c| c.memory_id == "mem-background"));
    assert!(stc.is_captured("mem-background"));
}

/// Test that weak importance events don't trigger capture.
///
/// Events below the PRP threshold should not produce PRPs.
#[test]
fn test_stc_weak_event_no_capture() {
    let mut stc = SynapticTaggingSystem::new();
    stc.tag_memory("mem-123");

    // Very weak event - below default 0.7 threshold
    let event = ImportanceEvent::with_strength(ImportanceEventType::TemporalProximity, 0.3);
    let result = stc.trigger_prp(event);

    assert!(!result.has_captures());
    assert!(!stc.is_captured("mem-123"));
}

/// Test different event types have different base strengths.
///
/// UserFlag has highest strength (explicit user intent), while
/// TemporalProximity has lower strength (indirect signal).
#[test]
fn test_stc_event_type_strengths() {
    assert_eq!(ImportanceEventType::UserFlag.base_strength(), 1.0);
    assert!(ImportanceEventType::NoveltySpike.base_strength() > 0.8);
    assert!(ImportanceEventType::EmotionalContent.base_strength() > 0.7);
    assert!(ImportanceEventType::TemporalProximity.base_strength() < 0.6);

    // User flag should be stronger than all other types
    let user_flag = ImportanceEventType::UserFlag.base_strength();
    assert!(user_flag > ImportanceEventType::NoveltySpike.base_strength());
    assert!(user_flag > ImportanceEventType::EmotionalContent.base_strength());
    assert!(user_flag > ImportanceEventType::RepeatedAccess.base_strength());
}

/// Test capture window probability calculation.
///
/// Memories closer to the importance event have higher capture probability.
/// Based on the neuroscience finding that STC works even with 9-hour intervals.
#[test]
fn test_stc_capture_window_probability() {
    let window = CaptureWindow::new(9.0, 2.0); // 9h backward, 2h forward
    let event_time = Utc::now();

    // Memory just before event - high probability (exponential decay with λ=4.605/9)
    let recent_before = event_time - Duration::hours(1);
    let prob_recent = window.capture_probability(recent_before, event_time).unwrap();
    // At 1h out of 9h with exponential decay: e^(-4.605/9 * 1) ≈ 0.6
    assert!(prob_recent > 0.5, "Recent memory should have high capture probability");

    // Memory 6 hours before event - moderate probability
    let medium_before = event_time - Duration::hours(6);
    let prob_medium = window.capture_probability(medium_before, event_time).unwrap();
    assert!(prob_medium > 0.0 && prob_medium < prob_recent);

    // Memory outside window - no capture
    let outside = event_time - Duration::hours(10);
    assert!(window.capture_probability(outside, event_time).is_none());
}

/// Test that decay functions work correctly.
///
/// Tags should decay over time, making older memories less likely to be captured.
#[test]
fn test_stc_decay_functions() {
    // Exponential decay
    let exp_decay = DecayFunction::Exponential;
    let exp_at_zero = exp_decay.apply(1.0, 0.0, 12.0);
    let exp_at_half = exp_decay.apply(1.0, 6.0, 12.0);
    let exp_at_end = exp_decay.apply(1.0, 12.0, 12.0);

    assert!((exp_at_zero - 1.0).abs() < 0.01, "Should be full strength at t=0");
    assert!(exp_at_half > 0.0 && exp_at_half < 0.5, "Significant decay at halfway");
    assert!(exp_at_end < 0.02, "Near zero at lifetime end");

    // Linear decay
    let linear_decay = DecayFunction::Linear;
    assert!((linear_decay.apply(1.0, 5.0, 10.0) - 0.5).abs() < 0.01, "Linear: 50% at halfway");
    assert!((linear_decay.apply(1.0, 10.0, 10.0) - 0.0).abs() < 0.01, "Linear: 0% at end");

    // Power decay (matches FSRS-6)
    let power_decay = DecayFunction::Power;
    let power_mid = power_decay.apply(1.0, 6.0, 12.0);
    assert!(power_mid > 0.5, "Power decay is slower than exponential");
}

/// Test importance cluster creation.
///
/// When an importance event captures multiple memories, they form a cluster
/// that provides context around a significant moment.
#[test]
fn test_stc_importance_clustering() {
    let mut stc = SynapticTaggingSystem::new();

    // Tag multiple memories
    stc.tag_memory("mem-1");
    stc.tag_memory("mem-2");
    stc.tag_memory("mem-3");

    // Trigger event
    let event = ImportanceEvent::user_flag("trigger", None);
    let result = stc.trigger_prp(event);

    // Should create cluster with captured memories
    assert!(result.cluster.is_some());
    let cluster = result.cluster.unwrap();
    assert!(cluster.size() >= 3);
    assert!(cluster.average_importance > 0.0);
}

/// Test batch operations for tagging and triggering.
///
/// The system should efficiently handle multiple memories and events.
#[test]
fn test_stc_batch_operations() {
    let mut stc = SynapticTaggingSystem::new();

    // Bulk tag memories
    let tags = stc.tag_memories(&["mem-1", "mem-2", "mem-3", "mem-4"]);
    assert_eq!(tags.len(), 4);

    // Batch trigger events
    let events = vec![
        ImportanceEvent::user_flag("trigger-1", None),
        ImportanceEvent::emotional("trigger-2", 0.9),
    ];
    let results = stc.trigger_prp_batch(events);
    assert_eq!(results.len(), 2);
}

/// Test statistics tracking.
///
/// The system should track comprehensive statistics about tagging and capture.
#[test]
fn test_stc_statistics_tracking() {
    let mut stc = SynapticTaggingSystem::new();

    stc.tag_memory("mem-1");
    stc.tag_memory("mem-2");

    let event = ImportanceEvent::user_flag("trigger", None);
    let _ = stc.trigger_prp(event);

    let stats = stc.stats();
    assert_eq!(stats.total_tags_created, 2);
    assert_eq!(stats.total_events, 1);
    assert!(stats.total_captures >= 2);
}

// ============================================================================
// MEMORY RECONSOLIDATION TESTS - 5 tests
// ============================================================================
// Based on Nader (2000): Retrieved memories enter a labile state
// where they can be modified before being reconsolidated.

/// Test that memories become labile when accessed.
///
/// According to reconsolidation theory, accessing a memory makes it
/// temporarily modifiable.
#[test]
fn test_reconsolidation_marks_memory_labile() {
    let mut manager = ReconsolidationManager::new();
    let snapshot = vestige_core::MemorySnapshot::capture(
        "Test content".to_string(),
        vec!["test".to_string()],
        0.8, 5.0, 0.9, vec![],
    );

    manager.mark_labile("mem-123", snapshot);

    assert!(manager.is_labile("mem-123"));
    assert!(!manager.is_labile("mem-456")); // Not marked
}

/// Test modifications during labile window.
///
/// While a memory is labile, various modifications can be applied.
#[test]
fn test_reconsolidation_apply_modifications() {
    let mut manager = ReconsolidationManager::new();
    let snapshot = vestige_core::MemorySnapshot::capture(
        "Original content".to_string(),
        vec!["original".to_string()],
        0.8, 5.0, 0.9, vec![],
    );

    manager.mark_labile("mem-123", snapshot);

    // Apply various modifications
    let success1 = manager.apply_modification("mem-123", Modification::AddTag {
        tag: "new-tag".to_string(),
    });
    let success2 = manager.apply_modification("mem-123", Modification::BoostRetrieval {
        boost: 0.1,
    });
    let success3 = manager.apply_modification("mem-123", Modification::LinkMemory {
        related_memory_id: "mem-456".to_string(),
        relationship: RelationshipType::Supports,
    });

    assert!(success1 && success2 && success3);
    assert_eq!(manager.get_stats().total_modifications, 3);
}

/// Test reconsolidation finalizes modifications.
///
/// When reconsolidation occurs, all pending modifications are applied.
#[test]
fn test_reconsolidation_finalizes_changes() {
    let mut manager = ReconsolidationManager::new();
    let snapshot = vestige_core::MemorySnapshot::capture(
        "Content".to_string(),
        vec!["tag".to_string()],
        0.8, 5.0, 0.9, vec![],
    );

    manager.mark_labile("mem-123", snapshot);
    manager.apply_modification("mem-123", Modification::AddTag {
        tag: "new-tag".to_string(),
    });
    manager.apply_modification("mem-123", Modification::AddContext {
        context: "Important meeting notes".to_string(),
    });

    let result = manager.reconsolidate("mem-123");

    assert!(result.is_some());
    let result = result.unwrap();
    assert!(result.was_modified);
    assert_eq!(result.change_summary.tags_added, 1);
    assert!(result.applied_modifications.len() >= 2);
}

/// Test access context is tracked.
///
/// The context of how a memory was accessed affects reconsolidation.
#[test]
fn test_reconsolidation_tracks_access_context() {
    let mut manager = ReconsolidationManager::new();
    let snapshot = vestige_core::MemorySnapshot::capture(
        "Content".to_string(),
        vec![], 0.8, 5.0, 0.9, vec![],
    );
    let context = AccessContext {
        trigger: AccessTrigger::Search,
        query: Some("test query".to_string()),
        co_retrieved: vec!["mem-2".to_string(), "mem-3".to_string()],
        session_id: Some("session-1".to_string()),
    };

    manager.mark_labile_with_context("mem-1", snapshot, context);

    let state = manager.get_labile_state("mem-1");
    assert!(state.is_some());
    assert!(state.unwrap().access_context.is_some());
}

/// Test retrieval history is maintained.
///
/// The system should track retrieval patterns over time.
#[test]
fn test_reconsolidation_retrieval_history() {
    let mut manager = ReconsolidationManager::new();
    let snapshot = vestige_core::MemorySnapshot::capture(
        "Content".to_string(),
        vec![], 0.8, 5.0, 0.9, vec![],
    );

    // Multiple retrievals
    for _ in 0..3 {
        manager.mark_labile("mem-123", snapshot.clone());
        manager.reconsolidate("mem-123");
    }

    assert_eq!(manager.get_retrieval_count("mem-123"), 3);
    assert_eq!(manager.get_retrieval_history("mem-123").len(), 3);
}

// ============================================================================
// FSRS-6 FORGETTING CURVES TESTS - 8 tests
// ============================================================================
// Based on FSRS-6 algorithm: power forgetting curve that is more accurate
// than exponential for modeling human memory.

/// Test retrievability at t=0 equals 1.0.
///
/// Immediately after encoding, a memory should be perfectly retrievable.
#[test]
fn test_fsrs_retrievability_at_zero() {
    let r = retrievability(10.0, 0.0);
    assert_eq!(r, 1.0, "Retrievability at t=0 should be 1.0");
}

/// Test retrievability decreases over time.
///
/// The forgetting curve shows monotonic decrease in recall probability.
#[test]
fn test_fsrs_retrievability_decreases() {
    let stability = 10.0;

    let r1 = retrievability(stability, 1.0);
    let r5 = retrievability(stability, 5.0);
    let r10 = retrievability(stability, 10.0);
    let r20 = retrievability(stability, 20.0);

    assert!(r1 > r5, "R at day 1 > R at day 5");
    assert!(r5 > r10, "R at day 5 > R at day 10");
    assert!(r10 > r20, "R at day 10 > R at day 20");
    assert!(r20 > 0.0, "R should never reach zero");
}

/// Test custom decay parameter affects forgetting rate.
///
/// FSRS-6's w20 parameter allows personalizing the forgetting curve.
#[test]
fn test_fsrs_custom_decay_parameter() {
    let stability = 10.0;
    let elapsed = 5.0;

    let r_low_decay = retrievability_with_decay(stability, elapsed, 0.1);
    let r_high_decay = retrievability_with_decay(stability, elapsed, 0.5);

    // Lower decay = steeper curve = lower retrievability for same time
    assert!(r_low_decay < r_high_decay,
        "Lower decay parameter should result in faster forgetting");
}

/// Test interval calculation round-trips with retrievability.
///
/// If we calculate an interval for a target R, retrievability at that
/// interval should match the target.
#[test]
fn test_fsrs_interval_retrievability_roundtrip() {
    let stability = 15.0;
    let target_r = 0.9;

    let interval = next_interval(stability, target_r);
    let actual_r = retrievability(stability, interval as f64);

    assert!(
        (actual_r - target_r).abs() < 0.05,
        "Round-trip: interval={}, actual_R={:.3}, target_R={:.3}",
        interval, actual_r, target_r
    );
}

/// Test initial difficulty ordering by rating.
///
/// Harder ratings should result in higher initial difficulty.
#[test]
fn test_fsrs_initial_difficulty_order() {
    let d_again = initial_difficulty(Rating::Again);
    let d_hard = initial_difficulty(Rating::Hard);
    let d_good = initial_difficulty(Rating::Good);
    let d_easy = initial_difficulty(Rating::Easy);

    assert!(d_again > d_hard, "Again > Hard difficulty");
    assert!(d_hard > d_good, "Hard > Good difficulty");
    assert!(d_good > d_easy, "Good > Easy difficulty");

    // All within valid bounds (1.0 to 10.0)
    for d in [d_again, d_hard, d_good, d_easy] {
        assert!((1.0..=10.0).contains(&d), "Difficulty {} out of bounds", d);
    }
}

/// Test scheduler handles first review correctly (FSRS-6 specific).
///
/// First review sets up initial stability and difficulty based on rating.
#[test]
fn test_fsrs_scheduler_first_review() {
    let scheduler = FSRSScheduler::default();
    let card = scheduler.new_card();

    let result = scheduler.review(&card, Rating::Good, 0.0, None);

    assert_eq!(result.state.reps, 1);
    assert_eq!(result.state.lapses, 0);
    assert!(result.interval > 0);
}

/// Test difficulty mean reversion.
///
/// Extreme difficulties should regress toward the mean over time.
#[test]
fn test_fsrs_difficulty_mean_reversion() {
    let scheduler = FSRSScheduler::default();

    // Create card with high difficulty
    let mut high_d_card = scheduler.new_card();
    high_d_card.difficulty = 9.0;
    let high_d_before = high_d_card.difficulty;

    // Good rating should move difficulty toward neutral
    let result = scheduler.review(&high_d_card, Rating::Good, 0.0, None);
    let high_d_after = result.state.difficulty;

    // Mean reversion should pull high difficulty down
    assert!(high_d_after < high_d_before, "High difficulty should decrease");

    // Create card with low difficulty
    let mut low_d_card = scheduler.new_card();
    low_d_card.difficulty = 2.0;
    let low_d_before = low_d_card.difficulty;

    // Again rating should increase difficulty
    let result = scheduler.review(&low_d_card, Rating::Again, 0.0, None);
    let low_d_after = result.state.difficulty;
    assert!(low_d_after > low_d_before, "Again should increase low difficulty");
}

/// Test scheduler lapse tracking.
///
/// When a review fails, it should be counted as a lapse.
#[test]
fn test_fsrs_scheduler_lapse_tracking() {
    let scheduler = FSRSScheduler::default();
    let mut card = scheduler.new_card();

    // First review - good
    let result = scheduler.review(&card, Rating::Good, 0.0, None);
    card = result.state;
    assert_eq!(card.lapses, 0);

    // Second review - lapse (Again)
    let result = scheduler.review(&card, Rating::Again, 1.0, None);
    assert!(result.is_lapse);
    assert_eq!(result.state.lapses, 1);
}

// ============================================================================
// MEMORY STATES TESTS - 7 tests
// ============================================================================
// Based on Bjork (1992): memories exist in different accessibility states
// and transitions between states follow specific rules.

/// Test accessibility multipliers for each state.
///
/// Different states have different base accessibility levels.
#[test]
fn test_memory_state_accessibility_multipliers() {
    assert!((MemoryState::Active.accessibility_multiplier() - 1.0).abs() < 0.001);
    assert!((MemoryState::Dormant.accessibility_multiplier() - 0.7).abs() < 0.001);
    assert!((MemoryState::Silent.accessibility_multiplier() - 0.3).abs() < 0.001);
    assert!((MemoryState::Unavailable.accessibility_multiplier() - 0.05).abs() < 0.001);

    // Active > Dormant > Silent > Unavailable
    assert!(MemoryState::Active.accessibility_multiplier() >
            MemoryState::Dormant.accessibility_multiplier());
    assert!(MemoryState::Dormant.accessibility_multiplier() >
            MemoryState::Silent.accessibility_multiplier());
    assert!(MemoryState::Silent.accessibility_multiplier() >
            MemoryState::Unavailable.accessibility_multiplier());
}

/// Test state retrievability properties.
///
/// Some states allow retrieval, others require strong cues or are blocked.
#[test]
fn test_memory_state_retrievability() {
    // Active and Dormant are retrievable
    assert!(MemoryState::Active.is_retrievable());
    assert!(MemoryState::Dormant.is_retrievable());

    // Silent requires strong cues
    assert!(!MemoryState::Silent.is_retrievable());
    assert!(MemoryState::Silent.requires_strong_cue());

    // Unavailable is blocked
    assert!(!MemoryState::Unavailable.is_retrievable());
    assert!(MemoryState::Unavailable.is_blocked());
}

/// Test lifecycle state transitions.
///
/// Accessing a memory should reactivate it to Active state.
#[test]
fn test_memory_lifecycle_transitions() {
    let mut lifecycle = MemoryLifecycle::with_state(MemoryState::Dormant);
    assert_eq!(lifecycle.state, MemoryState::Dormant);

    // Access should reactivate
    let changed = lifecycle.record_access();
    assert!(changed);
    assert_eq!(lifecycle.state, MemoryState::Active);
    assert_eq!(lifecycle.access_count, 2);
}

/// Test suppression from competition (retrieval-induced forgetting).
///
/// When memories compete, losers can be suppressed.
#[test]
fn test_memory_state_competition_suppression() {
    let mut lifecycle = MemoryLifecycle::new();

    lifecycle.suppress_from_competition(
        "winner-123".to_string(),
        0.85,
        Duration::hours(2),
    );

    assert_eq!(lifecycle.state, MemoryState::Unavailable);
    assert!(!lifecycle.is_suppression_expired());
    assert!(lifecycle.suppressed_by.contains(&"winner-123".to_string()));

    // Access should fail while suppressed
    let changed = lifecycle.record_access();
    assert!(!changed);
    assert_eq!(lifecycle.state, MemoryState::Unavailable);
}

/// Test cue reactivation of Silent memories.
///
/// Strong cues can reactivate Silent memories (like childhood memories).
#[test]
fn test_memory_state_cue_reactivation() {
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

/// Test competition manager tracks wins and losses.
///
/// The system should track how often memories win or lose competitions.
#[test]
fn test_memory_state_competition_tracking() {
    let mut manager = CompetitionManager::new();

    // Run competitions
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

/// Test accessibility calculator combines factors.
///
/// The final accessibility score combines state, recency, and frequency.
#[test]
fn test_memory_state_accessibility_calculator() {
    let calc = AccessibilityCalculator::default();
    let lifecycle = MemoryLifecycle::new();

    // Active memory just accessed should have high accessibility
    let score = calc.calculate(&lifecycle, 0.8);
    assert!(score > 0.8);
    assert!(score <= 1.0);

    // Test state affects minimum similarity threshold
    let active_threshold = calc.minimum_similarity_for_state(MemoryState::Active, 0.5);
    let silent_threshold = calc.minimum_similarity_for_state(MemoryState::Silent, 0.5);
    let unavailable_threshold = calc.minimum_similarity_for_state(MemoryState::Unavailable, 0.5);

    assert!(active_threshold < 0.5, "Active has lower threshold");
    assert!(silent_threshold > 0.5, "Silent has higher threshold");
    assert!(unavailable_threshold > 1.0, "Unavailable is effectively unreachable");
}

// ============================================================================
// MULTI-CHANNEL IMPORTANCE TESTS - 5 tests
// ============================================================================
// Based on neuromodulator systems: dopamine (novelty/reward), norepinephrine
// (arousal), and acetylcholine (attention) signal different types of importance.

/// Test novelty signal detects novel content.
///
/// Content never seen before should be rated as highly novel.
#[test]
fn test_importance_novelty_signal() {
    let mut novelty = NoveltySignal::new();
    let context = ImportanceContext::current();

    // First time seeing content should be novel
    let score1 = novelty.compute("The quick brown fox jumps over the lazy dog", &context);
    assert!(score1 > 0.5, "New content should be novel: {}", score1);

    // Learn the pattern
    novelty.update_model("The quick brown fox jumps over the lazy dog");
    novelty.update_model("The quick brown fox jumps over the lazy dog");
    novelty.update_model("The quick brown fox jumps over the lazy dog");

    // Same content should be less novel
    let score2 = novelty.compute("The quick brown fox jumps over the lazy dog", &context);
    assert!(score2 < score1, "Repeated content should be less novel");
}

/// Test arousal signal detects emotional content.
///
/// Emotionally charged content should have high arousal scores.
#[test]
fn test_importance_arousal_signal() {
    let arousal = ArousalSignal::new();

    // Neutral content
    let neutral_score = arousal.compute("The meeting is scheduled for tomorrow at 3pm.");

    // Highly emotional content
    let emotional_score = arousal.compute(
        "CRITICAL ERROR!!! Production database is DOWN! Data loss imminent!!!"
    );

    assert!(emotional_score > neutral_score,
        "Emotional content should have higher arousal: {} vs {}",
        emotional_score, neutral_score);
    assert!(emotional_score > 0.5, "Highly emotional content should score high");

    // Detect emotional markers
    let markers = arousal.detect_emotional_markers("URGENT: Critical failure!!!");
    assert!(!markers.is_empty(), "Should detect emotional markers");
}

/// Test reward signal tracks outcomes.
///
/// Memories with positive outcomes should have higher reward scores.
#[test]
fn test_importance_reward_signal() {
    let reward = RewardSignal::new();

    // Record positive outcomes
    reward.record_outcome("mem-helpful", OutcomeType::Helpful);
    reward.record_outcome("mem-helpful", OutcomeType::VeryHelpful);
    reward.record_outcome("mem-helpful", OutcomeType::Helpful);

    let helpful_score = reward.compute("mem-helpful");
    assert!(helpful_score > 0.5, "Memory with positive outcomes should score high");

    // Record negative outcomes
    reward.record_outcome("mem-unhelpful", OutcomeType::NotHelpful);
    reward.record_outcome("mem-unhelpful", OutcomeType::NotHelpful);

    let unhelpful_score = reward.compute("mem-unhelpful");
    assert!(unhelpful_score < 0.5, "Memory with negative outcomes should score low");

    assert!(helpful_score > unhelpful_score);
}

/// Test attention signal detects learning mode.
///
/// High query frequency and diverse access patterns indicate learning.
#[test]
fn test_importance_attention_signal() {
    let attention = AttentionSignal::new();

    // Create a learning-like session
    let learning_session = AttentionSession {
        session_id: "learning-1".to_string(),
        start_time: Utc::now(),
        duration_minutes: 45.0,
        query_count: 20,
        edit_count: 2,
        unique_memories_accessed: 15,
        viewed_docs: true,
        query_topics: vec!["rust".to_string(), "async".to_string(), "memory".to_string()],
    };

    assert!(attention.detect_learning_mode(&learning_session),
        "Should detect learning mode from session patterns");

    // Non-learning session (quick edit)
    let quick_session = AttentionSession {
        session_id: "quick-1".to_string(),
        start_time: Utc::now(),
        duration_minutes: 2.0,
        query_count: 1,
        edit_count: 5,
        unique_memories_accessed: 1,
        viewed_docs: false,
        query_topics: vec![],
    };

    assert!(!attention.detect_learning_mode(&quick_session),
        "Quick edit session should not be learning mode");
}

/// Test composite importance combines all signals.
///
/// The final importance score weights novelty, arousal, reward, and attention.
#[test]
fn test_importance_composite_score() {
    let signals = ImportanceSignals::new();
    let context = ImportanceContext::current()
        .with_project("test-project")
        .with_learning_session(true);

    // Test with emotional, novel content
    let score = signals.compute_importance(
        "BREAKTHROUGH: Solved the critical performance issue blocking release!!!",
        &context,
    );

    assert!(score.composite > 0.4, "Important content should score moderately high");
    assert!(score.arousal > 0.4, "Emotional content should have arousal");
    assert!(score.encoding_boost >= 1.0, "High importance should boost encoding");

    // Verify all components are present
    assert!(score.novelty >= 0.0 && score.novelty <= 1.0);
    assert!(score.arousal >= 0.0 && score.arousal <= 1.0);
    assert!(score.reward >= 0.0 && score.reward <= 1.0);
    assert!(score.attention >= 0.0 && score.attention <= 1.0);

    // Verify explanation exists
    let explanation = score.explain();
    assert!(!explanation.is_empty());
}
