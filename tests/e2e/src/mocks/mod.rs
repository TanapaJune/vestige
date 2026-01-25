//! Mock Services Module
//!
//! Provides mock implementations for testing:
//! - `MockEmbeddingService` - Deterministic embeddings using FxHash
//! - `TestDataFactory` - Generate test data with realistic properties

mod fixtures;
mod mock_embedding;

pub use fixtures::TestDataFactory;
pub use mock_embedding::MockEmbeddingService;
