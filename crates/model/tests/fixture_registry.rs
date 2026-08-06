//! Every fixture document has a record beside it, and every record has a
//! fixture.
//!
//! A fixture with no record is a file nobody can decide about: not whether a
//! change may alter its expected output, and not whether this repository may
//! hold it at all. A record with no fixture is a claim about a file that is not
//! there. Both fail the suite rather than being skipped, which is the whole
//! point: an ignored fixture is worse than an absent one, because the suite goes
//! green either way.
//!
//! The rules live in `tests/fixtures/README.md` and are argued in
//! `docs/test-harness.md`.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// What a fixture directory can be wrong about, one variant per refusal.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Problem {
    /// A fixture file with no `<stem>.md` beside it.
    FixtureWithoutRecord(String),
    /// A `<stem>.md` with no fixture beside it.
    RecordWithoutFixture(String),
    /// A record that does not say what the fixture is for, or where it came from.
    RecordMissingProvenance(String, &'static str),
    /// A subdirectory, which would hold fixtures this walk never sees.
    NotFlat(String),
}

impl Problem {
    /// The line a failing run prints. It names the path and what is wrong with
    /// it, so the cause is locatable from the output alone.
    fn describe(&self) -> String {
        match self {
            Problem::FixtureWithoutRecord(name) => format!(
                "{name} has no record beside it. Add {stem}.md saying what it is \
                 for and where it came from.",
                stem = Path::new(name)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or(name)
            ),
            Problem::RecordWithoutFixture(name) => format!(
                "{name} is a record for a fixture that is not here. Add the \
                 fixture or remove the record."
            ),
            Problem::RecordMissingProvenance(name, missing) => format!(
                "{name} does not say `{missing}`. Both that and the other \
                 required line are what make a fixture decidable."
            ),
            Problem::NotFlat(name) => format!(
                "{name} is a directory. The fixture directory is flat, because a \
                 fixture the walk does not see is one nobody maintains."
            ),
        }
    }
}

/// The one file in the directory that is neither a fixture nor a record.
const EXCLUDED: &str = "README.md";

const REQUIRED_LINES: [&str; 2] = ["What it is for:", "Where it came from:"];

/// Reads a fixture directory and returns everything wrong with it, sorted.
fn problems(dir: &Path) -> Vec<Problem> {
    let mut found = Vec::new();
    let mut records: BTreeSet<String> = BTreeSet::new();
    let mut fixtures: BTreeSet<String> = BTreeSet::new();

    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        // A missing directory is not a pass. The caller decides what to do with
        // that, and the test below asserts it exists.
        Err(error) => panic!("cannot read {}: {error}", dir.display()),
    };

    for entry in entries {
        let entry = entry.expect("cannot read a directory entry");
        let name = entry.file_name().to_string_lossy().into_owned();
        let kind = entry.file_type().expect("cannot read a file type");

        if kind.is_dir() {
            found.push(Problem::NotFlat(name));
            continue;
        }
        if name == EXCLUDED {
            continue;
        }

        if name.ends_with(".md") {
            let stem = name.trim_end_matches(".md").to_owned();
            let text = fs::read_to_string(entry.path())
                .unwrap_or_else(|error| panic!("cannot read {name}: {error}"));
            for required in REQUIRED_LINES {
                if !text.contains(required) {
                    found.push(Problem::RecordMissingProvenance(name.clone(), required));
                }
            }
            records.insert(stem);
        } else {
            let stem = match name.rsplit_once('.') {
                Some((stem, _)) => stem.to_owned(),
                None => name.clone(),
            };
            fixtures.insert(stem);
        }
    }

    for stem in fixtures.difference(&records) {
        found.push(Problem::FixtureWithoutRecord(format!("{stem}.*")));
    }
    for stem in records.difference(&fixtures) {
        found.push(Problem::RecordWithoutFixture(format!("{stem}.md")));
    }

    found.sort();
    found
}

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("fixtures")
}

#[test]
fn the_fixture_directory_is_where_it_is_declared_to_be() {
    let dir = fixture_dir();
    assert!(
        dir.is_dir(),
        "{} is the one declared fixture directory and it is not there",
        dir.display()
    );
    assert!(
        dir.join(EXCLUDED).is_file(),
        "{} has no {EXCLUDED}, which is where the fixture rules are written",
        dir.display()
    );
}

