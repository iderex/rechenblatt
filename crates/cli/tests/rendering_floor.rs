//! A crate the rendering pipeline links against is one the rendering decision
//! has judged.
//!
//! `docs/decisions/0006-rendering.md` states the rule this refuses the violation
//! of: the pipeline holds no document semantics in a dependency. If swapping a
//! library could change what a document is understood to say, the library is on
//! the wrong side of that line and the semantics belong in this repository. The
//! record then names the libraries the pipeline's floor is made of, one at a
//! time, with what each may decide and what it may not.
//!
//! That judgement is the thing being enforced here. It is made by a person
//! reading a crate and writing down what it may decide, and it is made before
//! the dependency exists rather than after a fidelity difference traces into it.
//! What this refuses is the commit that skips it: a crate appearing in the
//! pipeline's manifest that the record has never been asked about.
//!
//! It sits beside two checks that read the same manifests for different things
//! and would both pass such a commit. `crates/cli/tests/boundary.rs` asks
//! whether somebody has read what a crate can reach, which is a question about
//! capability. `crates/cli/tests/component_edges.rs` asks which components may
//! depend on which, and says nothing about crates from outside the workspace. A
//! shaping library that decides which font a cell asked for crosses neither: it
//! reaches no capability and it is not a component. Both lists have to hold it
//! before it links, and this is the second of the two.
//!
//! What this reads. The pipeline component's `[dependencies]` and
//! `[build-dependencies]`, which are what it links against, and the section of
//! the record that says what each library may decide. It does not read
//! `[dev-dependencies]`, on the line `crates/cli/tests/boundary.rs` draws and
//! for the same reason: a test is not the component.
//!
//! Where it stops, stated rather than left to be discovered.
//!
//! It reads the record as text and asks whether the crate's name is written in
//! that section. It cannot tell a name written as a judgement from one written
//! in passing, so a crate whose name is an ordinary word already in the prose
//! would pass. What it refuses is the name that is not there at all, which is
//! what the commit it is aimed at looks like, and the review is where the weaker
//! case is caught.
//!
//! Its subject is the rendering pipeline, because the rule it enforces is the
//! rendering decision's. `crates/model`, `crates/calc` and `crates/macro` carry
//! document semantics too and are not judged here; the records that govern them
//! name no floor of their own, and until one does there is nothing for a check
//! to read. `docs/architecture.md` is where that is written down beside the
//! rules a machine does hold.
//!
//! This test lives in the command's crate rather than in the pipeline's because
//! it walks the tree, and the command is on the side of the input boundary that
//! is allowed to.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// The package name of the component the rendering decision is about.
///
/// It is the package name rather than the directory, so a member renamed on
/// disk without its manifest changing does not quietly stop being the pipeline
/// and take this rule with it.
const PIPELINE: &str = "rechenblatt-render";

/// The record that admits a crate to the pipeline's floor.
const RECORD: &str = "docs/decisions/0006-rendering.md";

/// The heading of the section in that record which says what each library may
/// decide. A crate named anywhere else in the record has been mentioned; a crate
/// named here has been judged.
const SECTION: &str = "### The libraries, and what each may decide";

/// What a workspace and its record can be wrong about, one variant per refusal.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Problem {
    /// The pipeline links a crate from outside the workspace that the record's
    /// section does not name.
    NotAdmitted(String),
    /// No member of the workspace answers to the pipeline's name.
    ThePipelineIsNotAMember(String),
    /// The record is there and the section is not.
    TheRecordHasNoSection(String),
}

