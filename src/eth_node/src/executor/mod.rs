//! Revm-based transaction executor and state simulator.
//!
//! Phase 2 Track A: local transaction simulation using revm 14.x.
//! See PHASE2_FORMAL_SPEC.md FR-001 for behavioral contracts.

pub mod simulate;

pub use simulate::{ExecutorError, SimulationResult};
