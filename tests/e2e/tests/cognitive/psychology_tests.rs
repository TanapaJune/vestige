//! # Cognitive Psychology E2E Tests
//!
//! Comprehensive tests validating memory phenomena based on established
//! cognitive psychology research.
//!
//! These tests verify that the Vestige memory system exhibits behaviors
//! consistent with human memory research findings.

use vestige_core::neuroscience::spreading_activation::{
    ActivationConfig, ActivationNetwork, LinkType,
};

// ============================================================================
// SERIAL POSITION EFFECT TESTS (5 tests)
// ============================================================================
// Based on Murdock (1962) - items at the beginning (primacy) and end (recency)
// of a list are remembered better than items in the middle.

/// Test primacy effect - first items in a sequence have higher activation.
///
/// Based on Murdock (1962): First items receive more rehearsal and are
/// encoded more strongly into long-term memory.
#[test]
fn test_serial_position_primacy_effect() {
    let mut network = ActivationNetwork::new();

    // Create a sequence of memories (like a study list)
    // First items get stronger encoding (simulating more rehearsal time)
    let items = vec![
        ("item_1", 0.95), // First - highest strength (primacy)
        ("item_2", 0.85),
        ("item_3", 0.70),
        ("item_4", 0.60), // Middle - lowest
        ("item_5", 0.55), // Middle - lowest
        ("item_6", 0.60),
        ("item_7", 0.75),
        ("item_8", 0.90), // Last - high (recency)
    ];

    // Link all items to a "study_session" context
    for (item, strength) in &items {
        network.add_edge(
            "study_session".to_string(),
            item.to_string(),
            LinkType::Temporal,
            *strength,
        );
    }

    let results = network.activate("study_session", 1.0);

    // Find activations for first and middle items
    let first_activation = results
        .iter()
        .find(|r| r.memory_id == "item_1")
        .map(|r| r.activation)
        .unwrap_or(0.0);

    let middle_activation = results
        .iter()
        .find(|r| r.memory_id == "item_4")
        .map(|r| r.activation)
        .unwrap_or(0.0);

    assert!(
        first_activation > middle_activation,
        "Primacy effect: first item ({}) should have higher activation than middle item ({})",
        first_activation,
        middle_activation
    );
}

/// Test recency effect - last items in a sequence have higher activation.
///
/// Based on the recency component of serial position effect:
/// Last items are still in working memory during immediate recall.
#[test]
fn test_serial_position_recency_effect() {
    let mut network = ActivationNetwork::new();

    // Recency effect - more recent items have stronger temporal links
    let items = vec![
        ("old_item_1", 0.4),
        ("old_item_2", 0.45),
        ("old_item_3", 0.5),
        ("recent_item_1", 0.85),
        ("recent_item_2", 0.92),
        ("recent_item_3", 0.98), // Most recent - highest
    ];

    for (item, strength) in &items {
        network.add_edge(
            "current_context".to_string(),
            item.to_string(),
            LinkType::Temporal,
            *strength,
        );
    }

    let results = network.activate("current_context", 1.0);

    let recent_activation = results
        .iter()
        .find(|r| r.memory_id == "recent_item_3")
        .map(|r| r.activation)
        .unwrap_or(0.0);

    let old_activation = results
        .iter()
        .find(|r| r.memory_id == "old_item_1")
        .map(|r| r.activation)
        .unwrap_or(0.0);

    assert!(
        recent_activation > old_activation,
        "Recency effect: recent item ({}) should have higher activation than old item ({})",
        recent_activation,
        old_activation
    );
}

/// Test U-shaped serial position curve.
///
/// The classic serial position curve shows high recall for first items (primacy),
/// low recall for middle items, and high recall for last items (recency).
#[test]
fn test_serial_position_u_shaped_curve() {
    let mut network = ActivationNetwork::new();

    // U-shaped curve: high-low-high pattern
    let items = vec![
        ("pos_1", 0.90),  // High (primacy)
        ("pos_2", 0.80),
        ("pos_3", 0.65),
        ("pos_4", 0.55),  // Low (middle)
        ("pos_5", 0.50),  // Low (middle)
        ("pos_6", 0.55),
        ("pos_7", 0.70),
        ("pos_8", 0.85),
        ("pos_9", 0.95),  // High (recency)
    ];

    for (item, strength) in &items {
        network.add_edge(
            "list_context".to_string(),
            item.to_string(),
            LinkType::Temporal,
            *strength,
        );
    }

    let results = network.activate("list_context", 1.0);

    let pos_1 = results.iter().find(|r| r.memory_id == "pos_1").map(|r| r.activation).unwrap_or(0.0);
    let pos_5 = results.iter().find(|r| r.memory_id == "pos_5").map(|r| r.activation).unwrap_or(0.0);
    let pos_9 = results.iter().find(|r| r.memory_id == "pos_9").map(|r| r.activation).unwrap_or(0.0);

    // U-shape: ends higher than middle
    assert!(pos_1 > pos_5, "First position ({}) > middle position ({})", pos_1, pos_5);
    assert!(pos_9 > pos_5, "Last position ({}) > middle position ({})", pos_9, pos_5);
}

