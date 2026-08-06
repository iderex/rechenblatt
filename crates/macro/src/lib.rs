//! The macro compatibility track.
//!
//! Reads and changes a workbook through the model, and evaluates through the
//! calculation engine. It does not depend on the renderer: a macro that changes
//! a workbook is scored by rendering the result, which happens above this crate
//! rather than inside it. `docs/decisions/0002-track-order.md` is where that was
//! decided.
//!
//! It is empty. Milestone 8 fills it, starting at issue #67.

/// What this component is called where a diagnostic has to name one.
pub const COMPONENT: &str = "macro";
