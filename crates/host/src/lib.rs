//! The host side of the input boundary.
//!
//! Opening a path, making a network call and reading a clock happen here and
//! nowhere else. The components that read bytes this project did not create sit
//! on the other side and are handed what they need: a slice of bytes, a ceiling,
//! a value that was already read. They never reach back for more.
//!
//! That is what makes a fuzz target a wrapper instead of a rewrite. A parser
//! that takes a path has to be given a filesystem before it can be fuzzed at
//! all; one that takes bytes is already fuzzable, and issue #96 is where that is
//! collected on.
//!
//! It is empty. Milestone 9 fills it, starting at issue #77, and the components
//! that will hand it work are named in `docs/decisions/0014-input-boundary.md`.

/// What this component is called where a diagnostic has to name one.
pub const COMPONENT: &str = "host";