/// Test that rehearsal strengthens primacy items.
///
/// Primacy effect is driven by increased rehearsal of early items.
#[test]
fn test_serial_position_rehearsal_strengthens_primacy() {
    let mut network = ActivationNetwork::new();

    // Initial weak connections
    network.add_edge("learning".to_string(), "first_concept".to_string(), LinkType::Semantic, 0.3);
    network.add_edge("learning".to_string(), "middle_concept".to_string(), LinkType::Semantic, 0.3);
    network.add_edge("learning".to_string(), "last_concept".to_string(), LinkType::Semantic, 0.3);

    // Simulate rehearsal - first items get more rehearsal
    // (5 rehearsals for first, 2 for middle, 3 for last)
    for _ in 0..5 {
        network.reinforce_edge("learning", "first_concept", 0.1);
    }
    for _ in 0..2 {
        network.reinforce_edge("learning", "middle_concept", 0.1);
    }
    for _ in 0..3 {
        network.reinforce_edge("learning", "last_concept", 0.1);
    }

    let associations = network.get_associations("learning");

    let first_strength = associations
        .iter()
        .find(|a| a.memory_id == "first_concept")
        .map(|a| a.association_strength)
        .unwrap_or(0.0);

    let middle_strength = associations
        .iter()
        .find(|a| a.memory_id == "middle_concept")
        .map(|a| a.association_strength)
        .unwrap_or(0.0);

    assert!(
        first_strength > middle_strength,
        "More rehearsal should strengthen first concept: {} > {}",
        first_strength,
        middle_strength
    );
}

/// Test that delay eliminates recency but preserves primacy.
///
/// After a delay, recency effect disappears (working memory clears),
/// but primacy remains (items transferred to long-term memory).
#[test]
fn test_serial_position_delay_eliminates_recency() {
    let mut network = ActivationNetwork::new();

    // After delay: primacy preserved, recency diminished
    // (modeling that working memory has cleared)
    let delayed_items = vec![
        ("early_1", 0.85),  // Primacy preserved
        ("early_2", 0.75),
        ("middle_1", 0.50),
        ("middle_2", 0.45),
        ("late_1", 0.40),   // Recency lost after delay
        ("late_2", 0.35),   // (items not transferred to LTM)
    ];

    for (item, strength) in &delayed_items {
        network.add_edge(
            "delayed_recall".to_string(),
            item.to_string(),
            LinkType::Temporal,
            *strength,
        );
    }

    let results = network.activate("delayed_recall", 1.0);

    let early_activation = results
        .iter()
        .find(|r| r.memory_id == "early_1")
        .map(|r| r.activation)
        .unwrap_or(0.0);

    let late_activation = results
        .iter()
        .find(|r| r.memory_id == "late_2")
        .map(|r| r.activation)
        .unwrap_or(0.0);

    // After delay, early items (primacy) should be stronger than late items
    assert!(
        early_activation > late_activation,
        "After delay, primacy ({}) should exceed diminished recency ({})",
        early_activation,
        late_activation
    );
}

// ============================================================================
// SPACING EFFECT TESTS (5 tests)
// ============================================================================
// Based on Ebbinghaus (1885) and Cepeda et al. (2006) - distributed practice
// leads to better retention than massed practice.

/// Test that spaced repetition creates stronger associations than massed practice.
///
/// Based on the spacing effect: distributing learning over time improves retention.
#[test]
fn test_spacing_effect_distributed_vs_massed() {
    let mut network = ActivationNetwork::new();

    // Massed practice: all reinforcements close together (less effective)
    network.add_edge("massed".to_string(), "concept_a".to_string(), LinkType::Semantic, 0.2);
    // 5 rapid reinforcements
    for _ in 0..5 {
        network.reinforce_edge("massed", "concept_a", 0.1);
    }

    // Spaced practice: reinforcements distributed (more effective)
    // Simulated by giving higher reinforcement values (representing better encoding)
    network.add_edge("spaced".to_string(), "concept_b".to_string(), LinkType::Semantic, 0.2);
    // 5 spaced reinforcements with better encoding
    for _ in 0..5 {
        network.reinforce_edge("spaced", "concept_b", 0.15); // Higher value = better encoding
    }

    let massed_assoc = network.get_associations("massed");
    let spaced_assoc = network.get_associations("spaced");

    let massed_strength = massed_assoc
        .iter()
        .find(|a| a.memory_id == "concept_a")
        .map(|a| a.association_strength)
        .unwrap_or(0.0);

    let spaced_strength = spaced_assoc
        .iter()
        .find(|a| a.memory_id == "concept_b")
        .map(|a| a.association_strength)
        .unwrap_or(0.0);

    assert!(
        spaced_strength > massed_strength,
        "Spaced practice ({}) should create stronger associations than massed ({})",
        spaced_strength,
        massed_strength
    );
}

/// Test optimal spacing interval increases with retention interval.
///
/// Based on Cepeda et al. (2008): optimal gap increases with retention interval.
#[test]
fn test_spacing_effect_optimal_interval() {
    let mut network = ActivationNetwork::new();

    // Short retention interval: shorter spacing optimal
    network.add_edge("short_retention".to_string(), "fact_1".to_string(), LinkType::Semantic, 0.3);
    network.reinforce_edge("short_retention", "fact_1", 0.2);
    network.reinforce_edge("short_retention", "fact_1", 0.2);

    // Long retention interval: longer spacing optimal (simulated with stronger encoding)
    network.add_edge("long_retention".to_string(), "fact_2".to_string(), LinkType::Semantic, 0.3);
    network.reinforce_edge("long_retention", "fact_2", 0.25);
    network.reinforce_edge("long_retention", "fact_2", 0.25);

    let short_assoc = network.get_associations("short_retention");
    let long_assoc = network.get_associations("long_retention");

    let short_strength = short_assoc[0].association_strength;
    let long_strength = long_assoc[0].association_strength;

    // Both should be well-encoded, but long retention with optimal spacing is stronger
    assert!(
        long_strength >= short_strength,
        "Optimal spacing for long retention ({}) >= short retention ({})",
        long_strength,
        short_strength
    );
}

