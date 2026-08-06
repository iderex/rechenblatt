//! One component takes a document apart, and everything else reads what it
//! produced.
//!
//! Two readers of one file is the failure this project exists to complain about
//! in other software. It does not arrive as a decision. It arrives as a
//! consumer that needs one part the model does not hold yet, opens the document
//! for that one part, and is by then the second authority on what the document
//! says. The two disagree later, on a document neither was written against, and
//! the disagreement is reported as a rendering bug.
//!
//! `docs/decisions/0003-workbook-model.md` is where that is argued and where the
//! eager and lazy split is written out.
//!
//! What this reads. The workspace manifest declares the text that only appears
//! in code taking a document apart, in `document-part-markers`. Each member
//! manifest declares whether it is the component that may name such text:
//!
//! ```toml
//! [package.metadata.rechenblatt]
//! reads-documents = true
//! ```
//!
//! Where it stops, stated rather than left to be discovered.
//!
//! It scans each member's `src` directory and nothing else, so a document part
//! named in a component's own tests passes. That is the same line
//! `crates/cli/tests/boundary.rs` draws at `[dev-dependencies]`, and for the
//! same reason: a test is not the component. It is also what keeps this file
//! from refusing itself, since the markers it looks for are written out in it.
//!
//! It judges text, so a marker inside a comment or a string literal is refused
//! exactly as a parser would be. That is deliberate rather than tolerated: a
//! component that has reason to write a document part name down is a component
//! that is thinking about the file, and the cheap repair is to reword the
//! sentence.
//!
//! It judges names rather than behaviour. A component that reads a document
//! part without ever naming one - handed the bytes by something else, say -
//! passes this check. Nothing in the tree refuses that today, and issue #101 is
//! where the rest of the architecture becomes tests.
//!
//! This test lives in the command's crate rather than in the model because it
//! walks the tree, and the command is on the side of the input boundary that is
//! allowed to.

use std::fs;
use std::path::{Path, PathBuf};

/// What a workspace can be wrong about, one variant per refusal.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Problem {
    /// No member declares that it reads documents.
    NoReader,
    /// More than one does.
    MoreThanOneReader(String),
    /// The marker list is empty, so the scan could refuse nothing.
    NoMarkersDeclared,
    /// A component that reads the model names a part of the document instead.
    PartNamedOutsideTheReader {
        file: String,
        line: usize,
        marker: String,
    },
}

impl Problem {
    /// The line a failing run prints. It names the file and the position where
    /// there is one, so the cause is locatable from the output alone, and it
    /// ends in the repair.
    fn describe(&self) -> String {
        match self {
            Problem::NoReader => format!(
                "no member declares {READS} in [package.metadata.rechenblatt]. One \
                 component turns a document into the model; with none declared this \
                 check has nothing to hold, so it refuses rather than passing."
            ),
            Problem::MoreThanOneReader(members) => format!(
                "{members} each declare {READS}. The document is read once, by one \
                 component, and a second reader is a second answer to every question \
                 about a file. Read the model instead, or move the part into it."
            ),
            Problem::NoMarkersDeclared => format!(
                "{MARKERS} in the workspace manifest is empty, so the scan below has \
                 nothing to look for and would pass over a parser written anywhere. \
                 Name the text that appears in code taking a document apart, or \
                 remove this check rather than leaving one that cannot fail."
            ),
            Problem::PartNamedOutsideTheReader { file, line, marker } => format!(
                "{file}:{line} names `{marker}`, which is a part of a document. This \
                 component reads the model rather than the file. Ask the model what \
                 the document says, or move the reading into the component that \
                 declares {READS}."
            ),
        }
    }
}

/// The key a member sets to say it is the one that reads documents.
const READS: &str = "reads-documents";

/// The key the workspace sets to say what such code looks like.
const MARKERS: &str = "document-part-markers";

/// The directory inside a member that holds the component itself.
const COMPONENT: &str = "src";

/// The value of `key = "..."` on a line, where the line sets that key.
fn string_value(line: &str, key: &str) -> Option<String> {
    let rest = line.strip_prefix(key)?;
    let rest = rest.trim_start();
    let rest = rest.strip_prefix('=')?.trim();
    let inner = rest.strip_prefix('"')?;
    let end = inner.find('"')?;
    Some(inner[..end].to_owned())
}

/// Whether the line sets `key = true`.
fn is_true(line: &str, key: &str) -> bool {
    match line.strip_prefix(key) {
        Some(rest) => rest.trim_start().strip_prefix('=').map(str::trim) == Some("true"),
        None => false,
    }
}

