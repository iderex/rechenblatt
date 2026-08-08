//! Which component may depend on which, refused by the suite rather than
//! noticed by a reviewer.
//!
//! `docs/architecture.md` argues the direction: calc reads the model, render
//! reads the model and calc, the macro track reads the model and calc and not
//! render, host holds the capabilities and depends on nothing here, and the
//! command depends on all five. Two of those are worth naming on their own
//! because each is a thing somebody will want to do. A macro that renders is a
//! second renderer. A model that computes is a model with an opinion about
//! values, which is the one claim this project makes about itself.
//!
//! Until this file existed the note said the workspace manifest was the
//! enforcement for both, and it is not. A crate cannot USE what it does not
//! depend on, which is a different sentence: adding the dependency line is one
//! line in one manifest, after which the compiler is happy and every route here
//! stays green. What stood behind the rule was a reviewer noticing, and a
//! reviewer noticing is what this repository calls prose.
//!
//! The near-miss this is aimed at is that line and not a rewrite. Somebody in
//! the macro track wants a rendering, writes `rechenblatt-render.workspace =
//! true` into `crates/macro/Cargo.toml`, and gets it. Nothing else about the
//! change looks wrong.
//!
//! What this reads. Each member manifest's `[dependencies]` and
//! `[build-dependencies]`, which are what a compiled component links against,
//! and only the entries naming another member of this workspace. A dependency
//! from outside the workspace is not this check's subject at all:
//! `crates/cli/tests/boundary.rs` judges those against the list the workspace
//! manifest holds, and two checks refusing one thing would be two places to
//! repair it and two places to disagree.
//!
//! Where it stops. It does not read `[dev-dependencies]`, on the same line
//! `crates/cli/tests/boundary.rs` draws and for the same reason: a test is not
//! the component. It reads declared dependencies rather than source, so it says
//! nothing about what a component does with what it links. And it judges the
//! graph as declared in this file, so a component nobody wrote an entry for is
//! refused rather than allowed everything - which is the direction that keeps a
//! new crate from arriving outside the rule.
//!
//! Why the graph is written here rather than in a manifest. A member declaring
//! the edges it may have, beside the edges it has, makes adding a forbidden one
//! two lines in one file instead of one, and both lines land in the same diff
//! for the same reason. Written here, an edge costs an edit to the file that
//! argues why it is refused, next to the argument.
//!
//! This test lives in the command's crate rather than in a parsing one because
//! it walks the tree, and the command is on the side of the input boundary that
//! is allowed to.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// The package names, so a graph entry and a refusal cannot disagree about how
/// a component is spelled. The command's package is not `rechenblatt-cli`, and
/// that is the one somebody writing this list from memory gets wrong.
const MODEL: &str = "rechenblatt-model";
const CALC: &str = "rechenblatt-calc";
const RENDER: &str = "rechenblatt-render";
const MACROS: &str = "rechenblatt-macro";
const HOST: &str = "rechenblatt-host";
const COMMAND: &str = "rechenblatt";

/// The name no member answers to, used by the legs below for a component that
/// is not there and by nothing else.
const ABSENT: &str = "rechenblatt-drawing";