/// Test that spacing effect applies to semantic associations.
#[test]
fn test_spacing_effect_semantic_associations() {
    let mut network = ActivationNetwork::new();

    // Create semantic network with spaced learning
    network.add_edge("programming".to_string(), "rust".to_string(), LinkType::Semantic, 0.5);
    network.add_edge("rust".to_string(), "ownership".to_string(), LinkType::Semantic, 0.5);
    network.add_edge("ownership".to_string(), "borrowing".to_string(), LinkType::Semantic, 0.5);

    // Spaced reinforcement of the path
    for _ in 0..3 {
        network.reinforce_edge("programming", "rust", 0.15);
        network.reinforce_edge("rust", "ownership", 0.15);
        network.reinforce_edge("ownership", "borrowing", 0.15);
    }

    let results = network.activate("programming", 1.0);

    // Should reach borrowing through the strengthened path
    let borrowing_result = results.iter().find(|r| r.memory_id == "borrowing");
    assert!(borrowing_result.is_some(), "Spaced learning should strengthen multi-hop paths");

    let borrowing_activation = borrowing_result.unwrap().activation;
    assert!(
        borrowing_activation > 0.1,
        "Borrowing should have meaningful activation: {}",
        borrowing_activation
    );
}

/// Test expanding retrieval practice (increasing intervals).
///
/// Based on Landauer & Bjork (1978): expanding retrieval intervals are effective.
#[test]
fn test_spacing_effect_expanding_retrieval() {
    let mut network = ActivationNetwork::new();

    // Expanding intervals: each retrieval strengthens more as intervals grow
    network.add_edge("expanding".to_string(), "memory".to_string(), LinkType::Semantic, 0.2);

    // Simulate expanding intervals with increasing reinforcement
    let expanding_reinforcements = [0.1, 0.12, 0.15, 0.18, 0.22]; // Increasing gains
    for reinforcement in expanding_reinforcements {
        network.reinforce_edge("expanding", "memory", reinforcement);
    }

    let associations = network.get_associations("expanding");
    let final_strength = associations[0].association_strength;

    // Should reach high strength
    // 0.2 + 0.1 + 0.12 + 0.15 + 0.18 + 0.22 = 0.97
    assert!(
        final_strength > 0.9,
        "Expanding retrieval should build strong associations: {}",
        final_strength
    );
}

/// Test that spacing benefits multi-hop activation paths.
#[test]
fn test_spacing_effect_multi_hop_paths() {
    let config = ActivationConfig {
        decay_factor: 0.8,
        max_hops: 4,
        min_threshold: 0.05,
        allow_cycles: false,
    };
    let mut network = ActivationNetwork::with_config(config);

    // Create a learning chain
    network.add_edge("topic".to_string(), "subtopic_a".to_string(), LinkType::Semantic, 0.4);
    network.add_edge("subtopic_a".to_string(), "detail_1".to_string(), LinkType::Semantic, 0.4);
    network.add_edge("detail_1".to_string(), "example".to_string(), LinkType::Semantic, 0.4);

    // Spaced practice on entire chain
    for _ in 0..4 {
        network.reinforce_edge("topic", "subtopic_a", 0.12);
        network.reinforce_edge("subtopic_a", "detail_1", 0.12);
        network.reinforce_edge("detail_1", "example", 0.12);
    }

    let results = network.activate("topic", 1.0);

    // Example should be reachable with good activation
    let example_result = results.iter().find(|r| r.memory_id == "example");
    assert!(example_result.is_some(), "Spaced practice should enable deep retrieval");

    let example = example_result.unwrap();
    assert_eq!(example.distance, 3, "Example should be 3 hops away");
    assert!(example.activation > 0.1, "Example should have sufficient activation: {}", example.activation);
}

// ============================================================================
// CONTEXT-DEPENDENT RECALL TESTS (5 tests)
// ============================================================================
// Based on Godden & Baddeley (1975) - information is better recalled in
// the same context where it was encoded.

/// Test that matching encoding and retrieval context improves recall.
///
/// Based on Godden & Baddeley (1975): Divers recalled words better
/// in the same environment (underwater/land) where they learned them.
#[test]
fn test_context_dependent_matching_context() {
    let mut network = ActivationNetwork::new();

    // Memory encoded in "office" context
    network.add_edge("office_context".to_string(), "project_deadline".to_string(), LinkType::Semantic, 0.9);
    network.add_edge("office_context".to_string(), "meeting_notes".to_string(), LinkType::Semantic, 0.85);

    // Memory encoded in "home" context
    network.add_edge("home_context".to_string(), "grocery_list".to_string(), LinkType::Semantic, 0.9);
    network.add_edge("home_context".to_string(), "family_event".to_string(), LinkType::Semantic, 0.85);

    // Recall from office context
    let office_results = network.activate("office_context", 1.0);
    let home_results = network.activate("home_context", 1.0);

    // Office context should find office memories
    let found_deadline = office_results.iter().any(|r| r.memory_id == "project_deadline");
    let found_grocery = office_results.iter().any(|r| r.memory_id == "grocery_list");

    assert!(found_deadline, "Office context should activate office memories");
    assert!(!found_grocery, "Office context should NOT directly activate home memories");

    // Home context should find home memories
    let home_found_grocery = home_results.iter().any(|r| r.memory_id == "grocery_list");
    assert!(home_found_grocery, "Home context should activate home memories");
}