/// The quoted strings in the array opened by `key = [`.
///
/// This scans characters rather than lines, and that is the whole reason it is
/// not the shorter thing it looks like. One of the values it has to read carries
/// a closing bracket inside its own quotes, and a line-oriented reader stops at
/// the first `]` it sees and returns a shorter list than the file holds. A
/// silently shortened marker list is the worst failure available here: the check
/// goes on passing, with fewer things to look for and nothing saying so.
fn string_array(text: &str, key: &str) -> Vec<String> {
    let mut found = Vec::new();
    for (index, _) in text.match_indices(key) {
        // The key has to open its own line, so a mention of it inside a comment
        // or inside another value is not read as a declaration.
        let line_start = text[..index].rfind('\n').map(|at| at + 1).unwrap_or(0);
        if !text[line_start..index].trim().is_empty() {
            continue;
        }
        let after = text[index + key.len()..].trim_start();
        let Some(after) = after.strip_prefix('=') else {
            continue;
        };
        let Some(body) = after.trim_start().strip_prefix('[') else {
            continue;
        };

        let mut current = String::new();
        let mut quoted = false;
        let mut commented = false;
        for character in body.chars() {
            if commented {
                commented = character != '\n';
            } else if quoted {
                if character == '"' {
                    found.push(std::mem::take(&mut current));
                    quoted = false;
                } else {
                    current.push(character);
                }
            } else {
                match character {
                    '"' => quoted = true,
                    '#' => commented = true,
                    ']' => return found,
                    _ => {}
                }
            }
        }
        return found;
    }
    found
}

/// One member manifest, as much of it as this check reads.
struct Member {
    /// The package name, which is what a refusal has to name.
    name: String,
    /// Whether it declares that it reads documents. A value this cannot read is
    /// read as false, which makes the member scanned rather than exempt, so a
    /// typo tightens this check instead of opening it.
    reads_documents: bool,
}

fn read_member(text: &str) -> Member {
    let mut section = String::new();
    let mut name = String::new();
    let mut reads_documents = false;

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
            "package.metadata.rechenblatt" if is_true(line, READS) => {
                reads_documents = true;
            }
            _ => {}
        }
    }

    Member {
        name,
        reads_documents,
    }
}

/// Every file under `dir`, deepest last, as paths relative to `dir`.
///
/// A component is a directory tree rather than one file, so the walk recurses.
/// A parser one module down is the shape this is for.
fn files_under(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(path) = stack.pop() {
        let entries = match fs::read_dir(&path) {
            Ok(entries) => entries,
            // A member with no source directory at all is not a finding here.
            Err(_) => continue,
        };
        for entry in entries {
            let entry = entry.expect("cannot read a directory entry");
            if entry.file_type().expect("cannot read a file type").is_dir() {
                stack.push(entry.path());
            } else {
                found.push(entry.path());
            }
        }
    }
    found.sort();
    found
}

