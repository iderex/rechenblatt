//! A path named in the documentation is a path that is there.
//!
//! A document that points at a moved file reads exactly like one that points at
//! the right file. There is no symptom: the prose is still correct English, the
//! reference still looks authoritative, and the reader who follows it is the one
//! who finds out. That is the failure this refuses, and the near-miss it is
//! aimed at is a component renamed while the note describing it is not.
//!
//! What counts as a path here, and why the rule is drawn where it is. A
//! backticked token with a directory separator in it - `crates/model`,
//! `docs/checks.md` - is a path this repository can resolve, so it is resolved.
//! A bare file name with no separator is not judged, because a document
//! sometimes has to name a file precisely in order to say that it is absent, and
//! a check that refused that would refuse the sentence doing the disclosing.
//! Widening this, and covering the rest of the tree's prose, is issue #100.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// What the documentation can be wrong about.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct DanglingPath {
    /// The document naming it, relative to the directory that was walked.
    document: String,
    /// The path it named.
    named: String,
}

impl DanglingPath {
    /// The line a failing run prints, naming both ends so the cause is
    /// locatable from the output alone.
    fn describe(&self) -> String {
        format!(
            "{} names `{}`, which is not in the tree. Point it at what the thing \
             is called now, or stop naming it.",
            self.document, self.named
        )
    }
}

/// Directories at the root that are not this repository's. `.git` is git's own
/// and `target` is cargo's, and neither is tracked, so a token reaching into one
/// resolves or not depending on how the checkout was made rather than on
/// anything a commit changed.
const NOT_OURS: [&str; 2] = [".git", "target"];

/// Whether a backticked token is a path this repository can be asked to resolve.
///
/// Two things have to hold, and each of them keeps out a shape that is not a
/// path but reads like one.
///
/// It has a directory separator. That tells `docs/checks.md`, which is a path,
/// from `clippy.toml`, which in the one document naming it is a file being
/// described as absent - and a check that refused that sentence would be
/// refusing the disclosure.
///
/// Its first segment is a directory at the root of this repository. That tells
/// `crates/model` from `iderex/jellyfin-plugin-sso`, which is the name of a
/// repository somewhere else and was never going to be in this tree.
///
/// A placeholder is kept out by the character set: `<` and `>` are not in it, so
/// `docs/decisions/<number>-<slug>.md` is prose about a naming convention rather
/// than a claim that a file is there.
fn judged_as_a_path(root: &Path, token: &str) -> bool {
    if !token.contains('/') || token.starts_with('/') || token.contains("://") {
        return false;
    }
    let shaped = token.chars().all(|c| {
        c.is_ascii_alphanumeric() || c == '/' || c == '.' || c == '-' || c == '_' || c == '*'
    });
    if !shaped {
        return false;
    }
    let first = token.split('/').next().unwrap_or("");
    if first.is_empty() || NOT_OURS.contains(&first) {
        return false;
    }
    root.join(first).is_dir()
}

/// Every backticked token in a document. Fenced blocks are skipped: a block is
/// commands and output, where a path is often one that the command is about to
/// create or has just removed.
fn backticked(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut fenced = false;
    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            fenced = !fenced;
            continue;
        }
        if fenced {
            continue;
        }
        let mut rest = line;
        while let Some(open) = rest.find('`') {
            let after = &rest[open + 1..];
            match after.find('`') {
                Some(close) => {
                    found.push(after[..close].to_owned());
                    rest = &after[close + 1..];
                }
                None => break,
            }
        }
    }
    found
}