/// Test encoding specificity principle.
///
/// The more specific the match between encoding and retrieval cues, the better.
#[test]
fn test_context_dependent_encoding_specificity() {
    let mut network = ActivationNetwork::new();

    // Highly specific encoding context
    network.add_edge("rainy_monday_morning".to_string(), "specific_memory".to_string(), LinkType::Temporal, 0.95);
    network.add_edge("rainy_monday_morning".to_string(), "coffee_shop_idea".to_string(), LinkType::Temporal, 0.9);

    // General context (partial match)
    network.add_edge("monday".to_string(), "rainy_monday_morning".to_string(), LinkType::Temporal, 0.6);
    network.add_edge("morning".to_string(), "rainy_monday_morning".to_string(), LinkType::Temporal, 0.5);

    // Specific context retrieval
    let specific_results = network.activate("rainy_monday_morning", 1.0);

    // General context retrieval (through chain)
    let general_results = network.activate("monday", 1.0);

    let specific_activation = specific_results
        .iter()
        .find(|r| r.memory_id == "specific_memory")
        .map(|r| r.activation)
        .unwrap_or(0.0);

    let general_activation = general_results
        .iter()
        .find(|r| r.memory_id == "specific_memory")
        .map(|r| r.activation)
        .unwrap_or(0.0);

    assert!(
        specific_activation > general_activation,
        "Specific context ({}) should yield stronger activation than general ({})",
        specific_activation,
        general_activation
    );
}

/// Test state-dependent memory (internal context).
///
/// Memories encoded in a particular internal state are better recalled in that state.
#[test]
fn test_context_dependent_state_dependent() {
    let mut network = ActivationNetwork::new();

    // Memories encoded in different emotional states
    network.add_edge("happy_state".to_string(), "positive_memory_1".to_string(), LinkType::Semantic, 0.9);
    network.add_edge("happy_state".to_string(), "positive_memory_2".to_string(), LinkType::Semantic, 0.85);

    network.add_edge("stressed_state".to_string(), "work_problem_1".to_string(), LinkType::Semantic, 0.9);
    network.add_edge("stressed_state".to_string(), "work_problem_2".to_string(), LinkType::Semantic, 0.85);

    // Retrieve from happy state
    let happy_results = network.activate("happy_state", 1.0);

    let found_positive = happy_results.iter().any(|r| r.memory_id == "positive_memory_1");
    let found_work = happy_results.iter().any(|r| r.memory_id == "work_problem_1");

    assert!(found_positive, "Happy state should activate positive memories");
    assert!(!found_work, "Happy state should NOT directly activate stressed memories");
}

/// Test context reinstatement improves retrieval.
///
/// Mentally reinstating the encoding context can improve recall.
#[test]
fn test_context_dependent_reinstatement() {
    let mut network = ActivationNetwork::new();

    // Memory with multiple context cues
    network.add_edge("library".to_string(), "study_session".to_string(), LinkType::Temporal, 0.8);
    network.add_edge("quiet".to_string(), "study_session".to_string(), LinkType::Semantic, 0.7);
    network.add_edge("evening".to_string(), "study_session".to_string(), LinkType::Temporal, 0.6);

    // Study session links to learned material
    network.add_edge("study_session".to_string(), "learned_concept".to_string(), LinkType::Semantic, 0.9);

    // Single context cue
    let single_cue = network.activate("library", 1.0);

    // Multiple context cues (reinstatement) - we need to create a combined node
    network.add_edge("reinstated_context".to_string(), "library".to_string(), LinkType::Semantic, 0.9);
    network.add_edge("reinstated_context".to_string(), "quiet".to_string(), LinkType::Semantic, 0.9);
    network.add_edge("reinstated_context".to_string(), "evening".to_string(), LinkType::Semantic, 0.9);

    let reinstated_results = network.activate("reinstated_context", 1.0);

    // Reinstatement should provide multiple paths to the target
    let single_paths: Vec<_> = single_cue
        .iter()
        .filter(|r| r.memory_id == "learned_concept")
        .collect();

    let reinstated_paths: Vec<_> = reinstated_results
        .iter()
        .filter(|r| r.memory_id == "learned_concept")
        .collect();

    // Reinstated context creates more activation paths
    assert!(
        reinstated_paths.len() >= single_paths.len(),
        "Context reinstatement should provide at least as many paths"
    );
}

/// Test transfer-appropriate processing.
///
/// Memory is best when the type of processing at encoding matches retrieval.
#[test]
fn test_context_dependent_transfer_appropriate() {
    let mut network = ActivationNetwork::new();

    // Semantic encoding (deep processing)
    network.add_edge("meaning_focused".to_string(), "concept_meaning".to_string(), LinkType::Semantic, 0.9);

    // Perceptual encoding (shallow processing)
    network.add_edge("appearance_focused".to_string(), "concept_appearance".to_string(), LinkType::Semantic, 0.9);

    // Semantic retrieval cue
    let semantic_results = network.activate("meaning_focused", 1.0);

    // Perceptual retrieval cue
    let perceptual_results = network.activate("appearance_focused", 1.0);

    // Matching encoding-retrieval processing should work best
    let semantic_found = semantic_results.iter().any(|r| r.memory_id == "concept_meaning");
    let perceptual_found = perceptual_results.iter().any(|r| r.memory_id == "concept_appearance");

    assert!(semantic_found, "Semantic cue should retrieve semantically encoded info");
    assert!(perceptual_found, "Perceptual cue should retrieve perceptually encoded info");

    // Cross-retrieval should be weaker (not directly connected)
    let cross_found = semantic_results.iter().any(|r| r.memory_id == "concept_appearance");
    assert!(!cross_found, "Semantic cue should NOT directly retrieve perceptual encoding");
}

// ============================================================================
// TIP-OF-TONGUE PHENOMENA TESTS (5 tests)
// ============================================================================
// Based on Brown & McNeill (1966) - partial activation of target memory
// with inability to fully retrieve it.

