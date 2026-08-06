//! Nothing that reads bytes a stranger wrote may depend on something that can
//! open a path, make a network call or read a clock.
//!
//! This project reads documents it did not create. The code that walks those
//! bytes is where a malformed file turns into somebody else's problem, so it is
//! held on one side of a line and handed what it needs rather than reaching for
//! it. A parser that takes a byte slice and returns a value or a typed error is
//! a fuzz target with a wrapper around it; one that opens a path and logs as it
//! goes is a rewrite away from being fuzzable at all, and finding that out after
//! it is written costs the rewrite.
//!
//! Each member of the workspace declares its side in its own manifest:
//!
//! ```toml
//! [package.metadata.rechenblatt]
//! side = "pure"   # or "host"
//! ```
//!
//! `docs/decisions/0014-input-boundary.md` is where the split is argued and
//! where what each side may do is written out.
//!
//! What this reads and what it therefore cannot see. It reads the declared
//! dependencies in the manifests: `[dependencies]` and `[build-dependencies]`,
//! which are what a compiled component links against. It does not read
//! `[dev-dependencies]`, because a test is not the component: a pure crate's own
//! tests read the filesystem to build a fixture, and that is not the capability
//! this line is about. It does not read source, so a pure crate calling
//! `std::fs` directly passes here; the compiler offers no lint for that and the
//! honest answer is that this check does not cover it. Issue #101 is where the
//! rest of the architecture becomes tests.
//!
//! This test lives in the command's crate rather than in a parsing one because
//! it walks the tree, and the command is on the side that is allowed to.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// The two sides. A member declaring anything else is refused rather than
/// guessed at, because a typo that reads as "not host" would open the boundary
/// silently.
const PURE: &str = "pure";
const HOST: &str = "host";

/// What a workspace can be wrong about, one variant per refusal.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Problem {
    /// A member with no `[package.metadata.rechenblatt] side`.
    SideNotDeclared(String),
    /// A member declaring a side that is neither of the two.
    UnknownSide(String, String),
    /// A parsing component depending on one that may reach a capability.
    PureReachesHost(String, String),
    /// A parsing component depending on a crate from outside the workspace that
    /// the workspace has not accepted for that side.
    PureReachesUnreviewed(String, String),
}

impl Problem {
    /// The line a failing run prints. It names the edge, so the cause is
    /// locatable from the output alone, and it ends in the repair.
    fn describe(&self) -> String {
        match self {
            Problem::SideNotDeclared(member) => format!(
                "{member} does not say which side of the input boundary it is on. Add \
                 [package.metadata.rechenblatt] with side = \"{PURE}\" or side = \"{HOST}\" \
                 to its manifest."
            ),
            Problem::UnknownSide(member, side) => format!(
                "{member} declares side = \"{side}\", which is neither \"{PURE}\" nor \
                 \"{HOST}\". A side this check cannot read is one it cannot enforce."
            ),
            Problem::PureReachesHost(from, to) => format!(
                "{from} -> {to} crosses the input boundary: {from} reads bytes it did not \
                 create and {to} may open a path, make a network call or read a clock. Hand \
                 {from} the bytes instead of the capability."
            ),
            Problem::PureReachesUnreviewed(from, to) => format!(
                "{from} -> {to} leaves the workspace, and {to} is not in \
                 pure-may-depend-on in the workspace manifest. Read what {to} can reach, \
                 then add it there in the commit that needs it."
            ),
        }
    }
}

/// One member's manifest, as much of it as this check reads.
struct Member {
    /// The package name, which is what a dependency line names.
    name: String,
    /// The declared side, absent where the manifest does not declare one.
    side: Option<String>,
    /// The crates this member links against.
    deps: Vec<String>,
}

/// The value of `key = "..."` on a line, where the line sets that key.
fn string_value(line: &str, key: &str) -> Option<String> {
    let rest = line.strip_prefix(key)?;
    let rest = rest.trim_start();
    let rest = rest.strip_prefix('=')?.trim();
    let inner = rest.strip_prefix('"')?;
    let end = inner.find('"')?;
    Some(inner[..end].to_owned())
}

