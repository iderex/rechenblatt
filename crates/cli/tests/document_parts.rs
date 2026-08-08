//! A part of a document is named by the component that reads documents, and by
//! nothing else.
//!
//! `docs/decisions/0003-workbook-model.md` argues the rule: one model, read
//! once, owned by `crates/model`. Everything else here reads that model rather
//! than the file, so a question about a document has one answer and not one per
//! consumer. The failure it exists against is two readers of one document that
//! disagree about it, which is the thing this project complains about in other
//! software.
//!
//! The near-miss is small and it is not a rewrite. The renderer wants a theme
//! colour, the model does not carry it yet, and a line appears in
//! `crates/render` that reaches into the package for the theme part directly.
//! Nothing else about that change looks wrong: it compiles, the edges in
//! `crates/cli/tests/component_edges.rs` are unmoved because it adds no
//! dependency between components, and the boundary in
//! `crates/cli/tests/boundary.rs` is unmoved because it adds no capability. The
//! second reader is born in one line, and the first thing that notices is a
//! document the two of them read differently.
//!
//! What this reads. Every `.rs` file under a member's `src` directory, and the
//! double-quoted spans on each line of it. A part name is a string before it is
//! anything else: code cannot open `xl/theme/theme1.xml` without writing it
//! down, so the literal is where the reach is visible.
//!
//! Where it stops, stated rather than left to be discovered.
//!
//! It judges literals, not behaviour. A part name assembled from pieces -
//! `format!("xl/worksheets/sheet{n}.xml")` with the prefix built elsewhere -
//! passes, and so does one read out of a file at runtime. That is the bound of
//! any text scan and it is why this sits beside the two dependency checks rather
//! than replacing them.
//!
//! It reads only the double-quoted spans, so a comment naming a part is not
//! refused. A component is allowed to say in prose which part the model read for
//! it; what it may not do is name that part in code. The parity of the quotes is
//! per line and an escaped quote inside a literal shifts it, which loses the
//! rest of that line rather than refusing it: this errs toward missing a
//! violation, never toward refusing a component that did nothing.
//!
//! It does not read a member's own tests, on the line
//! `crates/cli/tests/boundary.rs` draws and for the same reason: a test is not
//! the component. A fixture built in a test names parts by necessity.
//!
//! The marker list is the part names of one package shape, and it is not the
//! format decision. Which formats the first release accepts is issue #15, and a
//! format accepted there arrives with its own names for its own parts - a
//! compound file holding streams rather than a container holding paths would be
//! covered by nothing here until somebody adds them.
//!
//! This test lives in the command's crate rather than in a parsing one because
//! it walks the tree, and the command is on the side of the input boundary that
//! is allowed to.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// The package name of the component that owns the document.
///
/// It is the package name rather than the directory, so a member renamed on
/// disk without its manifest changing does not quietly become a second reader
/// or stop being the first one.
const READER: &str = "rechenblatt-model";

/// The names a document package gives its own parts.
///
/// Each entry is a name that appears in code only where that code is walking a
/// document. The list is deliberately the shape of the package rather than a
/// list of every part in it: a prefix such as the worksheets directory covers
/// every sheet in a workbook, and a check listing them one by one would be a
/// check that misses the second sheet.
const PART_MARKERS: &[&str] = &[
    "[Content_Types].xml",
    "_rels/",
    "docProps/",
    "xl/workbook.xml",
    "xl/worksheets/",
    "xl/styles.xml",
    "xl/sharedStrings.xml",
    "xl/theme/",
    "xl/drawings/",
    "xl/charts/",
    "xl/media/",
    "xl/vbaProject.bin",
];

/// What a workspace can be wrong about, one variant per refusal.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Problem {
    /// A component other than the reader naming a document part in its source.
    PartNamedOutsideTheReader {
        member: String,
        file: String,
        line: usize,
        marker: String,
    },
    /// No member of the workspace answers to the reader's name.
    TheReaderIsNotAMember(String),
}