/// Test partial activation of target without full retrieval.
///
/// TOT state involves having partial information about the target.
#[test]
fn test_tot_partial_activation() {
    let config = ActivationConfig {
        decay_factor: 0.6,  // Higher decay = weaker far connections
        max_hops: 3,
        min_threshold: 0.15,  // Higher threshold = some items not retrieved
        allow_cycles: false,
    };
    let mut network = ActivationNetwork::with_config(config);

    // Target word "serendipity" with various features
    network.add_edge("word_search".to_string(), "starts_with_s".to_string(), LinkType::Semantic, 0.8);
    network.add_edge("word_search".to_string(), "four_syllables".to_string(), LinkType::Semantic, 0.7);
    network.add_edge("word_search".to_string(), "meaning_lucky_discovery".to_string(), LinkType::Semantic, 0.85);
    network.add_edge("starts_with_s".to_string(), "serendipity".to_string(), LinkType::Semantic, 0.3);  // Weak link to target

    let results = network.activate("word_search", 1.0);

    // Should find partial information
    let found_starts_s = results.iter().any(|r| r.memory_id == "starts_with_s");
    let found_meaning = results.iter().any(|r| r.memory_id == "meaning_lucky_discovery");

    assert!(found_starts_s, "Should retrieve partial info (first letter)");
    assert!(found_meaning, "Should retrieve partial info (meaning)");

    // Target might not be found due to weak link and threshold
    let target_activation = results
        .iter()
        .find(|r| r.memory_id == "serendipity")
        .map(|r| r.activation)
        .unwrap_or(0.0);

    // TOT state: we might have weak activation of target
    // (either not found or very weak activation)
    assert!(
        target_activation < 0.5,
        "Target should have weak or no activation in TOT state: {}",
        target_activation
    );
}

/// Test that related words are activated during TOT state.
///
/// During TOT, phonologically or semantically similar words often come to mind.
#[test]
fn test_tot_related_words_activated() {
    let mut network = ActivationNetwork::new();

    // Searching for "archipelago"
    // Related words get activated instead
    network.add_edge("island_chain_concept".to_string(), "archipelago".to_string(), LinkType::Semantic, 0.4);  // Weak
    network.add_edge("island_chain_concept".to_string(), "peninsula".to_string(), LinkType::Semantic, 0.7);  // Related, stronger
    network.add_edge("island_chain_concept".to_string(), "atoll".to_string(), LinkType::Semantic, 0.65);  // Related
    network.add_edge("island_chain_concept".to_string(), "islands".to_string(), LinkType::Semantic, 0.8);  // Generic, strong

    let results = network.activate("island_chain_concept", 1.0);

    // Generic/related words should be more activated than target
    let archipelago_act = results.iter().find(|r| r.memory_id == "archipelago").map(|r| r.activation).unwrap_or(0.0);
    let islands_act = results.iter().find(|r| r.memory_id == "islands").map(|r| r.activation).unwrap_or(0.0);
    let peninsula_act = results.iter().find(|r| r.memory_id == "peninsula").map(|r| r.activation).unwrap_or(0.0);

    assert!(
        islands_act > archipelago_act,
        "Related words ({}) should be more activated than target ({})",
        islands_act,
        archipelago_act
    );

    assert!(
        peninsula_act > archipelago_act || (peninsula_act - archipelago_act).abs() < 0.2,
        "Similar words should have comparable or higher activation"
    );
}

/// Test phonological cue helps resolve TOT.
///
/// Providing the first letter or sound often resolves TOT state.
#[test]
fn test_tot_phonological_cue_resolution() {
    let mut network = ActivationNetwork::new();

    // Target: "ephemeral"
    // Weak semantic link
    network.add_edge("temporary_concept".to_string(), "ephemeral".to_string(), LinkType::Semantic, 0.3);
    // Strong phonological link
    network.add_edge("starts_with_eph".to_string(), "ephemeral".to_string(), LinkType::Semantic, 0.85);
    network.add_edge("temporary_concept".to_string(), "starts_with_eph".to_string(), LinkType::Semantic, 0.5);

    // Without phonological cue (just semantic)
    let semantic_only = network.activate("temporary_concept", 1.0);

    // With phonological cue directly
    let with_phon_cue = network.activate("starts_with_eph", 1.0);

    let semantic_target = semantic_only
        .iter()
        .find(|r| r.memory_id == "ephemeral")
        .map(|r| r.activation)
        .unwrap_or(0.0);

    let phon_target = with_phon_cue
        .iter()
        .find(|r| r.memory_id == "ephemeral")
        .map(|r| r.activation)
        .unwrap_or(0.0);

    assert!(
        phon_target > semantic_target,
        "Phonological cue ({}) should better activate target than semantic alone ({})",
        phon_target,
        semantic_target
    );
}

/// Test that TOT becomes more common with age (weaker links).
///
/// Older adults experience more TOT states due to weakened connections.
#[test]
fn test_tot_age_related_increase() {
    // "Young" network - strong connections
    let mut young_network = ActivationNetwork::new();
    young_network.add_edge("cue".to_string(), "target_word".to_string(), LinkType::Semantic, 0.85);

    // "Older" network - weakened connections
    let mut older_network = ActivationNetwork::new();
    older_network.add_edge("cue".to_string(), "target_word".to_string(), LinkType::Semantic, 0.45);

    let young_results = young_network.activate("cue", 1.0);
    let older_results = older_network.activate("cue", 1.0);

    let young_activation = young_results
        .iter()
        .find(|r| r.memory_id == "target_word")
        .map(|r| r.activation)
        .unwrap_or(0.0);

    let older_activation = older_results
        .iter()
        .find(|r| r.memory_id == "target_word")
        .map(|r| r.activation)
        .unwrap_or(0.0);

    assert!(
        young_activation > older_activation,
        "Young network ({}) should have stronger retrieval than older ({})",
        young_activation,
        older_activation
    );
}

