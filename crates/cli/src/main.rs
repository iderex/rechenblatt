//! The command an operator drives.
//!
//! One component among five rather than the whole tree. Everything this binary
//! can do lives in a library crate beside it, so the engine is usable without
//! the command and the command is replaceable without the engine.
//!
//! It does nothing yet. Milestone 9 gives it something to do, starting at issue
//! #77, and `docs/decisions/0012-operator-surface.md` is where the shape of it
//! was decided.

/// The components this binary was built from, in workspace order.
///
/// Every one of them is a crate this package depends on, so the list cannot name
/// something that is not linked in. The test below is the other direction: it
/// cannot silently stop naming something the workspace does carry.
pub fn components() -> [&'static str; 5] {
    [
        rechenblatt_model::COMPONENT,
        rechenblatt_calc::COMPONENT,
        rechenblatt_render::COMPONENT,
        rechenblatt_macro::COMPONENT,
        rechenblatt_host::COMPONENT,
    ]
}

fn main() {
    println!("rechenblatt {}", env!("CARGO_PKG_VERSION"));
    println!("components: {}", components().join(", "));
    println!("There is nothing to run yet. See https://github.com/iderex/rechenblatt");
}

#[cfg(test)]
mod tests {
    use super::components;

    /// The workspace manifest, read at compile time from the tree rather than
    /// from a copy of it.
    const WORKSPACE_MANIFEST: &str = include_str!("../../../Cargo.toml");

    /// Every library component in the workspace is named by the binary.
    ///
    /// A crate added to `members` and wired into nothing is the failure this
    /// catches: it builds, it passes its own tests, and no operator-facing thing
    /// knows it exists. The binary itself is a member and is not a component, so
    /// it is the one exclusion and it is named here rather than assumed.
    #[test]
    fn every_workspace_library_is_a_named_component() {
        let members: Vec<&str> = WORKSPACE_MANIFEST
            .lines()
            .filter_map(|line| line.trim().strip_prefix("\"crates/"))
            .filter_map(|rest| rest.split('"').next())
            .filter(|name| *name != "cli")
            .collect();

        for member in &members {
            assert!(
                components().contains(member),
                "the workspace carries crates/{member} and the binary does not name it"
            );
        }

        // The loop above cannot see a member the parse failed to read, so the
        // count is checked as well. A manifest this test reads as empty would
        // otherwise pass it.
        assert_eq!(
            members.len(),
            components().len(),
            "read {members:?} out of the workspace manifest, against the {} \
             component(s) the binary names",
            components().len()
        );
    }

    /// Two components answering to one name would make every diagnostic that
    /// quotes one ambiguous.
    #[test]
    fn component_names_are_distinct() {
        let mut seen = components();
        seen.sort_unstable();
        let before = seen.len();
        let mut deduped = seen.to_vec();
        deduped.dedup();
        assert_eq!(
            deduped.len(),
            before,
            "duplicate component name in {seen:?}"
        );
    }
}