/// The quoted strings in the array opened by `key = [`, which may be on one line
/// or spread over several.
fn string_array(text: &str, key: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut inside = false;
    for raw in text.lines() {
        let line = raw.trim();
        if line.starts_with('#') {
            continue;
        }
        let body = if inside {
            line
        } else if line.starts_with(key) && line.contains('[') {
            inside = true;
            match line.split_once('[') {
                Some((_, after)) => after,
                None => continue,
            }
        } else {
            continue;
        };
        let mut rest = body;
        while let Some(open) = rest.find('"') {
            let after = &rest[open + 1..];
            match after.find('"') {
                Some(close) => {
                    found.push(after[..close].to_owned());
                    rest = &after[close + 1..];
                }
                None => break,
            }
        }
        if body.contains(']') {
            break;
        }
    }
    found
}

/// Reads the parts of a member manifest this check judges.
///
/// The parse is deliberately small: sections by their header line, values by
/// their key. A manifest whose dependency table spreads one entry over several
/// lines would be read as more entries than it has, which the guard on the key
/// shape below turns into nothing rather than into a wrong name.
fn read_member(text: &str) -> Member {
    let mut section = String::new();
    let mut name = String::new();
    let mut side = None;
    let mut deps = Vec::new();

    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(header) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            section = header.to_owned();
            continue;
        }
        match section.as_str() {
            "package" => {
                if let Some(value) = string_value(line, "name") {
                    name = value;
                }
            }
            "package.metadata.rechenblatt" => {
                if let Some(value) = string_value(line, "side") {
                    side = Some(value);
                }
            }
            "dependencies" | "build-dependencies" => {
                if !line.contains('=') {
                    continue;
                }
                let key = line.split(['.', '=', ' ']).next().unwrap_or("").trim();
                let shaped = !key.is_empty()
                    && key
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
                if shaped {
                    deps.push(key.to_owned());
                }
            }
            _ => {}
        }
    }

    Member { name, side, deps }
}

/// Reads a workspace and returns everything wrong with its boundary, sorted.
///
/// A root that cannot be read, or that declares no members, panics rather than
/// returning an empty list. A silent pass over a workspace this could not see is
/// the one outcome worse than a refusal.
fn problems(root: &Path) -> Vec<Problem> {
    let manifest = root.join("Cargo.toml");
    let text = fs::read_to_string(&manifest)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", manifest.display()));

    let members = string_array(&text, "members");
    assert!(
        !members.is_empty(),
        "{} declares no workspace members; refusing to report a clean tree",
        manifest.display()
    );
    let reviewed = string_array(&text, "pure-may-depend-on");

    let mut read: Vec<(String, Member)> = Vec::new();
    for member in &members {
        let path = root.join(member).join("Cargo.toml");
        let body = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
        read.push((member.clone(), read_member(&body)));
    }

    let sides: BTreeMap<&str, &str> = read
        .iter()
        .filter_map(|(_, m)| m.side.as_deref().map(|side| (m.name.as_str(), side)))
        .collect();

    let mut found = Vec::new();
    for (member, parsed) in &read {
        let side = match parsed.side.as_deref() {
            None => {
                found.push(Problem::SideNotDeclared(member.clone()));
                continue;
            }
            Some(side) if side != PURE && side != HOST => {
                found.push(Problem::UnknownSide(member.clone(), side.to_owned()));
                continue;
            }
            Some(side) => side,
        };
        if side != PURE {
            continue;
        }
        for dep in &parsed.deps {
            match sides.get(dep.as_str()) {
                Some(&HOST) => {
                    found.push(Problem::PureReachesHost(parsed.name.clone(), dep.clone()));
                }
                Some(_) => {}
                None if reviewed.iter().any(|name| name == dep) => {}
                None => {
                    found.push(Problem::PureReachesUnreviewed(
                        parsed.name.clone(),
                        dep.clone(),
                    ));
                }
            }
        }
    }

    found.sort();
    found
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .to_path_buf()
}