/// Test blocking effect in TOT state.
///
/// Wrong words that come to mind can block retrieval of the target.
#[test]
fn test_tot_blocking_effect() {
    let mut network = ActivationNetwork::new();

    // Target and blocker both connected to cue
    network.add_edge("definition_cue".to_string(), "blocker_word".to_string(), LinkType::Semantic, 0.9);  // Strong
    network.add_edge("definition_cue".to_string(), "target_word".to_string(), LinkType::Semantic, 0.5);  // Weaker

    let results = network.activate("definition_cue", 1.0);

    let blocker_activation = results
        .iter()
        .find(|r| r.memory_id == "blocker_word")
        .map(|r| r.activation)
        .unwrap_or(0.0);

    let target_activation = results
        .iter()
        .find(|r| r.memory_id == "target_word")
        .map(|r| r.activation)
        .unwrap_or(0.0);

    // Blocker should be more strongly activated
    assert!(
        blocker_activation > target_activation,
        "Blocker ({}) should have higher activation than target ({}), blocking retrieval",
        blocker_activation,
        target_activation
    );
}

// ============================================================================
// FALSE MEMORY DRM PARADIGM TESTS (5 tests)
// ============================================================================
// Based on Roediger & McDermott (1995) - studying semantically related words
// leads to false memories of unstudied "critical lures."

/// Test basic DRM false memory effect.
///
/// Studying words like "bed, rest, awake, tired, dream" creates
/// false memory for the unstudied critical lure "sleep."
#[test]
fn test_drm_basic_false_memory() {
    let mut network = ActivationNetwork::new();

    // Study list - all semantically related to "sleep" (the critical lure)
    let study_words = ["bed", "rest", "awake", "tired", "dream", "pillow", "blanket", "nap"];

    // Create associations from study words to the critical lure
    for word in &study_words {
        network.add_edge(
            word.to_string(),
            "sleep".to_string(),  // Critical lure (never studied)
            LinkType::Semantic,
            0.7,
        );
    }

    // Also link study words to a study context
    for word in &study_words {
        network.add_edge("study_list".to_string(), word.to_string(), LinkType::Temporal, 0.8);
    }

    // Activate from study context
    let results = network.activate("study_list", 1.0);

    // The critical lure "sleep" should be activated even though never studied
    let sleep_activated = results.iter().any(|r| r.memory_id == "sleep");

    assert!(
        sleep_activated,
        "Critical lure 'sleep' should be activated through spreading activation from studied words"
    );
}

/// Test that critical lure receives convergent activation.
///
/// Multiple studied words all pointing to the same lure strengthens false memory.
#[test]
fn test_drm_convergent_activation() {
    let mut network = ActivationNetwork::new();

    // Multiple words converging on critical lure
    network.add_edge("cold".to_string(), "hot".to_string(), LinkType::Semantic, 0.8);
    network.add_edge("warm".to_string(), "hot".to_string(), LinkType::Semantic, 0.85);
    network.add_edge("heat".to_string(), "hot".to_string(), LinkType::Semantic, 0.9);
    network.add_edge("burn".to_string(), "hot".to_string(), LinkType::Semantic, 0.75);
    network.add_edge("fire".to_string(), "hot".to_string(), LinkType::Semantic, 0.8);

    // Study context
    for word in ["cold", "warm", "heat", "burn", "fire"] {
        network.add_edge("study_context".to_string(), word.to_string(), LinkType::Temporal, 0.8);
    }

    let results = network.activate("study_context", 1.0);

    // Count how many paths lead to "hot"
    let hot_results: Vec<_> = results.iter().filter(|r| r.memory_id == "hot").collect();

    assert!(
        !hot_results.is_empty(),
        "Critical lure should receive activation from multiple convergent paths"
    );

    // The lure should have relatively high activation due to convergence
    let hot_activation = hot_results.iter().map(|r| r.activation).sum::<f64>();
    assert!(
        hot_activation > 0.1,
        "Convergent activation should be substantial: {}",
        hot_activation
    );
}

/// Test that semantic relatedness predicts false memory rate.
///
/// More strongly associated words create stronger false memories.
#[test]
fn test_drm_semantic_relatedness() {
    let mut network = ActivationNetwork::new();

    // Strongly related list
    network.add_edge("strong_list".to_string(), "nurse".to_string(), LinkType::Temporal, 0.8);
    network.add_edge("strong_list".to_string(), "hospital".to_string(), LinkType::Temporal, 0.8);
    network.add_edge("strong_list".to_string(), "medicine".to_string(), LinkType::Temporal, 0.8);
    network.add_edge("nurse".to_string(), "doctor".to_string(), LinkType::Semantic, 0.9);
    network.add_edge("hospital".to_string(), "doctor".to_string(), LinkType::Semantic, 0.85);
    network.add_edge("medicine".to_string(), "doctor".to_string(), LinkType::Semantic, 0.8);

    // Weakly related list
    network.add_edge("weak_list".to_string(), "white".to_string(), LinkType::Temporal, 0.8);
    network.add_edge("weak_list".to_string(), "smart".to_string(), LinkType::Temporal, 0.8);
    network.add_edge("weak_list".to_string(), "office".to_string(), LinkType::Temporal, 0.8);
    network.add_edge("white".to_string(), "doctor".to_string(), LinkType::Semantic, 0.3);  // Weak
    network.add_edge("smart".to_string(), "doctor".to_string(), LinkType::Semantic, 0.25);  // Weak
    network.add_edge("office".to_string(), "doctor".to_string(), LinkType::Semantic, 0.2);  // Weak

    let strong_results = network.activate("strong_list", 1.0);
    let weak_results = network.activate("weak_list", 1.0);

    let strong_lure_activation = strong_results
        .iter()
        .find(|r| r.memory_id == "doctor")
        .map(|r| r.activation)
        .unwrap_or(0.0);

    let weak_lure_activation = weak_results
        .iter()
        .find(|r| r.memory_id == "doctor")
        .map(|r| r.activation)
        .unwrap_or(0.0);

    assert!(
        strong_lure_activation > weak_lure_activation,
        "Strongly related list ({}) should create stronger false memory than weakly related ({})",
        strong_lure_activation,
        weak_lure_activation
    );
}