/// Reads a documentation directory and returns every path it names that is not
/// in `root`, sorted.
fn dangling(root: &Path, docs: &Path) -> Vec<DanglingPath> {
    let mut found = BTreeSet::new();
    let mut documents = 0;

    let entries = fs::read_dir(docs)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", docs.display()));
    let mut stack: Vec<PathBuf> = entries
        .map(|entry| entry.expect("cannot read a directory entry").path())
        .collect();

    while let Some(path) = stack.pop() {
        if path.is_dir() {
            let entries = fs::read_dir(&path)
                .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
            for entry in entries {
                stack.push(entry.expect("cannot read a directory entry").path());
            }
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        documents += 1;
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
        let document = path
            .strip_prefix(docs)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        for token in backticked(&text) {
            if judged_as_a_path(root, &token) && !root.join(&token).exists() {
                found.insert(DanglingPath {
                    document: document.clone(),
                    named: token,
                });
            }
        }
    }

    assert!(
        documents > 0,
        "{} holds no documents; refusing to report a clean tree",
        docs.display()
    );

    found.into_iter().collect()
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .to_path_buf()
}

#[test]
fn every_path_the_documentation_names_is_in_the_tree() {
    let root = workspace_root();
    let docs = root.join("docs");
    let found = dangling(&root, &docs);
    assert!(
        found.is_empty(),
        "the documentation points at things that are not there:\n{}",
        found
            .iter()
            .map(|problem| format!("  {}", problem.describe()))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn the_architecture_note_is_where_it_is_declared_to_be() {
    let note = workspace_root().join("docs").join("architecture.md");
    assert!(
        note.is_file(),
        "{} is the note describing the workspace and it is not there",
        note.display()
    );
}

/// A scratch documentation directory that removes itself.
struct Scratch(PathBuf);

impl Scratch {
    fn new(label: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "rechenblatt-documentation-{}-{label}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("docs")).expect("cannot create a scratch directory");
        fs::create_dir_all(dir.join("crates").join("model"))
            .expect("cannot create a scratch component");
        fs::write(
            dir.join("crates").join("model").join("lib.rs"),
            "//! here\n",
        )
        .expect("cannot write a scratch file");
        Scratch(dir)
    }

    fn document(&self, name: &str, text: &str) -> &Self {
        fs::write(self.0.join("docs").join(name), text).expect("cannot write a scratch document");
        self
    }

    fn found(&self) -> Vec<DanglingPath> {
        dangling(&self.0, &self.0.join("docs"))
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

// The legs. Each one writes a document holding exactly the thing its property is
// about and requires that refusal and no other, and the neighbouring leg changes
// the one thing back and requires nothing at all.

#[test]
fn a_named_path_that_is_not_there_is_refused() {
    let scratch = Scratch::new("dangling");
    scratch.document("note.md", "The model lives in `crates/modle/lib.rs`.\n");
    assert_eq!(
        scratch.found(),
        vec![DanglingPath {
            document: "note.md".into(),
            named: "crates/modle/lib.rs".into()
        }],
        "a component renamed without its note going with it is the near-miss this \
         is aimed at"
    );
}

#[test]
fn the_same_path_spelled_right_is_not_refused() {
    let scratch = Scratch::new("resolves");
    scratch.document("note.md", "The model lives in `crates/model/lib.rs`.\n");
    assert_eq!(scratch.found(), vec![]);
}

#[test]
fn a_named_directory_that_is_there_is_not_refused() {
    let scratch = Scratch::new("directory");
    scratch.document("note.md", "Components live under `crates/model`.\n");
    assert_eq!(scratch.found(), vec![]);
}

#[test]
fn a_bare_file_name_is_not_judged() {
    let scratch = Scratch::new("bare-name");
    scratch.document("note.md", "There is no `clippy.toml` in this tree.\n");
    assert_eq!(
        scratch.found(),
        vec![],
        "a document has to be able to name a file in order to say it is absent"
    );
}

#[test]
fn a_path_inside_a_fenced_block_is_not_judged() {
    let scratch = Scratch::new("fenced");
    scratch.document(
        "note.md",
        "Run it:\n\n```\nmv crates/modle crates/model\n```\n\nThat is all.\n",
    );
    assert_eq!(
        scratch.found(),
        vec![],
        "a block is commands and output, where a path is often one the command \
         is about to create or has just removed"
    );
}

#[test]
fn a_url_is_not_judged_as_a_path() {
    let scratch = Scratch::new("url");
    scratch.document("note.md", "See `https://example.invalid/a/b` for more.\n");
    assert_eq!(scratch.found(), vec![]);
}

#[test]
fn a_repository_somewhere_else_is_not_judged_as_a_path() {
    let scratch = Scratch::new("foreign-slug");
    scratch.document("note.md", "The sibling is `iderex/jellyfin-plugin-sso`.\n");
    assert_eq!(
        scratch.found(),
        vec![],
        "`owner/name` is the shape of a repository elsewhere and was never going \
         to be in this tree"
    );
}

#[test]
fn something_reaching_into_a_directory_that_is_not_ours_is_not_judged() {
    let scratch = Scratch::new("not-ours");
    fs::create_dir_all(scratch.0.join(".git")).expect("cannot create a scratch .git");
    scratch.document("note.md", "The token would land in `.git/config`.\n");
    assert_eq!(
        scratch.found(),
        vec![],
        "whether that resolves depends on how the checkout was made rather than \
         on anything a commit changed"
    );
}

#[test]
fn a_placeholder_is_not_judged_as_a_path() {
    let scratch = Scratch::new("placeholder");
    scratch.document(
        "note.md",
        "A record is `docs/decisions/<number>-<slug>.md`.\n",
    );
    assert_eq!(scratch.found(), vec![]);
}

#[test]
fn a_second_document_is_walked_too() {
    let scratch = Scratch::new("two-documents");
    scratch
        .document("one.md", "The model lives in `crates/model`.\n")
        .document("two.md", "And also in `crates/modle`.\n");
    assert_eq!(
        scratch.found(),
        vec![DanglingPath {
            document: "two.md".into(),
            named: "crates/modle".into()
        }]
    );
}
