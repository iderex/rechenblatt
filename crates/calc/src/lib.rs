//! Calculation.
//!
//! Reads the model and evaluates what the page needs. It depends on the model
//! and on nothing else in the workspace, so a change to how a document is drawn
//! cannot reach how a value is computed.
//!
//! It is empty. Milestone 5 fills it, starting at issue #44.

/// What this component is called where a diagnostic has to name one.
pub const COMPONENT: &str = "calc";