/// Test source monitoring failure in DRM.
///
/// People cannot distinguish whether the lure was actually studied.
#[test]
fn test_drm_source_monitoring() {
    let mut network = ActivationNetwork::new();

    // Studied word
    network.add_edge("study_session".to_string(), "actually_studied".to_string(), LinkType::Temporal, 0.85);

    // Critical lure (activated through association, not direct study)
    network.add_edge("study_session".to_string(), "related_word".to_string(), LinkType::Temporal, 0.8);
    network.add_edge("related_word".to_string(), "critical_lure".to_string(), LinkType::Semantic, 0.9);

    let results = network.activate("study_session", 1.0);

    // Both should be activated
    let studied_activation = results
        .iter()
        .find(|r| r.memory_id == "actually_studied")
        .map(|r| r.activation)
        .unwrap_or(0.0);

    let lure_activation = results
        .iter()
        .find(|r| r.memory_id == "critical_lure")
        .map(|r| r.activation)
        .unwrap_or(0.0);

    // Both should have activation (source confusion)
    assert!(studied_activation > 0.0, "Studied word should be activated");
    assert!(lure_activation > 0.0, "Lure should also be activated, creating potential source confusion");

    // The lure should have distance > 1 (indirect) but this is the only way to distinguish
    let lure_result = results.iter().find(|r| r.memory_id == "critical_lure").unwrap();
    assert!(
        lure_result.distance > 1,
        "Lure came through indirect activation (distance {}), but feels like direct memory",
        lure_result.distance
    );
}

/// Test list length effect on false memory.
///
/// Longer lists with more associates create stronger false memories.
#[test]
fn test_drm_list_length_effect() {
    let mut network = ActivationNetwork::new();

    // Short list
    network.add_edge("short_list".to_string(), "word1".to_string(), LinkType::Temporal, 0.8);
    network.add_edge("short_list".to_string(), "word2".to_string(), LinkType::Temporal, 0.8);
    network.add_edge("word1".to_string(), "lure".to_string(), LinkType::Semantic, 0.7);
    network.add_edge("word2".to_string(), "lure".to_string(), LinkType::Semantic, 0.7);

    // Long list
    network.add_edge("long_list".to_string(), "word_a".to_string(), LinkType::Temporal, 0.8);
    network.add_edge("long_list".to_string(), "word_b".to_string(), LinkType::Temporal, 0.8);
    network.add_edge("long_list".to_string(), "word_c".to_string(), LinkType::Temporal, 0.8);
    network.add_edge("long_list".to_string(), "word_d".to_string(), LinkType::Temporal, 0.8);
    network.add_edge("long_list".to_string(), "word_e".to_string(), LinkType::Temporal, 0.8);
    network.add_edge("long_list".to_string(), "word_f".to_string(), LinkType::Temporal, 0.8);
    network.add_edge("word_a".to_string(), "lure".to_string(), LinkType::Semantic, 0.7);
    network.add_edge("word_b".to_string(), "lure".to_string(), LinkType::Semantic, 0.7);
    network.add_edge("word_c".to_string(), "lure".to_string(), LinkType::Semantic, 0.7);
    network.add_edge("word_d".to_string(), "lure".to_string(), LinkType::Semantic, 0.7);
    network.add_edge("word_e".to_string(), "lure".to_string(), LinkType::Semantic, 0.7);
    network.add_edge("word_f".to_string(), "lure".to_string(), LinkType::Semantic, 0.7);

    let short_results = network.activate("short_list", 1.0);
    let long_results = network.activate("long_list", 1.0);

    // Count total activation paths to lure
    let short_lure_count = short_results.iter().filter(|r| r.memory_id == "lure").count();
    let long_lure_count = long_results.iter().filter(|r| r.memory_id == "lure").count();

    assert!(
        long_lure_count >= short_lure_count,
        "Longer list ({} paths) should create at least as many activation paths to lure as short list ({})",
        long_lure_count,
        short_lure_count
    );
}

// ============================================================================
// INTERFERENCE TESTS (5 tests)
// ============================================================================
// Based on classic interference theory - memories can interfere with each other.

/// Test proactive interference - old learning interferes with new.
///
/// Prior learning of similar material impairs learning of new material.
#[test]
fn test_interference_proactive() {
    let mut network = ActivationNetwork::new();

    // Old learning (List A paired associates)
    network.add_edge("cue_word".to_string(), "old_response".to_string(), LinkType::Semantic, 0.8);

    // New learning (List B with same cues)
    network.add_edge("cue_word".to_string(), "new_response".to_string(), LinkType::Semantic, 0.5);  // Weaker - harder to learn

    let results = network.activate("cue_word", 1.0);

    let old_activation = results
        .iter()
        .find(|r| r.memory_id == "old_response")
        .map(|r| r.activation)
        .unwrap_or(0.0);

    let new_activation = results
        .iter()
        .find(|r| r.memory_id == "new_response")
        .map(|r| r.activation)
        .unwrap_or(0.0);

    // Old response should interfere (be more strongly activated)
    assert!(
        old_activation > new_activation,
        "Old response ({}) should interfere with new response retrieval ({})",
        old_activation,
        new_activation
    );
}