impl Problem {
    /// The line a failing run prints. It names the file and the position, so the
    /// cause is locatable from the output alone, and it ends in the repair.
    fn describe(&self) -> String {
        match self {
            Problem::PartNamedOutsideTheReader {
                member,
                file,
                line,
                marker,
            } => format!(
                "{member} names the document part `{marker}` at {file}:{line}. A component \
                 that names a part is a component that reads the file, and this repository \
                 has one of those. Ask {READER} for what the part holds, and widen the model \
                 where it does not hold it yet. docs/decisions/0003-workbook-model.md is \
                 where that is argued, so change the argument before the line."
            ),
            Problem::TheReaderIsNotAMember(name) => format!(
                "crates/cli/tests/document_parts.rs names {name} as the component that reads \
                 documents, and no member of this workspace is called that. A reader this \
                 check cannot find is one it cannot exempt, so every component would be \
                 judged against a rule with nobody allowed to satisfy it."
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

/// The package name a member manifest declares.
fn package_name(text: &str) -> String {
    let mut section = String::new();
    let mut name = String::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(header) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            section = header.to_owned();
            continue;
        }
        if section == "package"
            && let Some(value) = string_value(line, "name")
        {
            name = value;
        }
    }
    name
}

/// The double-quoted spans on one line of source.
///
/// The split is on the quote character, so the spans are the odd pieces. An
/// escaped quote inside a literal moves the parity for the remainder of the
/// line, which drops the rest of it: a violation can hide behind one, and a
/// component that wrote no part name cannot be refused because of one.
fn literals(line: &str) -> Vec<&str> {
    line.split('"')
        .enumerate()
        .filter_map(|(index, piece)| (index % 2 == 1).then_some(piece))
        .collect()
}

/// Every `.rs` file under `dir`, deepest last, with the path each was read from.
fn sources(dir: &Path) -> Vec<(PathBuf, String)> {
    let mut found = Vec::new();
    if !dir.is_dir() {
        return found;
    }
    let mut stack = vec![dir.to_path_buf()];
    while let Some(path) = stack.pop() {
        let entries = fs::read_dir(&path)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
        for entry in entries {
            let entry = entry.expect("cannot read a directory entry").path();
            if entry.is_dir() {
                stack.push(entry);
                continue;
            }
            if entry.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let text = fs::read_to_string(&entry)
                .unwrap_or_else(|error| panic!("cannot read {}: {error}", entry.display()));
            found.push((entry, text));
        }
    }
    found.sort_by(|left, right| left.0.cmp(&right.0));
    found
}

/// Reads a workspace and returns every place a component other than the reader
/// names a document part, sorted.
///
/// A root that cannot be read, a workspace declaring no members, an empty marker
/// list and a walk that found no source all panic. A silent pass over a
/// workspace this could not see is the one outcome worse than a refusal.
fn problems(root: &Path, markers: &[&str]) -> Vec<Problem> {
    assert!(
        !markers.is_empty(),
        "no part markers; refusing to report a clean tree against a list that names nothing"
    );

    let manifest = root.join("Cargo.toml");
    let text = fs::read_to_string(&manifest)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", manifest.display()));

    let members = string_array(&text, "members");
    assert!(
        !members.is_empty(),
        "{} declares no workspace members; refusing to report a clean tree",
        manifest.display()
    );

    let mut found = Vec::new();
    let mut names = BTreeSet::new();
    let mut read = 0;

    for member in &members {
        let directory = root.join(member);
        let body = fs::read_to_string(directory.join("Cargo.toml"))
            .unwrap_or_else(|error| panic!("cannot read {member}: {error}"));
        let name = package_name(&body);
        names.insert(name.clone());

        for (path, source) in sources(&directory.join("src")) {
            read += 1;
            if name == READER {
                continue;
            }
            let shown = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            for (number, line) in source.lines().enumerate() {
                for span in literals(line) {
                    for marker in markers {
                        if span.contains(marker) {
                            found.push(Problem::PartNamedOutsideTheReader {
                                member: name.clone(),
                                file: shown.clone(),
                                line: number + 1,
                                marker: (*marker).to_owned(),
                            });
                        }
                    }
                }
            }
        }
    }

    assert!(
        read > 0,
        "{} declares members holding no source; refusing to report a clean tree",
        manifest.display()
    );

    if !names.contains(READER) {
        found.push(Problem::TheReaderIsNotAMember(READER.to_owned()));
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
fn the_document_is_named_by_one_component() {
    let root = workspace_root();
    let found = problems(&root, PART_MARKERS);
    assert!(
        found.is_empty(),
        "a component other than the model reads the document:\n{}",
        found
            .iter()
            .map(|problem| format!("  {}", problem.describe()))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// A scratch workspace that removes itself, so the legs below leave nothing
/// behind and need no dependency to build one.
struct Scratch(PathBuf);

impl Scratch {
    fn new(label: &str, members: &[&str]) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "rechenblatt-document-parts-{}-{label}",
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
        Scratch(dir)
    }

    /// A member whose `src` holds one file, and whose manifest names it.
    fn member(&self, name: &str, source: &str) -> &Self {
        self.file(name, "src/lib.rs", source)
    }

    /// A member with a file at a chosen place inside it.
    fn file(&self, name: &str, at: &str, source: &str) -> &Self {
        let dir = self.0.join("crates").join(name);
        fs::create_dir_all(&dir).expect("cannot create a scratch member");
        fs::write(
            dir.join("Cargo.toml"),
            format!("[package]\nname = \"rechenblatt-{name}\"\n"),
        )
        .expect("cannot write a scratch member");
        let path = dir.join(at);
        fs::create_dir_all(path.parent().expect("a file has a parent"))
            .expect("cannot create a scratch directory");
        fs::write(path, source).expect("cannot write a scratch source");
        self
    }

    fn found(&self) -> Vec<Problem> {
        problems(&self.0, PART_MARKERS)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

// The legs. Each one builds a workspace holding exactly the thing its property
// is about and requires that refusal and no other; each has a neighbour that
// changes the one thing back and requires nothing at all. A check that refuses
// everything fails the neighbours, and one that refuses nothing fails the first
// kind.
//
// Every leg carries a member called `model`, because a workspace without one is
// the subject of its own leg below and would otherwise be reported instead of
// the thing the leg is about.

#[test]
fn a_component_other_than_the_model_naming_a_part_is_refused() {
    let scratch = Scratch::new("render-reads-the-theme", &["model", "render"]);
    scratch
        .member("model", "pub const COMPONENT: &str = \"model\";\n")
        .member(
            "render",
            "pub const THEME: &str = \"xl/theme/theme1.xml\";\n",
        );
    assert_eq!(
        scratch.found(),
        vec![Problem::PartNamedOutsideTheReader {
            member: "rechenblatt-render".into(),
            file: "crates/render/src/lib.rs".into(),
            line: 1,
            marker: "xl/theme/".into(),
        }],
        "the renderer wanting a theme colour the model does not carry yet is how \
         a second reader of one document gets written"
    );
}

#[test]
fn the_same_line_in_the_model_is_not_refused() {
    let scratch = Scratch::new("model-reads-the-theme", &["model", "render"]);
    scratch
        .member(
            "model",
            "pub const THEME: &str = \"xl/theme/theme1.xml\";\n",
        )
        .member("render", "pub const COMPONENT: &str = \"render\";\n");
    assert_eq!(
        scratch.found(),
        vec![],
        "reading the document is what the model is for, so the identical literal \
         is the rule being kept rather than broken"
    );
}

#[test]
fn the_macro_track_reaching_for_its_own_part_is_refused() {
    let scratch = Scratch::new("macro-reads-the-project", &["model", "macro"]);
    scratch
        .member("model", "pub const COMPONENT: &str = \"model\";\n")
        .member(
            "macro",
            "fn project() -> &'static str {\n    \"xl/vbaProject.bin\"\n}\n",
        );
    assert_eq!(
        scratch.found(),
        vec![Problem::PartNamedOutsideTheReader {
            member: "rechenblatt-macro".into(),
            file: "crates/macro/src/lib.rs".into(),
            line: 2,
            marker: "xl/vbaProject.bin".into(),
        }],
        "the macro project is the part whose consumer most obviously wants it \
         directly, and the model handing over the bytes is the whole rule"
    );
}

#[test]
fn a_part_named_in_a_comment_is_not_refused() {
    let scratch = Scratch::new("comment", &["model", "render"]);
    scratch
        .member("model", "pub const COMPONENT: &str = \"model\";\n")
        .member(
            "render",
            "// The model read xl/theme/theme1.xml and resolved the slot.\n",
        );
    assert_eq!(
        scratch.found(),
        vec![],
        "a component may say in prose which part the model read for it; what it \
         may not do is name that part in code"
    );
}

#[test]
fn a_part_named_in_a_component_test_is_not_refused() {
    let scratch = Scratch::new("member-test", &["model", "render"]);
    scratch
        .member("model", "pub const COMPONENT: &str = \"model\";\n")
        .file(
            "render",
            "tests/parts.rs",
            "const FIXTURE: &str = \"xl/worksheets/sheet1.xml\";\n",
        );
    assert_eq!(
        scratch.found(),
        vec![],
        "a test is not the component, which is the line the boundary check draws \
         for dependencies and this one draws for source"
    );
}

#[test]
fn a_part_deeper_in_the_component_is_still_refused() {
    let scratch = Scratch::new("deep", &["model", "calc"]);
    scratch
        .member("model", "pub const COMPONENT: &str = \"model\";\n")
        .file(
            "calc",
            "src/read/cached.rs",
            "const AT: &str = \"xl/workbook.xml\";\n",
        );
    assert_eq!(
        scratch.found(),
        vec![Problem::PartNamedOutsideTheReader {
            member: "rechenblatt-calc".into(),
            file: "crates/calc/src/read/cached.rs".into(),
            line: 1,
            marker: "xl/workbook.xml".into(),
        }],
        "a component is every file under its src, not the one the crate root is in"
    );
}

#[test]
fn a_workspace_whose_reader_is_absent_is_refused() {
    let scratch = Scratch::new("no-reader", &["render"]);
    scratch.member("render", "pub const COMPONENT: &str = \"render\";\n");
    assert_eq!(
        scratch.found(),
        vec![Problem::TheReaderIsNotAMember(READER.into())],
        "a reader this check cannot find is one it cannot exempt, and a rule \
         nobody is allowed to satisfy is worse than no rule"
    );
}

#[test]
#[should_panic(expected = "refusing to report a clean tree")]
fn a_marker_list_that_names_nothing_stops_the_run() {
    let scratch = Scratch::new("no-markers", &["model", "render"]);
    scratch
        .member("model", "pub const COMPONENT: &str = \"model\";\n")
        .member(
            "render",
            "pub const THEME: &str = \"xl/theme/theme1.xml\";\n",
        );
    let _ = problems(&scratch.0, &[]);
}

#[test]
#[should_panic(expected = "refusing to report a clean tree")]
fn a_workspace_holding_no_source_stops_the_run() {
    let scratch = Scratch::new("no-source", &["model"]);
    let dir = scratch.0.join("crates").join("model");
    fs::create_dir_all(&dir).expect("cannot create a scratch member");
    fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"rechenblatt-model\"\n",
    )
    .expect("cannot write a scratch member");
    let _ = scratch.found();
}
