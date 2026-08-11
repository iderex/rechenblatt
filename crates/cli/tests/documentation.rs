//! A path named in the tracked prose is a path that is there, and a link in it
//! goes somewhere.
//!
//! A document that points at a moved file reads exactly like one that points at
//! the right file. There is no symptom: the prose is still correct English, the
//! reference still looks authoritative, and the reader who follows it is the one
//! who finds out. That is the failure this refuses, and the near-miss it is
//! aimed at is a component renamed while the note describing it is not.
//!
//! Two shapes carry a reference and each is judged by its own rule.
//!
//! A backticked token with a directory separator in it - `crates/model`,
//! `docs/checks.md` - is a path this repository can resolve, so it is resolved
//! against the root. A bare file name with no separator is not judged, because a
//! document sometimes has to name a file precisely in order to say that it is
//! absent, and a check that refused that would refuse the sentence doing the
//! disclosing.
//!
//! A markdown link is the other shape, and the exception above does not apply to
//! it: nobody links to a file in order to say it is not there. So a link target
//! is resolved whether or not it holds a separator, and it is resolved beside the
//! document rather than at the root, which is what
//! `docs/decisions/0010-macro-track.md` means when it links to
//! `docs/decisions/0002-track-order.md` by file name alone.
//!
//! What this walks is every markdown document in the tree rather than `docs/`
//! alone, so the two files a contributor opens first are judged like the rest.
//! Issue #100 is where that widening was asked for. Two of the things it asks for
//! are not here: a fenced block declaring its language, which is a convention for
//! the whole tree before it is a check, and a document enumerating something the
//! repository can print, which no reading of a document decides.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Which of the two rules found a reference, so the failure line says how the
/// thing was named and where it was looked for.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum How {
    /// A backticked token, resolved against the root of the repository.
    Backticked,
    /// A markdown link target, resolved beside the document holding it.
    Linked,
}

/// What the documentation can be wrong about.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct DanglingPath {
    /// The document naming it, relative to the root that was walked.
    document: String,
    /// The path it named, as the document spells it.
    named: String,
    /// Which rule judged it.
    how: How,
}

impl DanglingPath {
    /// The line a failing run prints, naming both ends so the cause is
    /// locatable from the output alone.
    fn describe(&self) -> String {
        match self.how {
            How::Backticked => format!(
                "{} names `{}`, which is not in the tree. Point it at what the \
                 thing is called now, or stop naming it.",
                self.document, self.named
            ),
            How::Linked => format!(
                "{} links to `{}`, which is not there. A link resolves beside the \
                 document holding it, so that is where it was looked for.",
                self.document, self.named
            ),
        }
    }
}

/// Directories that are not this repository's. `.git` is git's own and `target`
/// is cargo's, and neither is tracked, so a token reaching into one resolves or
/// not depending on how the checkout was made rather than on anything a commit
/// changed. They are skipped by name at any depth, which is wider than the two
/// places they actually occur and is the safe direction.
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

/// The part of a link target this repository can be asked to resolve, or nothing
/// where the target is somebody else's to resolve.
///
/// A fragment is dropped rather than refused: `DCO.md#3` is a claim about the
/// file and a claim about a heading inside it, and only the first is a thing this
/// tree can answer. A target that is nothing but a fragment is a move inside the
/// document and names no file at all.
///
/// A colon anywhere means a scheme - `https:`, `mailto:` - and a leading slash
/// means a target the site serving the document resolves, not this tree. Angle
/// brackets are the placeholder case the backticked rule keeps out for the same
/// reason, and they also spell markdown's bracketed target form, which is left
/// unjudged rather than half-parsed.
fn judged_as_a_link(target: &str) -> Option<&str> {
    let without_fragment = target.split('#').next().unwrap_or("");
    // `[text](target "title")` puts a title after the target, which is not part
    // of it.
    let target = without_fragment
        .split_whitespace()
        .next()
        .unwrap_or("")
        .trim();
    if target.is_empty() || target.contains(':') || target.starts_with('/') {
        return None;
    }
    if target.contains('<') || target.contains('>') {
        return None;
    }
    Some(target)
}

/// Where a link target lands, starting beside the document that holds it.
///
/// `..` is applied here rather than handed to the filesystem, so a target that
/// climbs above the root is a target that resolves to nothing instead of one
/// that resolves to whatever is above the checkout.
fn resolve_beside(root: &Path, document: &str, target: &str) -> Option<PathBuf> {
    let mut segments: Vec<&str> = match Path::new(document).parent() {
        Some(parent) => parent
            .to_str()
            .unwrap_or("")
            .split('/')
            .filter(|s| !s.is_empty())
            .collect(),
        None => Vec::new(),
    };
    for segment in target.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                segments.pop()?;
            }
            other => segments.push(other),
        }
    }
    if segments.is_empty() {
        return None;
    }
    if NOT_OURS.contains(&segments[0]) {
        return None;
    }
    let mut resolved = root.to_path_buf();
    for segment in &segments {
        resolved.push(segment);
    }
    Some(resolved)
}

