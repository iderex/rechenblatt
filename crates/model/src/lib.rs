//! The workbook model.
//!
//! One model, read once, serving both tracks. A question about a document is
//! answered from here and never by opening the file a second time. This crate
//! depends on no other crate in the workspace, and that is the point of it:
//! everything downstream reads the model, and the model reads the document.
//!
//! It is empty. Milestone 2 fills it, starting at issue #14.

/// What this component is called where a diagnostic has to name one.
pub const COMPONENT: &str = "model";
