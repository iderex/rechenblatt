//! The rendering engine.
//!
//! Turns a model into geometry and then into output. It reads the model and the
//! calculated values and is read by nothing except the operator-facing binary,
//! so the renderer can be replaced without the model moving.
//!
//! It is empty. Milestone 4 fills it, starting at issue #34.

/// What this component is called where a diagnostic has to name one.
pub const COMPONENT: &str = "render";