/// Every backticked token in a document. Fenced blocks are skipped: a block is
/// commands and output, where a path is often one that the command is about to
/// create or has just removed.
fn backticked(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    for line in outside_fences(text) {
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

/// Every markdown link target in a document, fenced blocks skipped for the same
/// reason.
fn linked(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    for line in outside_fences(text) {
        let mut rest = line;
        while let Some(open) = rest.find("](") {
            let after = &rest[open + 2..];
            match after.find(')') {
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

/// The lines of a document that are prose rather than a fenced block.
fn outside_fences(text: &str) -> Vec<&str> {
    let mut lines = Vec::new();
    let mut fenced = false;
    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            fenced = !fenced;
            continue;
        }
        if !fenced {
            lines.push(line);
        }
    }
    lines
}

/// Every markdown document under `root`, relative to it and in forward slashes,
/// with the directories that are not this repository's left out.
fn documents(root: &Path) -> Vec<String> {
    let mut found = Vec::new();
    let mut stack = vec![root.to_path_buf()];

    while let Some(directory) = stack.pop() {
        let entries = fs::read_dir(&directory)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", directory.display()));
        for entry in entries {
            let path = entry.expect("cannot read a directory entry").path();
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            if path.is_dir() {
                if !NOT_OURS.contains(&name.as_str()) {
                    stack.push(path);
                }
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            found.push(
                path.strip_prefix(root)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .replace('\\', "/"),
            );
        }
    }

    found.sort();
    found
}

/// Reads every markdown document under `root` and returns every reference in
/// them that resolves to nothing, sorted.
fn dangling(root: &Path) -> Vec<DanglingPath> {
    let mut found = BTreeSet::new();
    let walked = documents(root);

    for document in &walked {
        let path = root.join(document);
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));

        for token in backticked(&text) {
            if judged_as_a_path(root, &token) && !root.join(&token).exists() {
                found.insert(DanglingPath {
                    document: document.clone(),
                    named: token,
                    how: How::Backticked,
                });
            }
        }

        for target in linked(&text) {
            let Some(judged) = judged_as_a_link(&target) else {
                continue;
            };
            let resolved = resolve_beside(root, document, judged);
            if !resolved.is_some_and(|path| path.exists()) {
                found.insert(DanglingPath {
                    document: document.clone(),
                    named: target,
                    how: How::Linked,
                });
            }
        }
    }

    assert!(
        !walked.is_empty(),
        "{} holds no documents; refusing to report a clean tree",
        root.display()
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
    let found = dangling(&root);
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

/// The walk reaches the documents at the root of the repository, which is the
/// half of this check that did not exist before.
///
/// A count would drift, so this names the two files a contributor opens first
/// and requires the walk to have read them. A walk that quietly stopped at
/// `docs/` again would pass every other leg here, because every other leg builds
/// its own tree.
#[test]
fn the_walk_reaches_the_documents_at_the_root() {
    let walked = documents(&workspace_root());
    for expected in ["README.md", "CONTRIBUTING.md", "tests/fixtures/README.md"] {
        assert!(
            walked.contains(&expected.to_owned()),
            "{expected} is a tracked document and the walk did not read it; it \
             read {walked:?}"
        );
    }
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

/// A scratch tree that removes itself.
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

    /// Writes a document at a path relative to the scratch root, so a leg can put
    /// one where the walk has to descend to find it.
    fn document(&self, name: &str, text: &str) -> &Self {
        let path = self.0.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("cannot create a scratch directory");
        }
        fs::write(path, text).expect("cannot write a scratch document");
        self
    }

    fn found(&self) -> Vec<DanglingPath> {
        dangling(&self.0)
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
            named: "crates/modle/lib.rs".into(),
            how: How::Backticked
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
            named: "crates/modle".into(),
            how: How::Backticked
        }]
    );
}

#[test]
fn a_document_below_the_root_is_walked_too() {
    let scratch = Scratch::new("nested");
    scratch.document("docs/decisions/note.md", "And also in `crates/modle`.\n");
    assert_eq!(
        scratch.found(),
        vec![DanglingPath {
            document: "docs/decisions/note.md".into(),
            named: "crates/modle".into(),
            how: How::Backticked
        }],
        "the walk used to start at `docs/`, so a document above it was unjudged"
    );
}

#[test]
fn a_document_inside_a_directory_that_is_not_ours_is_not_walked() {
    let scratch = Scratch::new("not-ours-document");
    scratch
        .document("note.md", "The model lives in `crates/model`.\n")
        .document("target/doc/note.md", "And also in `crates/modle`.\n");
    assert_eq!(
        scratch.found(),
        vec![],
        "what a build tool put there is not this repository's prose"
    );
}

#[test]
fn a_link_to_a_document_that_is_not_there_is_refused() {
    let scratch = Scratch::new("link-dangling");
    scratch.document("note.md", "Read [the guide](guide.md) first.\n");
    assert_eq!(
        scratch.found(),
        vec![DanglingPath {
            document: "note.md".into(),
            named: "guide.md".into(),
            how: How::Linked
        }],
        "a link is a claim that the thing is there, so the exception for a bare \
         file name does not reach it"
    );
}

#[test]
fn a_link_to_a_document_that_is_there_is_not_refused() {
    let scratch = Scratch::new("link-resolves");
    scratch
        .document("guide.md", "Here.\n")
        .document("note.md", "Read [the guide](guide.md) first.\n");
    assert_eq!(scratch.found(), vec![]);
}

#[test]
fn a_link_resolves_beside_its_document_rather_than_at_the_root() {
    let scratch = Scratch::new("link-relative");
    scratch
        .document("docs/decisions/0002-track-order.md", "The order.\n")
        .document(
            "docs/decisions/0010-macro-track.md",
            "See [the order](0002-track-order.md).\n",
        );
    assert_eq!(
        scratch.found(),
        vec![],
        "resolving this at the root would refuse a link the reader can follow"
    );
}

#[test]
fn a_link_climbing_out_of_its_directory_resolves_where_it_lands() {
    let scratch = Scratch::new("link-climbing");
    scratch
        .document("CONTRIBUTING.md", "The guide.\n")
        .document(
            "docs/note.md",
            "See [the guide](../CONTRIBUTING.md) and [the other](../CONTRIBUTNG.md).\n",
        );
    assert_eq!(
        scratch.found(),
        vec![DanglingPath {
            document: "docs/note.md".into(),
            named: "../CONTRIBUTNG.md".into(),
            how: How::Linked
        }]
    );
}

#[test]
fn a_link_that_climbs_above_the_root_resolves_to_nothing() {
    let scratch = Scratch::new("link-above-root");
    scratch.document("note.md", "See [outside](../elsewhere.md).\n");
    assert_eq!(
        scratch.found(),
        vec![DanglingPath {
            document: "note.md".into(),
            named: "../elsewhere.md".into(),
            how: How::Linked
        }],
        "what is above the checkout is not this repository's to point at"
    );
}

#[test]
fn a_link_to_somewhere_else_is_not_judged() {
    let scratch = Scratch::new("link-url");
    scratch.document(
        "note.md",
        "See [the text](https://example.invalid/a) or write to \
         [somebody](mailto:nobody@example.invalid).\n",
    );
    assert_eq!(scratch.found(), vec![]);
}

#[test]
fn a_link_that_is_only_a_fragment_is_not_judged() {
    let scratch = Scratch::new("link-fragment-only");
    scratch.document("note.md", "See [the style section](#style) below.\n");
    assert_eq!(
        scratch.found(),
        vec![],
        "a move inside the document names no file"
    );
}

#[test]
fn a_link_carrying_a_fragment_is_judged_on_the_file_before_it() {
    let scratch = Scratch::new("link-fragment");
    scratch.document("guide.md", "# Style\n").document(
        "note.md",
        "See [style](guide.md#style) and [the other](guied.md#style).\n",
    );
    assert_eq!(
        scratch.found(),
        vec![DanglingPath {
            document: "note.md".into(),
            named: "guied.md#style".into(),
            how: How::Linked
        }],
        "only the file half of the target is a thing this tree can answer"
    );
}

#[test]
fn a_link_inside_a_fenced_block_is_not_judged() {
    let scratch = Scratch::new("link-fenced");
    scratch.document(
        "note.md",
        "Like this:\n\n```\n[the guide](guide.md)\n```\n\nThat is all.\n",
    );
    assert_eq!(scratch.found(), vec![]);
}

#[test]
fn a_link_target_that_is_a_placeholder_is_not_judged() {
    let scratch = Scratch::new("link-placeholder");
    scratch.document(
        "note.md",
        "A record is [the record](docs/decisions/<number>-<slug>.md).\n",
    );
    assert_eq!(scratch.found(), vec![]);
}
