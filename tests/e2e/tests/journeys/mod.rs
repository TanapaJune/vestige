//! # User Journey E2E Tests
//!
//! Comprehensive end-to-end tests that validate complete user workflows
//! from start to finish. These tests ensure that the core user journeys
//! work correctly across all system components.
//!
//! ## Test Categories
//!
//! 1. **Ingest-Recall-Review**: Core memory lifecycle
//! 2. **Consolidation Workflow**: Sleep-inspired memory processing
//! 3. **Intentions Workflow**: Intent detection and memory relevance
//! 4. **Spreading Activation**: Associative memory retrieval
//! 5. **Import/Export**: Data portability and backup

pub mod consolidation_workflow;
pub mod import_export;
pub mod ingest_recall_review;
pub mod intentions_workflow;
pub mod spreading_activation;