/// One component and everything it may depend on inside this workspace.
type Edges = (&'static str, &'static [&'static str]);

/// The graph `docs/architecture.md` argues, as data a run can read.
///
/// An empty list is a component that may depend on nothing here, which is a
/// declaration rather than an omission: the model is what a document says and
/// host is where a capability is allowed to be, and neither reads anything else
/// in this tree.
const DECLARED: &[Edges] = &[
    (MODEL, &[]),
    (CALC, &[MODEL]),
    (RENDER, &[MODEL, CALC]),
    (MACROS, &[MODEL, CALC]),
    (HOST, &[]),
    (COMMAND, &[MODEL, CALC, RENDER, MACROS, HOST]),
];

/// What a workspace can be wrong about, one variant per refusal.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Problem {
    /// A component depending on one the graph does not let it depend on.
    EdgeNotDeclared {
        from: String,
        to: String,
        allowed: String,
    },
    /// A member of the workspace that the graph holds no entry for.
    ComponentNotInTheGraph(String),
    /// A name the graph uses that no member of the workspace answers to.
    GraphNamesSomethingAbsent(String),
}

impl Problem {
    /// The line a failing run prints. It names the edge, so the cause is
    /// locatable from the output alone, and it ends in the repair.
    fn describe(&self) -> String {
        match self {
            Problem::EdgeNotDeclared { from, to, allowed } => format!(
                "{from} -> {to} is not an edge this workspace has. {from} may depend on \
                 {allowed} and on nothing else here. docs/architecture.md is where the \
                 direction is argued, so change the argument before the edge."
            ),
            Problem::ComponentNotInTheGraph(name) => format!(
                "{name} is a member of this workspace and the graph in \
                 crates/cli/tests/component_edges.rs holds no entry for it. A component \
                 whose edges nobody wrote down is one this check cannot judge, so it is \
                 refused rather than allowed everything."
            ),
            Problem::GraphNamesSomethingAbsent(name) => format!(
                "the graph in crates/cli/tests/component_edges.rs names {name}, which no \
                 member of this workspace is called. An entry left behind by a rename \
                 permits edges onto a name nothing has, while reading as though it were \
                 still holding something."
            ),
        }
    }
}

/// One member's manifest, as much of it as this check reads.
struct Member {
    /// The package name, which is what a dependency line names.
    name: String,
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

/// The quoted strings in the array opened by `key = [`, which may be on one
/// line or spread over several.
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
/// their key. A dependency entry spread over several lines would be read as
/// more entries than it has, which the guard on the key shape turns into
/// nothing rather than into a wrong name.
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

/// Reads a workspace against a graph and returns everything wrong with its
/// edges, sorted.
///
/// A root that cannot be read, that declares no members, or whose members do
/// not name themselves, panics rather than returning an empty list. A silent
/// pass over a workspace this could not see is the one outcome worse than a
/// refusal.
fn problems(root: &Path, graph: &[Edges]) -> Vec<Problem> {
    let manifest = root.join("Cargo.toml");
    let text = fs::read_to_string(&manifest)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", manifest.display()));

    let members = string_array(&text, "members");
    assert!(
        !members.is_empty(),
        "{} declares no workspace members; refusing to report a clean tree",
        manifest.display()
    );

    let mut read: Vec<Member> = Vec::new();
    for member in &members {
        let path = root.join(member).join("Cargo.toml");
        let body = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
        let parsed = read_member(&body);
        assert!(
            !parsed.name.is_empty(),
            "{} declares no package name; refusing to report a clean tree",
            path.display()
        );
        read.push(parsed);
    }

    let present: BTreeSet<&str> = read.iter().map(|member| member.name.as_str()).collect();
    let mut found = BTreeSet::new();

    for (component, allowed) in graph {
        if !present.contains(component) {
            found.insert(Problem::GraphNamesSomethingAbsent((*component).to_owned()));
        }
        for target in *allowed {
            if !present.contains(target) {
                found.insert(Problem::GraphNamesSomethingAbsent((*target).to_owned()));
            }
        }
    }

    for member in &read {
        let entry = graph.iter().find(|(name, _)| *name == member.name);
        let Some((_, allowed)) = entry else {
            found.insert(Problem::ComponentNotInTheGraph(member.name.clone()));
            continue;
        };
        for dep in &member.deps {
            if !present.contains(dep.as_str()) || allowed.contains(&dep.as_str()) {
                continue;
            }
            found.insert(Problem::EdgeNotDeclared {
                from: member.name.clone(),
                to: dep.clone(),
                allowed: describe_allowed(allowed),
            });
        }
    }

    found.into_iter().collect()
}

/// What a component may depend on, as a refusal reads it out.
fn describe_allowed(allowed: &[&str]) -> String {
    if allowed.is_empty() {
        return "nothing in this workspace".to_owned();
    }
    allowed.join(", ")
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .to_path_buf()
}

#[test]
fn the_workspace_holds_the_declared_graph() {
    let found = problems(&workspace_root(), DECLARED);
    assert!(
        found.is_empty(),
        "the component edges are not the ones the architecture note argues:\n{}",
        found
            .iter()
            .map(|problem| format!("  {}", problem.describe()))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// One member of a fixture workspace: the directory it sits in, the package it
/// declares, and what it links.
type Fixture = (&'static str, &'static str, Vec<&'static str>);

/// The workspace this repository declares, as a fixture the legs start from.
///
/// Every leg below changes exactly one thing about this and requires exactly
/// one refusal, so what a leg proves is the rule this repository ships rather
/// than a rule invented for the fixture. A fixture holding fewer components
/// than the graph names would redden for the graph's own entries and prove
/// nothing about the edge it was written for.
fn correct() -> Vec<Fixture> {
    vec![
        ("model", MODEL, vec![]),
        ("calc", CALC, vec![MODEL]),
        ("render", RENDER, vec![MODEL, CALC]),
        ("macro", MACROS, vec![MODEL, CALC]),
        ("host", HOST, vec![]),
        ("cli", COMMAND, vec![MODEL, CALC, RENDER, MACROS, HOST]),
    ]
}

/// The same fixture with one member's dependency list added to.
fn also_depending_on(component: &str, extra: &'static str) -> Vec<Fixture> {
    let mut members = correct();
    let entry = members
        .iter_mut()
        .find(|(_, package, _)| *package == component)
        .expect("the fixture holds no such component");
    entry.2.push(extra);
    members
}

/// The declared graph with one more entry in it.
fn declared_plus(extra: Edges) -> Vec<Edges> {
    let mut graph = DECLARED.to_vec();
    graph.push(extra);
    graph
}

/// The declared graph with one component's permissions replaced.
fn declared_letting(component: &str, allowed: &'static [&'static str]) -> Vec<Edges> {
    DECLARED
        .iter()
        .map(|(name, declared)| {
            if *name == component {
                (*name, allowed)
            } else {
                (*name, *declared)
            }
        })
        .collect()
}

/// A scratch workspace that removes itself, so the legs below leave nothing
/// behind and need no dependency to build one.
struct Scratch(PathBuf);

impl Scratch {
    fn new(label: &str, members: &[Fixture]) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "rechenblatt-component-edges-{}-{label}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("cannot create a scratch directory");
        let listed = members
            .iter()
            .map(|(directory, _, _)| format!("  \"crates/{directory}\",\n"))
            .collect::<String>();
        fs::write(
            dir.join("Cargo.toml"),
            format!("[workspace]\nmembers = [\n{listed}]\n"),
        )
        .expect("cannot write the scratch workspace");

        for (directory, package, deps) in members {
            let member = dir.join("crates").join(directory);
            fs::create_dir_all(&member).expect("cannot create a scratch member");
            let lines = deps
                .iter()
                .map(|dep| format!("{dep}.workspace = true\n"))
                .collect::<String>();
            fs::write(
                member.join("Cargo.toml"),
                format!("[package]\nname = \"{package}\"\n\n[dependencies]\n{lines}"),
            )
            .expect("cannot write a scratch member");
        }

        Scratch(dir)
    }

    fn found(&self, graph: &[Edges]) -> Vec<Problem> {
        problems(&self.0, graph)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

// The legs. Each one builds a workspace holding exactly the thing its property
// is about and requires that refusal and no other, and the leg below is the
// neighbour every one of them changes back to. A check that refuses everything
// fails that leg; one that refuses nothing fails all the others.

#[test]
fn the_workspace_the_fixture_describes_is_not_refused() {
    let scratch = Scratch::new("correct", &correct());
    assert_eq!(
        scratch.found(DECLARED),
        vec![],
        "this is the shape every leg below departs from by one thing, so a \
         refusal here would make each of them prove something else"
    );
}

#[test]
fn the_macro_track_depending_on_the_renderer_is_refused() {
    let scratch = Scratch::new("macro-reaches-render", &also_depending_on(MACROS, RENDER));
    assert_eq!(
        scratch.found(DECLARED),
        vec![Problem::EdgeNotDeclared {
            from: MACROS.into(),
            to: RENDER.into(),
            allowed: format!("{MODEL}, {CALC}"),
        }],
        "a macro runtime that can reach the drawing code is a second renderer, \
         and the one line that makes it one has to redden a run"
    );
}

#[test]
fn the_model_depending_on_calculation_is_refused() {
    let scratch = Scratch::new("model-reaches-calc", &also_depending_on(MODEL, CALC));
    assert_eq!(
        scratch.found(DECLARED),
        vec![Problem::EdgeNotDeclared {
            from: MODEL.into(),
            to: CALC.into(),
            allowed: "nothing in this workspace".into(),
        }],
        "a model that computes has an opinion about values, and the claim this \
         project makes is that the model says what the document says. The edge \
         the other way is in the fixture above and is not refused, which is the \
         direction this graph runs in"
    );
}

#[test]
fn the_renderer_depending_on_the_macro_track_is_refused() {
    let scratch = Scratch::new("render-reaches-macro", &also_depending_on(RENDER, MACROS));
    assert_eq!(
        scratch.found(DECLARED),
        vec![Problem::EdgeNotDeclared {
            from: RENDER.into(),
            to: MACROS.into(),
            allowed: format!("{MODEL}, {CALC}"),
        }],
        "the two tracks read one model and never each other, so the edge is \
         refused in both directions rather than only in the one somebody \
         thought of"
    );
}

#[test]
fn a_dependency_from_outside_the_workspace_is_not_judged_here() {
    let scratch = Scratch::new(
        "outside-crate",
        &also_depending_on(MODEL, "some-zip-reader"),
    );
    assert_eq!(
        scratch.found(DECLARED),
        vec![],
        "what a parsing component may link from outside this workspace is the \
         subject of crates/cli/tests/boundary.rs, and one rule with two places \
         to repair it is two places to disagree"
    );
}

#[test]
fn a_member_the_graph_holds_no_entry_for_is_refused() {
    let mut members = correct();
    members.push(("drawing", ABSENT, vec![MODEL]));
    let scratch = Scratch::new("member-not-in-graph", &members);
    assert_eq!(
        scratch.found(DECLARED),
        vec![Problem::ComponentNotInTheGraph(ABSENT.into())],
        "a component added without an entry would otherwise arrive outside the \
         rule and be permitted every edge it wrote for itself"
    );
}

#[test]
fn the_same_member_once_the_graph_holds_it_is_not_refused() {
    let mut members = correct();
    members.push(("drawing", ABSENT, vec![MODEL]));
    let scratch = Scratch::new("member-in-graph", &members);
    assert_eq!(scratch.found(&declared_plus((ABSENT, &[MODEL]))), vec![]);
}

#[test]
fn a_graph_entry_no_member_answers_to_is_refused() {
    let scratch = Scratch::new("graph-names-absent", &correct());
    assert_eq!(
        scratch.found(&declared_plus((ABSENT, &[MODEL]))),
        vec![Problem::GraphNamesSomethingAbsent(ABSENT.into())],
        "an entry outliving the crate it was written for permits edges onto a \
         name nothing has"
    );
}

#[test]
fn a_graph_permitting_an_edge_onto_a_name_nothing_has_is_refused() {
    let scratch = Scratch::new("graph-allows-absent", &correct());
    assert_eq!(
        scratch.found(&declared_letting(CALC, &[MODEL, ABSENT])),
        vec![Problem::GraphNamesSomethingAbsent(ABSENT.into())],
        "the permission side of an entry rots the same way its name does, and a \
         check reading only the entry names would report this one clean"
    );
}