#[test]
fn the_workspace_holds_the_input_boundary() {
    let root = workspace_root();
    let found = problems(&root);
    assert!(
        found.is_empty(),
        "the input boundary does not hold:\n{}",
        found
            .iter()
            .map(|problem| format!("  {}", problem.describe()))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn every_member_declares_a_side_and_at_least_one_of_each_exists() {
    let root = workspace_root();
    let text = fs::read_to_string(root.join("Cargo.toml")).expect("cannot read the workspace");
    let mut pure = 0;
    let mut host = 0;
    for member in string_array(&text, "members") {
        let body = fs::read_to_string(root.join(&member).join("Cargo.toml"))
            .unwrap_or_else(|error| panic!("cannot read {member}: {error}"));
        match read_member(&body).side.as_deref() {
            Some(PURE) => pure += 1,
            Some(HOST) => host += 1,
            other => panic!("{member} declares side {other:?}"),
        }
    }
    // A boundary with nothing on one side of it is a boundary that cannot be
    // crossed, which would make every leg below pass for the wrong reason.
    assert!(pure > 0, "no component is on the parsing side");
    assert!(host > 0, "no component is on the host side");
}

/// A scratch workspace that removes itself, so the legs below leave nothing
/// behind and need no dependency to build one.
struct Scratch(PathBuf);

impl Scratch {
    fn new(label: &str, members: &[&str], reviewed: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "rechenblatt-boundary-{}-{label}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("cannot create a scratch directory");
        let listed = members
            .iter()
            .map(|name| format!("  \"crates/{name}\",\n"))
            .collect::<String>();
        fs::write(
            dir.join("Cargo.toml"),
            format!(
                "[workspace]\nmembers = [\n{listed}]\n\n\
                 [workspace.metadata.rechenblatt]\npure-may-depend-on = [{reviewed}]\n"
            ),
        )
        .expect("cannot write the scratch workspace");
        Scratch(dir)
    }

    /// A member with a declared side and a dependency table holding `deps`.
    fn member(&self, name: &str, side: Option<&str>, deps: &[&str]) -> &Self {
        let dir = self.0.join("crates").join(name);
        fs::create_dir_all(&dir).expect("cannot create a scratch member");
        let declared = match side {
            Some(side) => format!("\n[package.metadata.rechenblatt]\nside = \"{side}\"\n"),
            None => String::new(),
        };
        let listed = deps
            .iter()
            .map(|dep| format!("{dep}.workspace = true\n"))
            .collect::<String>();
        fs::write(
            dir.join("Cargo.toml"),
            format!(
                "[package]\nname = \"rechenblatt-{name}\"\n{declared}\n[dependencies]\n{listed}"
            ),
        )
        .expect("cannot write a scratch member");
        self
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

// The legs. Each one builds a workspace holding exactly the thing its property
// is about and requires that property and no other, and the neighbouring leg
// changes the one thing back and requires nothing at all. A check that refuses
// everything fails the second kind; one that refuses nothing fails the first.
//
// Every leg carries both sides of the boundary, so the workspace under test
// differs from a correct one in exactly one thing.

#[test]
fn a_parsing_component_depending_on_a_capability_is_refused() {
    let scratch = Scratch::new("pure-reaches-host", &["model", "host"], "");
    scratch
        .member("model", Some(PURE), &["rechenblatt-host"])
        .member("host", Some(HOST), &[]);
    assert_eq!(
        problems(&scratch.0),
        vec![Problem::PureReachesHost(
            "rechenblatt-model".into(),
            "rechenblatt-host".into()
        )],
        "a deliberate dependency from a parsing component onto the filesystem \
         capability has to make this red"
    );
}

#[test]
fn the_same_pair_without_that_dependency_is_not_refused() {
    let scratch = Scratch::new("pure-clean", &["model", "host"], "");
    scratch
        .member("model", Some(PURE), &[])
        .member("host", Some(HOST), &[]);
    assert_eq!(problems(&scratch.0), vec![]);
}

#[test]
fn a_capability_component_depending_on_a_parsing_one_is_not_refused() {
    let scratch = Scratch::new("host-reaches-pure", &["model", "host"], "");
    scratch
        .member("model", Some(PURE), &[])
        .member("host", Some(HOST), &["rechenblatt-model"]);
    assert_eq!(
        problems(&scratch.0),
        vec![],
        "the boundary has a direction; the host side reading the model is the \
         direction it is supposed to run in"
    );
}

#[test]
fn a_parsing_component_depending_on_another_parsing_one_is_not_refused() {
    let scratch = Scratch::new("pure-reaches-pure", &["model", "calc", "host"], "");
    scratch
        .member("model", Some(PURE), &[])
        .member("calc", Some(PURE), &["rechenblatt-model"])
        .member("host", Some(HOST), &[]);
    assert_eq!(problems(&scratch.0), vec![]);
}

#[test]
fn a_parsing_component_depending_on_an_unreviewed_outside_crate_is_refused() {
    let scratch = Scratch::new("pure-reaches-outside", &["model", "host"], "");
    scratch
        .member("model", Some(PURE), &["some-zip-reader"])
        .member("host", Some(HOST), &[]);
    assert_eq!(
        problems(&scratch.0),
        vec![Problem::PureReachesUnreviewed(
            "rechenblatt-model".into(),
            "some-zip-reader".into()
        )]
    );
}

#[test]
fn the_same_outside_crate_once_the_workspace_accepts_it_is_not_refused() {
    let scratch = Scratch::new(
        "pure-reaches-reviewed",
        &["model", "host"],
        "\"some-zip-reader\"",
    );
    scratch
        .member("model", Some(PURE), &["some-zip-reader"])
        .member("host", Some(HOST), &[]);
    assert_eq!(problems(&scratch.0), vec![]);
}

#[test]
fn a_capability_component_depending_on_an_unreviewed_outside_crate_is_not_refused() {
    let scratch = Scratch::new("host-reaches-outside", &["model", "host"], "");
    scratch
        .member("model", Some(PURE), &[])
        .member("host", Some(HOST), &["some-http-client"]);
    assert_eq!(
        problems(&scratch.0),
        vec![],
        "the list bounds what a parser may link against, and the host side is \
         where a capability is allowed to come from"
    );
}

#[test]
fn a_member_that_declares_no_side_is_refused() {
    let scratch = Scratch::new("no-side", &["model", "host", "drawing"], "");
    scratch
        .member("model", Some(PURE), &[])
        .member("host", Some(HOST), &[])
        .member("drawing", None, &[]);
    assert_eq!(
        problems(&scratch.0),
        vec![Problem::SideNotDeclared("crates/drawing".into())],
        "a component added without a side is the way this boundary erodes, so it \
         is refused rather than assumed to be pure"
    );
}

#[test]
fn the_same_member_with_a_side_is_not_refused() {
    let scratch = Scratch::new("side-back", &["model", "host", "drawing"], "");
    scratch
        .member("model", Some(PURE), &[])
        .member("host", Some(HOST), &[])
        .member("drawing", Some(PURE), &[]);
    assert_eq!(problems(&scratch.0), vec![]);
}

#[test]
fn a_member_declaring_a_side_this_check_cannot_read_is_refused() {
    let scratch = Scratch::new("unknown-side", &["model", "host", "drawing"], "");
    scratch
        .member("model", Some(PURE), &[])
        .member("host", Some(HOST), &[])
        .member("drawing", Some("puer"), &[]);
    assert_eq!(
        problems(&scratch.0),
        vec![Problem::UnknownSide("crates/drawing".into(), "puer".into())],
        "a typo that reads as `not host` would open the boundary without saying so"
    );
}