/// Reads a workspace and returns everything wrong with it, sorted.
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
    let markers = string_array(&text, MARKERS);

    let mut found = Vec::new();
    if markers.is_empty() {
        found.push(Problem::NoMarkersDeclared);
    }

    let mut readers = Vec::new();
    let mut others = Vec::new();
    for member in &members {
        let path = root.join(member).join("Cargo.toml");
        let body = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
        let parsed = read_member(&body);
        if parsed.reads_documents {
            readers.push(parsed.name);
        } else {
            others.push(member.clone());
        }
    }

    match readers.len() {
        0 => found.push(Problem::NoReader),
        1 => {}
        _ => {
            readers.sort();
            found.push(Problem::MoreThanOneReader(readers.join(", ")));
        }
    }

    for member in &others {
        let component = root.join(member).join(COMPONENT);
        for path in files_under(&component) {
            let Ok(source) = fs::read_to_string(&path) else {
                // A component may carry bytes that are not text. This check is
                // about what somebody wrote, so it reads what can be read.
                continue;
            };
            let named = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            for (number, line) in source.lines().enumerate() {
                for marker in &markers {
                    if line.contains(marker.as_str()) {
                        found.push(Problem::PartNamedOutsideTheReader {
                            file: named.clone(),
                            line: number + 1,
                            marker: marker.clone(),
                        });
                    }
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
fn one_component_reads_documents_and_the_rest_read_the_model() {
    let root = workspace_root();
    let found = problems(&root);
    assert!(
        found.is_empty(),
        "the model is not the only reader:\n{}",
        found
            .iter()
            .map(|problem| format!("  {}", problem.describe()))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn the_component_that_reads_documents_is_the_model() {
    let root = workspace_root();
    let text = fs::read_to_string(root.join("Cargo.toml")).expect("cannot read the workspace");
    let declared: Vec<String> = string_array(&text, "members")
        .iter()
        .filter_map(|member| {
            let body = fs::read_to_string(root.join(member).join("Cargo.toml"))
                .unwrap_or_else(|error| panic!("cannot read {member}: {error}"));
            let parsed = read_member(&body);
            parsed.reads_documents.then_some(parsed.name)
        })
        .collect();
    assert_eq!(
        declared,
        vec!["rechenblatt-model".to_owned()],
        "the record names the model as the component that owns the reading, and \
         the workspace has to say the same thing"
    );
}

/// A scratch workspace that removes itself, so the legs below leave nothing
/// behind and need no dependency to build one.
struct Scratch(PathBuf);

impl Scratch {
    fn new(label: &str, members: &[&str], markers: &[&str]) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "rechenblatt-model-ownership-{}-{label}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("cannot create a scratch directory");
        let listed = members
            .iter()
            .map(|name| format!("  \"crates/{name}\",\n"))
            .collect::<String>();
        let declared = markers
            .iter()
            .map(|marker| format!("  \"{marker}\",\n"))
            .collect::<String>();
        fs::write(
            dir.join("Cargo.toml"),
            format!(
                "[workspace]\nmembers = [\n{listed}]\n\n\
                 [workspace.metadata.rechenblatt]\n{MARKERS} = [\n{declared}]\n"
            ),
        )
        .expect("cannot write the scratch workspace");
        Scratch(dir)
    }

    /// A member, with the files it carries given as paths relative to the member
    /// directory, so a leg can put one inside `src` and another beside it.
    fn member(&self, name: &str, reads_documents: bool, files: &[(&str, &str)]) -> &Self {
        let dir = self.0.join("crates").join(name);
        fs::create_dir_all(&dir).expect("cannot create a scratch member");
        let declared = if reads_documents {
            format!("\n[package.metadata.rechenblatt]\n{READS} = true\n")
        } else {
            String::new()
        };
        fs::write(
            dir.join("Cargo.toml"),
            format!("[package]\nname = \"rechenblatt-{name}\"\n{declared}"),
        )
        .expect("cannot write a scratch member");
        for (path, contents) in files {
            let file = dir.join(path);
            if let Some(parent) = file.parent() {
                fs::create_dir_all(parent).expect("cannot create a scratch directory");
            }
            fs::write(file, contents).expect("cannot write a scratch file");
        }
        self
    }

    fn found(&self) -> Vec<Problem> {
        problems(&self.0)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// The marker the legs plant. It is one of the real ones, so a leg proves the
/// list the tree actually carries rather than a list invented for the test.
const PART: &str = "xl/";

/// A line a component that had opened the document itself would carry.
const OPENED_IT: &str = "let sheet = package.part(\"xl/worksheets/sheet1.xml\")?;\n";

/// The same component doing what the decision says it does instead.
const ASKED_THE_MODEL: &str = "let sheet = model.sheet(index)?;\n";

// The legs. Each one builds a workspace holding exactly the thing its property
// is about and requires that refusal and no other, and the neighbouring leg
// changes the one thing back and requires nothing at all. A check that refuses
// everything fails the second kind; one that refuses nothing fails the first.

#[test]
fn a_document_part_named_outside_the_reader_is_refused() {
    let scratch = Scratch::new("part-outside", &["model", "render"], &[PART]);
    scratch
        .member("model", true, &[("src/lib.rs", ASKED_THE_MODEL)])
        .member("render", false, &[("src/lib.rs", OPENED_IT)]);
    assert_eq!(
        scratch.found(),
        vec![Problem::PartNamedOutsideTheReader {
            file: "crates/render/src/lib.rs".into(),
            line: 1,
            marker: PART.into(),
        }],
        "a consumer that opens the document for the one part the model does not \
         hold yet is the way this erodes"
    );
}

#[test]
fn the_same_component_asking_the_model_is_not_refused() {
    let scratch = Scratch::new("asks-model", &["model", "render"], &[PART]);
    scratch
        .member("model", true, &[("src/lib.rs", ASKED_THE_MODEL)])
        .member("render", false, &[("src/lib.rs", ASKED_THE_MODEL)]);
    assert_eq!(scratch.found(), vec![]);
}

#[test]
fn the_same_line_inside_the_reader_is_not_refused() {
    let scratch = Scratch::new("part-inside", &["model", "render"], &[PART]);
    scratch
        .member("model", true, &[("src/lib.rs", OPENED_IT)])
        .member("render", false, &[("src/lib.rs", ASKED_THE_MODEL)]);
    assert_eq!(
        scratch.found(),
        vec![],
        "the component that declares the reading is the one place a document \
         part is supposed to be named"
    );
}

#[test]
fn a_document_part_deeper_in_the_component_is_refused() {
    let scratch = Scratch::new("part-nested", &["model", "render"], &[PART]);
    scratch
        .member("model", true, &[("src/lib.rs", ASKED_THE_MODEL)])
        .member("render", false, &[("src/package/parts.rs", OPENED_IT)]);
    assert_eq!(
        scratch.found(),
        vec![Problem::PartNamedOutsideTheReader {
            file: "crates/render/src/package/parts.rs".into(),
            line: 1,
            marker: PART.into(),
        }],
        "a component is a tree, and a second reader would arrive as a module \
         rather than as a line in the root of one"
    );
}

#[test]
fn a_document_part_named_in_a_components_own_tests_is_not_refused() {
    let scratch = Scratch::new("part-in-tests", &["model", "render"], &[PART]);
    scratch
        .member("model", true, &[("src/lib.rs", ASKED_THE_MODEL)])
        .member("render", false, &[("tests/fixtures.rs", OPENED_IT)]);
    assert_eq!(
        scratch.found(),
        vec![],
        "a test is not the component, which is the line the input boundary check \
         draws too, and a test naming a part it hands to the model is doing the \
         right thing"
    );
}

#[test]
fn a_second_component_declaring_that_it_reads_documents_is_refused() {
    let scratch = Scratch::new("two-readers", &["model", "render"], &[PART]);
    scratch
        .member("model", true, &[("src/lib.rs", ASKED_THE_MODEL)])
        .member("render", true, &[("src/lib.rs", ASKED_THE_MODEL)]);
    assert_eq!(
        scratch.found(),
        vec![Problem::MoreThanOneReader(
            "rechenblatt-model, rechenblatt-render".into()
        )],
        "two readers of one file is the failure the record is about, and a \
         second one that names no part yet is the moment before it"
    );
}

#[test]
fn the_same_workspace_with_one_declared_reader_is_not_refused() {
    let scratch = Scratch::new("one-reader", &["model", "render"], &[PART]);
    scratch
        .member("model", true, &[("src/lib.rs", ASKED_THE_MODEL)])
        .member("render", false, &[("src/lib.rs", ASKED_THE_MODEL)]);
    assert_eq!(scratch.found(), vec![]);
}

#[test]
fn a_workspace_where_nobody_declares_the_reading_is_refused() {
    let scratch = Scratch::new("no-reader", &["model", "render"], &[PART]);
    scratch
        .member("model", false, &[("src/lib.rs", ASKED_THE_MODEL)])
        .member("render", false, &[("src/lib.rs", ASKED_THE_MODEL)]);
    assert_eq!(
        scratch.found(),
        vec![Problem::NoReader],
        "a declaration removed rather than moved leaves a workspace this check \
         would otherwise pass over in silence"
    );
}

#[test]
fn an_empty_marker_list_is_refused() {
    let scratch = Scratch::new("no-markers", &["model", "render"], &[]);
    scratch
        .member("model", true, &[("src/lib.rs", ASKED_THE_MODEL)])
        .member("render", false, &[("src/lib.rs", OPENED_IT)]);
    assert_eq!(
        scratch.found(),
        vec![Problem::NoMarkersDeclared],
        "emptying the list is the one edit that turns this check off without \
         removing it, and it takes the refusal below with it"
    );
}

#[test]
fn a_marker_carrying_a_bracket_does_not_shorten_the_list() {
    let scratch = Scratch::new("bracketed-marker", &["model", "render"], &[]);
    // Written into the manifest by hand, because the shape being proved is the
    // one the scratch builder would produce anyway and the point is the reader.
    fs::write(
        scratch.0.join("Cargo.toml"),
        format!(
            "[workspace]\nmembers = [\n  \"crates/model\",\n  \"crates/render\",\n]\n\n\
             [workspace.metadata.rechenblatt]\n{MARKERS} = [\n  \
             \"[Content_Types].xml\",\n  \"{PART}\",\n]\n"
        ),
    )
    .expect("cannot write the scratch workspace");
    scratch
        .member("model", true, &[("src/lib.rs", ASKED_THE_MODEL)])
        .member("render", false, &[("src/lib.rs", OPENED_IT)]);
    assert_eq!(
        scratch.found(),
        vec![Problem::PartNamedOutsideTheReader {
            file: "crates/render/src/lib.rs".into(),
            line: 1,
            marker: PART.into(),
        }],
        "a reader that stopped at the bracket inside the first value would find \
         nothing here and report a clean workspace"
    );
}
