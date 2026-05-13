//! Compatibility re-export for the execution-shaped IR now owned by `mir`.
//!
//! Existing callers may still import `muga::hir`, but new backend work should
//! use `muga::mir` directly.

pub use crate::mir::*;
