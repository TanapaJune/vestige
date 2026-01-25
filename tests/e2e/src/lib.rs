//! E2E Test Infrastructure for Vestige
//!
//! Provides comprehensive testing utilities for 250+ end-to-end tests:
//!
//! - **Harness**: Test setup, time travel, database management
//! - **Mocks**: MockEmbeddingService (FxHash-based), test fixtures
//! - **Assertions**: Custom assertions for memory states, decay, etc.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use vestige_e2e_tests::prelude::*;
//!
//! #[test]
//! fn test_memory_decay() {
//!     let mut env = TimeTravelEnvironment::new();
//!     let mut db = TestDatabaseManager::new_temp();
//!
//!     // Create test data
//!     let node = TestDataFactory::create_memory(&mut db.storage, "test content");
//!
//!     // Time travel to test decay
//!     env.advance_days(30);
//!
//!     // Assert decay occurred
//!     assert_retention_decreased!(db.storage.get_node(&node.id), 0.9);
//! }
//! ```

pub mod assertions;
pub mod harness;
pub mod mocks;

// Re-export commonly used items
pub use harness::{TestDatabaseManager, TimeTravelEnvironment};
pub use mocks::{MockEmbeddingService, TestDataFactory};

/// Convenient imports for tests
pub mod prelude {
    pub use crate::assertions::*;
    pub use crate::harness::{TestDatabaseManager, TimeTravelEnvironment};
    pub use crate::mocks::{MockEmbeddingService, TestDataFactory};

    // Re-export vestige-core essentials
    pub use vestige_core::{
        FSRSScheduler, FSRSState, IngestInput, KnowledgeNode, NodeType, Rating, RecallInput,
        Result, SearchMode, Storage, StorageError,
    };
}