#[test]
fn every_fixture_has_a_record_and_every_record_has_a_fixture() {
    let dir = fixture_dir();
    let found = problems(&dir);
    assert!(
        found.is_empty(),
        "{} does not hold up:\n{}",
        dir.display(),
        found
            .iter()
            .map(|problem| format!("  {}", problem.describe()))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// A scratch directory that removes itself, so the legs below leave nothing
/// behind and need no dependency to build one.
struct Scratch(PathBuf);

impl Scratch {
    fn new(label: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "rechenblatt-fixture-registry-{}-{label}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("cannot create a scratch directory");
        fs::write(dir.join(EXCLUDED), "# fixtures\n").expect("cannot write the readme");
        Scratch(dir)
    }

    fn write(&self, name: &str, contents: &str) -> &Self {
        fs::write(self.0.join(name), contents).expect("cannot write a scratch file");
        self
    }

    fn record(&self, name: &str) -> &Self {
        self.write(
            name,
            "What it is for: a leg of this test.\nWhere it came from: written here.\n",
        )
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

// The legs. Each one builds a directory holding exactly the thing its property is
// about and requires that property and no other, and the neighbouring leg changes
// the one thing back and requires nothing at all. A check that refuses everything
// fails the second kind; one that refuses nothing fails the first.

#[test]
fn a_fixture_without_a_record_is_refused() {
    let scratch = Scratch::new("fixture-without-record");
    scratch.write("sheet.xlsx", "bytes");
    assert_eq!(
        problems(&scratch.0),
        vec![Problem::FixtureWithoutRecord("sheet.*".into())]
    );
}

#[test]
fn the_same_fixture_with_a_record_is_not_refused() {
    let scratch = Scratch::new("fixture-with-record");
    scratch.write("sheet.xlsx", "bytes").record("sheet.md");
    assert_eq!(problems(&scratch.0), vec![]);
}

#[test]
fn a_record_without_a_fixture_is_refused() {
    let scratch = Scratch::new("record-without-fixture");
    scratch.record("sheet.md");
    assert_eq!(
        problems(&scratch.0),
        vec![Problem::RecordWithoutFixture("sheet.md".into())]
    );
}

#[test]
fn a_record_that_does_not_say_where_it_came_from_is_refused() {
    let scratch = Scratch::new("record-without-provenance");
    scratch
        .write("sheet.xlsx", "bytes")
        .write("sheet.md", "What it is for: a leg of this test.\n");
    assert_eq!(
        problems(&scratch.0),
        vec![Problem::RecordMissingProvenance(
            "sheet.md".into(),
            "Where it came from:"
        )]
    );
}

#[test]
fn a_record_that_does_not_say_what_it_is_for_is_refused() {
    let scratch = Scratch::new("record-without-purpose");
    scratch
        .write("sheet.xlsx", "bytes")
        .write("sheet.md", "Where it came from: written here.\n");
    assert_eq!(
        problems(&scratch.0),
        vec![Problem::RecordMissingProvenance(
            "sheet.md".into(),
            "What it is for:"
        )]
    );
}

#[test]
fn a_subdirectory_is_refused() {
    let scratch = Scratch::new("not-flat");
    fs::create_dir(scratch.0.join("nested")).expect("cannot create a subdirectory");
    assert_eq!(
        problems(&scratch.0),
        vec![Problem::NotFlat("nested".into())]
    );
}

#[test]
fn a_directory_holding_only_its_readme_is_not_refused() {
    let scratch = Scratch::new("empty");
    assert_eq!(problems(&scratch.0), vec![]);
}

#[test]
fn the_readme_is_excluded_by_name_and_cannot_hide_a_record() {
    let scratch = Scratch::new("readme-not-a-record");
    scratch.record("notes.md");
    assert_eq!(
        problems(&scratch.0),
        vec![Problem::RecordWithoutFixture("notes.md".into())],
        "a file that is not the readme is judged as a record even when it is named like prose"
    );
}
