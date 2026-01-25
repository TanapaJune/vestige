//! Test Harness Module
//!
//! Provides test setup utilities:
//! - `TimeTravelEnvironment` for testing time-dependent behavior (decay, scheduling)
//! - `TestDatabaseManager` for isolated test databases

mod db_manager;
mod time_travel;

pub use db_manager::TestDatabaseManager;
pub use time_travel::TimeTravelEnvironment;