impl Problem {
    /// The line a failing run prints. It names the crate, so the cause is
    /// locatable from the output alone, and it ends in the repair.
    fn describe(&self) -> String {
        match self {
            Problem::NotAdmitted(name) => format!(
                "{PIPELINE} links `{name}`, and {RECORD} does not name it under \
                 \"{SECTION}\". Read what {name} decides, and if it decides nothing about \
                 what a document means, write that in the record in the commit that adds \
                 the dependency. If it does decide something, the semantics belong in this \
                 repository and the dependency does not."
            ),
            Problem::ThePipelineIsNotAMember(name) => format!(
                "crates/cli/tests/rendering_floor.rs names {name} as the rendering \
                 pipeline, and no member of this workspace is called that. A pipeline this \
                 check cannot find is one whose floor nothing judges, so it is refused \
                 rather than reported as clean."
            ),
            Problem::TheRecordHasNoSection(heading) => format!(
                "{RECORD} does not carry the heading \"{heading}\". That section is where a \
                 library is admitted to the pipeline's floor, and a check that cannot find \
                 it cannot tell an admitted crate from an unread one. Restore the heading, \
                 or point this check at the one that replaced it."
            ),
        }
    }
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

/// One member manifest, as much of it as this check reads.
struct Member {
    /// The package name, which is what a dependency line names.
    name: String,
    /// The crates this member links against.
    deps: Vec<String>,
}

/// Reads the parts of a member manifest this check judges.
///
/// The parse is the same small one the two dependency checks beside this make:
/// sections by their header line, values by their key, and a dependency key that
/// is not shaped like a crate name is dropped rather than reported under a wrong
/// one.
fn read_member(text: &str) -> Member {
    let mut section = String::new();
    let mut name = String::new();
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

    Member { name, deps }
}

/// The tokens of a piece of text, split on everything a crate name cannot hold.
///
/// A crate name is letters, digits, hyphens and underscores, so a name is
/// present when it survives that split as a whole token. `image` in a sentence
/// is the same token as `image` in the licence transcript, which is the bound
/// the module comment above discloses.
fn tokens(text: &str) -> BTreeSet<String> {
    text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'))
        .filter(|token| !token.is_empty())
        .map(str::to_owned)
        .collect()
}

/// The body of the record's section, from its heading to the next heading at the
/// same level or above, and `None` where the heading is not there.
fn admitting_section(record: &str) -> Option<String> {
    let mut body = String::new();
    let mut inside = false;
    for line in record.lines() {
        if line.trim_end() == SECTION {
            inside = true;
            continue;
        }
        if inside {
            let trimmed = line.trim_start();
            let ends = trimmed.starts_with("### ")
                || trimmed.starts_with("## ")
                || trimmed.starts_with("# ");
            if ends {
                break;
            }
            body.push_str(line);
            body.push('\n');
        }
    }
    inside.then_some(body)
}

/// Reads a tree and returns everything wrong with the pipeline's floor, sorted.
///
/// A workspace or a record that cannot be read panics rather than returning an
/// empty list. A silent pass over a tree this could not see is the one outcome
/// worse than a refusal.
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