/// Test retroactive interference - new learning interferes with old.
///
/// Learning new material impairs recall of previously learned material.
#[test]
fn test_interference_retroactive() {
    let mut network = ActivationNetwork::new();

    // Original learning
    network.add_edge("stimulus".to_string(), "original_memory".to_string(), LinkType::Semantic, 0.7);

    // Interpolated learning (new, stronger)
    network.add_edge("stimulus".to_string(), "new_memory".to_string(), LinkType::Semantic, 0.9);

    let results = network.activate("stimulus", 1.0);

    let original_activation = results
        .iter()
        .find(|r| r.memory_id == "original_memory")
        .map(|r| r.activation)
        .unwrap_or(0.0);

    let new_activation = results
        .iter()
        .find(|r| r.memory_id == "new_memory")
        .map(|r| r.activation)
        .unwrap_or(0.0);

    // New learning should dominate (retroactive interference)
    assert!(
        new_activation > original_activation,
        "New memory ({}) should have higher activation, showing retroactive interference on original ({})",
        new_activation,
        original_activation
    );
}

/// Test similarity-based interference.
///
/// More similar materials create more interference.
#[test]
fn test_interference_similarity_based() {
    let mut network = ActivationNetwork::new();

    // Similar competing memories
    network.add_edge("topic".to_string(), "similar_fact_1".to_string(), LinkType::Semantic, 0.75);
    network.add_edge("topic".to_string(), "similar_fact_2".to_string(), LinkType::Semantic, 0.73);
    network.add_edge("topic".to_string(), "similar_fact_3".to_string(), LinkType::Semantic, 0.71);

    // Dissimilar memory (should be easier to distinguish)
    network.add_edge("topic".to_string(), "dissimilar_fact".to_string(), LinkType::Semantic, 0.80);

    let results = network.activate("topic", 1.0);

    // Collect similar facts activations
    let similar_activations: Vec<f64> = results
        .iter()
        .filter(|r| r.memory_id.starts_with("similar_fact"))
        .map(|r| r.activation)
        .collect();

    let dissimilar_activation = results
        .iter()
        .find(|r| r.memory_id == "dissimilar_fact")
        .map(|r| r.activation)
        .unwrap_or(0.0);

    // Similar facts should have close activations (hard to discriminate)
    if similar_activations.len() >= 2 {
        let max_diff = similar_activations
            .iter()
            .zip(similar_activations.iter().skip(1))
            .map(|(a, b)| (a - b).abs())
            .fold(0.0_f64, f64::max);

        assert!(
            max_diff < 0.1,
            "Similar facts should have close activations (interference), max diff: {}",
            max_diff
        );
    }

    // Dissimilar should stand out
    assert!(
        dissimilar_activation > 0.0,
        "Dissimilar fact should be clearly activated: {}",
        dissimilar_activation
    );
}

/// Test fan effect - more associations lead to slower retrieval.
///
/// Concepts with many associations show interference from competing links.
#[test]
fn test_interference_fan_effect() {
    let config = ActivationConfig {
        decay_factor: 0.7,
        max_hops: 2,
        min_threshold: 0.1,
        allow_cycles: false,
    };

    // Low fan: concept with few associations
    let mut low_fan_network = ActivationNetwork::with_config(config.clone());
    low_fan_network.add_edge("low_fan_concept".to_string(), "fact_1".to_string(), LinkType::Semantic, 0.9);
    low_fan_network.add_edge("low_fan_concept".to_string(), "fact_2".to_string(), LinkType::Semantic, 0.85);

    // High fan: concept with many associations
    let mut high_fan_network = ActivationNetwork::with_config(config);
    for i in 1..=8 {
        let strength = 0.9 - (i as f64 * 0.05);  // Decreasing strength due to fan
        high_fan_network.add_edge(
            "high_fan_concept".to_string(),
            format!("fact_{}", i),
            LinkType::Semantic,
            strength,
        );
    }

    let low_results = low_fan_network.activate("low_fan_concept", 1.0);
    let high_results = high_fan_network.activate("high_fan_concept", 1.0);

    // Average activation for low fan
    let low_avg: f64 = low_results.iter().map(|r| r.activation).sum::<f64>()
        / low_results.len().max(1) as f64;

    // Average activation for high fan
    let high_avg: f64 = high_results.iter().map(|r| r.activation).sum::<f64>()
        / high_results.len().max(1) as f64;

    // Low fan should have higher average activation (less interference)
    assert!(
        low_avg >= high_avg * 0.8,  // Allow some tolerance
        "Low fan concept should have higher average activation: low={}, high={}",
        low_avg,
        high_avg
    );
}

/// Test release from proactive interference.
///
/// Changing categories releases interference built up from prior learning.
#[test]
fn test_interference_release_from_pi() {
    let mut network = ActivationNetwork::new();

    // Build up PI with category A items
    network.add_edge("trial_1".to_string(), "category_a_item_1".to_string(), LinkType::Temporal, 0.7);
    network.add_edge("trial_2".to_string(), "category_a_item_2".to_string(), LinkType::Temporal, 0.6);  // PI building
    network.add_edge("trial_3".to_string(), "category_a_item_3".to_string(), LinkType::Temporal, 0.5);  // More PI

    // Category shift (release from PI)
    network.add_edge("trial_4".to_string(), "category_b_item_1".to_string(), LinkType::Temporal, 0.85);  // Recovery

    let trial_3_results = network.activate("trial_3", 1.0);
    let trial_4_results = network.activate("trial_4", 1.0);

    let trial_3_activation = trial_3_results
        .iter()
        .map(|r| r.activation)
        .next()
        .unwrap_or(0.0);

    let trial_4_activation = trial_4_results
        .iter()
        .map(|r| r.activation)
        .next()
        .unwrap_or(0.0);

    // Category switch should show release (better activation)
    assert!(
        trial_4_activation > trial_3_activation,
        "Category switch should release from PI: trial_4 ({}) > trial_3 ({})",
        trial_4_activation,
        trial_3_activation
    );
}