    let record_path = root.join(RECORD);
    let record = fs::read_to_string(&record_path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", record_path.display()));

    let mut read: Vec<Member> = Vec::new();
    for member in &members {
        let path = root.join(member).join("Cargo.toml");
        let body = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
        read.push(read_member(&body));
    }

    let names: BTreeSet<&str> = read.iter().map(|m| m.name.as_str()).collect();

    let mut found = Vec::new();
    let Some(pipeline) = read.iter().find(|m| m.name == PIPELINE) else {
        found.push(Problem::ThePipelineIsNotAMember(PIPELINE.to_owned()));
        return found;
    };

    let Some(section) = admitting_section(&record) else {
        found.push(Problem::TheRecordHasNoSection(SECTION.to_owned()));
        return found;
    };

    let admitted = tokens(&section);
    for dep in &pipeline.deps {
        if names.contains(dep.as_str()) {
            continue;
        }
        if !admitted.contains(dep) {
            found.push(Problem::NotAdmitted(dep.clone()));
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
fn the_pipeline_links_only_what_the_rendering_record_admits() {
    let root = workspace_root();
    let found = problems(&root);
    assert!(
        found.is_empty(),
        "the rendering floor does not hold:\n{}",
        found
            .iter()
            .map(|problem| format!("  {}", problem.describe()))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// A scratch tree that removes itself, so the legs below leave nothing behind
/// and need no dependency to build one.
struct Scratch(PathBuf);

impl Scratch {
    /// A workspace holding `members`, beside a record whose body is `record`.
    fn new(label: &str, members: &[&str], record: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "rechenblatt-rendering-floor-{}-{label}",
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
            format!("[workspace]\nmembers = [\n{listed}]\n"),
        )
        .expect("cannot write the scratch workspace");
        let record_path = dir.join(RECORD);
        fs::create_dir_all(
            record_path
                .parent()
                .expect("the record path has a directory"),
        )
        .expect("cannot create the scratch record directory");
        fs::write(&record_path, record).expect("cannot write the scratch record");
        Scratch(dir)
    }

    /// A member with a dependency table holding `deps`.
    fn member(&self, name: &str, deps: &[&str]) -> &Self {
        let dir = self.0.join("crates").join(name);
        fs::create_dir_all(&dir).expect("cannot create a scratch member");
        let listed = deps
            .iter()
            .map(|dep| format!("{dep}.workspace = true\n"))
            .collect::<String>();
        fs::write(
            dir.join("Cargo.toml"),
            format!("[package]\nname = \"rechenblatt-{name}\"\n\n[dependencies]\n{listed}"),
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

/// A record whose admitting section names `admitted`, with a paragraph before it
/// so that a leg can plant a name outside the section as well as inside it.
fn record_admitting(before: &str, admitted: &str) -> String {
    format!(
        "# 0006 Rendering\n\n## The decision, in full\n\n{before}\n\n\
         {SECTION}\n\n**Shaping.** {admitted} turns a run of text into positioned \
         glyphs and decides nothing about what a document means.\n\n\
         ### The stages between the model and the output\n\n1. Resolve.\n"
    )
}

// The legs. Each one plants exactly one thing in a scratch tree and requires
// that refusal and no other; each has a neighbour that changes the one thing
// back and requires nothing to be refused. A check that refuses everything fails
// the second kind; one that refuses nothing fails the first.

#[test]
fn a_crate_the_record_does_not_name_is_refused() {
    let scratch = Scratch::new(
        "not-admitted",
        &["model", "render"],
        &record_admitting("Own the pipeline.", "rustybuzz"),
    );
    scratch
        .member("model", &[])
        .member("render", &["some-layout-engine"]);
    assert_eq!(
        problems(&scratch.0),
        vec![Problem::NotAdmitted("some-layout-engine".into())],
        "a library linked into the pipeline without the record being asked what \
         it decides has to make this red"
    );
}

#[test]
fn the_same_crate_once_the_record_names_it_is_not_refused() {
    let scratch = Scratch::new(
        "admitted",
        &["model", "render"],
        &record_admitting("Own the pipeline.", "some-layout-engine"),
    );
    scratch
        .member("model", &[])
        .member("render", &["some-layout-engine"]);
    assert_eq!(problems(&scratch.0), vec![]);
}

#[test]
fn a_crate_named_in_the_record_outside_that_section_is_refused() {
    let scratch = Scratch::new(
        "named-elsewhere",
        &["model", "render"],
        &record_admitting(
            "Own the pipeline rather than wrap some-layout-engine.",
            "rustybuzz",
        ),
    );
    scratch
        .member("model", &[])
        .member("render", &["some-layout-engine"]);
    assert_eq!(
        problems(&scratch.0),
        vec![Problem::NotAdmitted("some-layout-engine".into())],
        "a crate mentioned in the argument is not a crate the record has said \
         what may decide, and a rejected alternative is named in a record too"
    );
}

#[test]
fn the_pipeline_depending_on_another_member_is_not_refused() {
    let scratch = Scratch::new(
        "member-edge",
        &["model", "render"],
        &record_admitting("Own the pipeline.", "rustybuzz"),
    );
    scratch
        .member("model", &[])
        .member("render", &["rechenblatt-model"]);
    assert_eq!(
        problems(&scratch.0),
        vec![],
        "which component may depend on which is crates/cli/tests/component_edges.rs, \
         and a second opinion about it here would be a second place to change"
    );
}

#[test]
fn another_component_linking_a_crate_the_record_does_not_name_is_not_refused() {
    let scratch = Scratch::new(
        "other-component",
        &["model", "render"],
        &record_admitting("Own the pipeline.", "rustybuzz"),
    );
    scratch
        .member("model", &["some-zip-reader"])
        .member("render", &[]);
    assert_eq!(
        problems(&scratch.0),
        vec![],
        "this rule is the rendering decision's and its subject is the pipeline; \
         what a reader may link against is crates/cli/tests/boundary.rs"
    );
}

#[test]
fn a_record_without_the_admitting_section_is_refused() {
    let scratch = Scratch::new(
        "no-section",
        &["model", "render"],
        "# 0006 Rendering\n\n## The decision, in full\n\nOwn the pipeline.\n",
    );
    scratch
        .member("model", &[])
        .member("render", &["some-layout-engine"]);
    assert_eq!(
        problems(&scratch.0),
        vec![Problem::TheRecordHasNoSection(SECTION.into())],
        "a heading this check cannot find is one that admits nothing and explains \
         nothing, so it says the record moved rather than blaming the dependency"
    );
}

#[test]
fn the_same_record_with_that_section_is_not_refused() {
    let scratch = Scratch::new(
        "section-back",
        &["model", "render"],
        &record_admitting("Own the pipeline.", "some-layout-engine"),
    );
    scratch
        .member("model", &[])
        .member("render", &["some-layout-engine"]);
    assert_eq!(problems(&scratch.0), vec![]);
}

#[test]
fn a_workspace_with_no_pipeline_member_is_refused() {
    let scratch = Scratch::new(
        "no-pipeline",
        &["model", "drawing"],
        &record_admitting("Own the pipeline.", "rustybuzz"),
    );
    scratch
        .member("model", &[])
        .member("drawing", &["some-layout-engine"]);
    assert_eq!(
        problems(&scratch.0),
        vec![Problem::ThePipelineIsNotAMember(PIPELINE.into())],
        "a pipeline renamed out from under this check would leave every leg green \
         over a floor nothing judges"
    );
}

#[test]
fn the_same_workspace_with_the_pipeline_present_is_not_refused() {
    let scratch = Scratch::new(
        "pipeline-back",
        &["model", "render"],
        &record_admitting("Own the pipeline.", "rustybuzz"),
    );
    scratch.member("model", &[]).member("render", &[]);
    assert_eq!(problems(&scratch.0), vec![]);
}
