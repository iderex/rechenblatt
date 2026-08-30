//! The feature list is data, and a row that cannot be read is refused.
//!
//! `docs/fidelity-features.md` is what the corpus is built to cover, what a
//! fidelity report is broken down by, and what every rendering issue in
//! milestones 4 to 7 points at. All three of those read it by identifier, so the
//! failure this refuses is a list that has quietly stopped being readable: a row
//! with a cell missing, an identifier spelled two ways, a band nothing declares,
//! or a feature that says nothing delivers it while sitting in a table of
//! features something does.
//!
//! None of those has a symptom. The document still renders, the table still
//! looks like a table, and the reader who finds out is whoever compares this
//! month's per-feature report against last month's and gets two rows where there
//! was one.
//!
//! The bands are read out of the document's own section rather than carried
//! here, so adding a band is one edit and not two. The tracker is not read at
//! all: an issue reference is judged by its shape, because resolving one needs
//! the network and the default suite has none. A row pointing at an issue that
//! does not exist is a claim the document makes, and issue #25 says so where a
//! reader of the document will meet it.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// The heading of the one section where a feature may say nothing delivers it.
const UNDELIVERED_SECTION: &str = "Features no issue delivers";

/// What the `Delivered by` cell says when no issue delivers the feature.
const UNDELIVERED: &str = "none";

/// What a feature list can be wrong about, one variant per refusal.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Problem {
    /// A table row that is not four cells, so which cell is which is a guess.
    RowIsNotFourCells(usize, usize),
    /// An identifier that is not backticked lower-case dotted segments.
    IdentifierIsMalformed(usize, String),
    /// One identifier on two rows, which splits a feature in every report.
    IdentifierIsDeclaredTwice(String),
    /// A band no `### Band` heading declares.
    BandIsNotDeclared(String, String),
    /// A `Delivered by` cell that is neither an issue reference nor `none`.
    DelivererIsNotAnIssue(String, String),
    /// A feature nothing delivers, outside the section that exists for those.
    UndeliveredOutsideItsSection(String, String),
    /// A feature inside that section that names a deliverer after all.
    DeliveredInsideThatSection(String, String),
    /// A document declaring no bands, which would make every band acceptable.
    NoBandIsDeclared,
    /// A document declaring no features, which would make every check vacuous.
    NoFeatureIsDeclared,
}

impl Problem {
    /// The line a failing run prints. It names the row and the repair, because
    /// the reader is usually somebody who has just added a feature.
    fn describe(&self) -> String {
        match self {
            Problem::RowIsNotFourCells(line, count) => format!(
                "line {line} is a table row with {count} cell(s). A feature row \
                 is four: the identifier, what it is, the band, and what \
                 delivers it."
            ),
            Problem::IdentifierIsMalformed(line, cell) => format!(
                "line {line} names {cell} as an identifier. An identifier is \
                 backticked, lower case, and at least two dot-separated \
                 segments of letters, digits and hyphens, as in `text.runs`."
            ),
            Problem::IdentifierIsDeclaredTwice(id) => format!(
                "{id} is on two rows. One identifier is one feature, because a \
                 report compared against an earlier one is compared by \
                 identifier."
            ),
            Problem::BandIsNotDeclared(id, band) => format!(
                "{id} sits in band {band}, which no `### Band` heading declares. \
                 Declare the band where the bands are argued, or move the row."
            ),
            Problem::DelivererIsNotAnIssue(id, cell) => format!(
                "{id} says {cell} delivers it. That cell is an issue reference \
                 such as #37, or the word {UNDELIVERED} in the section for \
                 features no issue delivers."
            ),
            Problem::UndeliveredOutsideItsSection(id, section) => format!(
                "{id} says nothing delivers it and sits under {section}. Move it \
                 to {UNDELIVERED_SECTION}, which is the section that keeps such \
                 a feature visible."
            ),
            Problem::DeliveredInsideThatSection(id, cell) => format!(
                "{id} sits under {UNDELIVERED_SECTION} and names {cell} as its \
                 deliverer. A feature something delivers belongs in the table \
                 for its area."
            ),
            Problem::NoBandIsDeclared => {
                "no `### Band` heading declares a band, so every band on every \
                 row would be accepted."
                    .to_owned()
            }
            Problem::NoFeatureIsDeclared => {
                "no feature row was read, so this list measures nothing and \
                 every check over it passes."
                    .to_owned()
            }
        }
    }
}

/// One feature, as the document declares it.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Feature {
    /// The identifier, without its backticks.
    id: String,
    /// The section heading the row sits under.
    section: String,
}

/// Whether a token is an identifier this list may carry.
///
/// Two dot-separated segments at least, because a single word is an area rather
/// than a feature and areas are headings here. Each segment is lower-case
/// letters, digits and hyphens, and neither starts nor ends with a hyphen, so
/// one spelling of a feature cannot become two.
fn is_an_identifier(token: &str) -> bool {
    let segments: Vec<&str> = token.split('.').collect();
    if segments.len() < 2 {
        return false;
    }
    segments.iter().all(|segment| {
        !segment.is_empty()
            && !segment.starts_with('-')
            && !segment.ends_with('-')
            && segment
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    })
}

/// Whether a cell is a tracker reference, which is a hash and then digits.
fn is_an_issue_reference(cell: &str) -> bool {
    match cell.strip_prefix('#') {
        Some(digits) => !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()),
        None => false,
    }
}

/// The cells of a markdown table row, trimmed, without the outer empties.
///
/// A row is recognised by its leading pipe alone, so a separator row and the
/// header row come back here too and are dropped by the caller. Doing it that
/// way means a row this parser cannot read is a refusal rather than a line it
/// quietly skipped.
fn cells(line: &str) -> Vec<String> {
    let inner = line.trim().trim_start_matches('|').trim_end_matches('|');
    inner
        .split('|')
        .map(|cell| cell.trim().to_owned())
        .collect()
}

/// Reads the document and returns everything wrong with it, sorted, together
/// with the features it declared.
fn read(document: &str) -> (Vec<Feature>, Vec<Problem>) {
    let mut features: Vec<Feature> = Vec::new();
    let mut problems: Vec<Problem> = Vec::new();
    let mut bands: BTreeSet<String> = BTreeSet::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut section = String::from("no section");

    for (index, line) in document.lines().enumerate() {
        let number = index + 1;

        if let Some(heading) = line.strip_prefix("## ") {
            section = heading.trim().to_owned();
            continue;
        }
        if let Some(heading) = line.strip_prefix("### Band ") {
            if let Some((band, _)) = heading.split_once(':') {
                bands.insert(band.trim().to_owned());
            }
            continue;
        }
        if !line.starts_with('|') {
            continue;
        }

        let row = cells(line);
        if row.first().map(String::as_str) == Some("Id") || row.iter().all(|c| c.starts_with("--"))
        {
            continue;
        }
        if row.len() != 4 {
            problems.push(Problem::RowIsNotFourCells(number, row.len()));
            continue;
        }

        let Some(id) = row[0]
            .strip_prefix('`')
            .and_then(|rest| rest.strip_suffix('`'))
            .filter(|token| is_an_identifier(token))
        else {
            problems.push(Problem::IdentifierIsMalformed(number, row[0].clone()));
            continue;
        };

        if !seen.insert(id.to_owned()) {
            problems.push(Problem::IdentifierIsDeclaredTwice(id.to_owned()));
        }

        let band = row[2].clone();
        let deliverer = row[3].clone();

        if deliverer == UNDELIVERED {
            if section != UNDELIVERED_SECTION {
                problems.push(Problem::UndeliveredOutsideItsSection(
                    id.to_owned(),
                    section.clone(),
                ));
            }
        } else if !is_an_issue_reference(&deliverer) {
            problems.push(Problem::DelivererIsNotAnIssue(id.to_owned(), deliverer));
        } else if section == UNDELIVERED_SECTION {
            problems.push(Problem::DeliveredInsideThatSection(
                id.to_owned(),
                deliverer,
            ));
        }

        features.push(Feature {
            id: id.to_owned(),
            section: section.clone(),
        });

        // The band is compared after the bands are known, because a band
        // declared below a table is still a declared band.
        problems.push(Problem::BandIsNotDeclared(id.to_owned(), band));
    }

    // Every row pushed a band problem above; the ones whose band was declared
    // are taken back out here, which is what lets a band be declared anywhere in
    // the document rather than only above the first table that uses it.
    problems.retain(|problem| match problem {
        Problem::BandIsNotDeclared(_, band) => !bands.contains(band),
        _ => true,
    });

    if bands.is_empty() {
        problems.push(Problem::NoBandIsDeclared);
    }
    if features.is_empty() {
        problems.push(Problem::NoFeatureIsDeclared);
    }

    problems.sort();
    (features, problems)
}

/// The identifiers something else claims, that this list does not declare.
///
/// This is the mismatch issue #27 needs: a corpus document's manifest names the
/// features it exercises by identifier, and a name that is not on the list is a
/// document exercising something nobody can report on. It is here rather than in
/// the manifest checker because the list is here, and a second reader of this
/// document would be a second place for the reading to be wrong.
fn undeclared<'a>(features: &[Feature], claimed: &[&'a str]) -> Vec<&'a str> {
    let declared: BTreeSet<&str> = features.iter().map(|f| f.id.as_str()).collect();
    let mut missing: Vec<&str> = claimed
        .iter()
        .copied()
        .filter(|id| !declared.contains(id))
        .collect();
    missing.sort_unstable();
    missing
}

/// Where the feature list is, read from this crate rather than from a guess
/// about the working directory.
fn feature_list() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("docs")
        .join("fidelity-features.md")
}

fn tracked_document() -> String {
    let path = feature_list();
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()))
}

/// A list with nothing wrong with it, small enough that one edit below plants
/// exactly one mistake and nothing else moves.
const SOUND: &str = "# A list

### Band 1: the ones every document has

## Text

| Id | What it is | Band | Delivered by |
| --- | --- | --- | --- |
| `text.runs` | Formatting that changes inside one cell | 1 | #21 |
| `text.wrap` | Text wrapped inside its cell | 1 | #37 |

## Features no issue delivers

| Id | What it is | Band | Delivered by |
| --- | --- | --- | --- |
| `pivot.table` | A pivot table | 1 | none |
";

/// The one edit a leg makes, so the leg and its neighbour differ by that edit
/// and by nothing else.
fn with(replaced: &str, replacement: &str) -> String {
    assert!(
        SOUND.contains(replaced),
        "the fixture no longer holds {replaced}, so this leg plants nothing"
    );
    SOUND.replace(replaced, replacement)
}

fn problems_of(document: &str) -> Vec<Problem> {
    read(document).1
}

/// The lines a failing assertion prints, one problem per line.
fn described(problems: &[Problem]) -> String {
    problems
        .iter()
        .map(Problem::describe)
        .collect::<Vec<String>>()
        .join("\n")
}

#[test]
fn the_feature_list_is_where_it_is_declared_to_be() {
    let path = feature_list();
    assert!(
        path.is_file(),
        "the feature list is not at {}. Everything that reads it by identifier \
         reads it from there.",
        path.display()
    );
}

#[test]
fn the_feature_list_has_nothing_wrong_with_it() {
    let (_, problems) = read(&tracked_document());
    assert!(
        problems.is_empty(),
        "the feature list has {} problem(s):\n{}",
        problems.len(),
        described(&problems)
    );
}

#[test]
fn the_feature_list_declares_features() {
    let (features, _) = read(&tracked_document());
    assert!(
        features.len() > 50,
        "the feature list declares {} feature(s). A list this short is one that \
         has been emptied rather than one that is finished, and every check over \
         it would pass.",
        features.len()
    );
}

#[test]
fn the_three_areas_the_gap_analysis_named_are_covered() {
    let (features, _) = read(&tracked_document());
    // docs/decisions/0002-track-order.md names these three as the rendering
    // gaps this project was planned against, so a list covering none of one of
    // them is a list that has drifted away from the reason for the corpus.
    for area in ["cf.", "calc.nested-conditional", "chart."] {
        assert!(
            features.iter().any(|f| f.id.starts_with(area)),
            "no feature identifier begins with {area}, and the track order \
             record names that area as one of the three the corpus exists for."
        );
    }
}

#[test]
fn a_feature_nothing_delivers_is_in_the_section_for_those() {
    let (features, _) = read(&tracked_document());
    let listed: Vec<&Feature> = features
        .iter()
        .filter(|f| f.section == UNDELIVERED_SECTION)
        .collect();
    assert!(
        !listed.is_empty(),
        "no feature sits under {UNDELIVERED_SECTION}. An empty hole list is \
         either a plan with no holes or a section somebody deleted, and the \
         document says which one it claims to be."
    );
}

#[test]
fn a_sound_list_is_not_refused() {
    let problems = problems_of(SOUND);
    assert!(
        problems.is_empty(),
        "the fixture every leg below starts from is itself refused:\n{}",
        described(&problems)
    );
}

#[test]
fn a_row_that_is_not_four_cells_is_refused() {
    let planted = with(
        "| `text.wrap` | Text wrapped inside its cell | 1 | #37 |",
        "| `text.wrap` | Text wrapped inside its cell | 1 |",
    );
    let problems = problems_of(&planted);
    assert!(
        problems
            .iter()
            .any(|p| matches!(p, Problem::RowIsNotFourCells(_, 3))),
        "a three-cell row was accepted:\n{}",
        described(&problems)
    );
}

#[test]
fn a_row_with_its_four_cells_is_not_refused() {
    assert!(problems_of(SOUND).is_empty());
}

#[test]
fn an_identifier_that_is_not_two_dotted_segments_is_refused() {
    let planted = with("`text.wrap`", "`textwrap`");
    let problems = problems_of(&planted);
    assert!(
        problems
            .iter()
            .any(|p| matches!(p, Problem::IdentifierIsMalformed(_, _))),
        "an identifier with no dot was accepted:\n{}",
        described(&problems)
    );
}

#[test]
fn an_identifier_that_is_not_backticked_is_refused() {
    let planted = with("| `text.wrap` |", "| text.wrap |");
    let problems = problems_of(&planted);
    assert!(
        problems
            .iter()
            .any(|p| matches!(p, Problem::IdentifierIsMalformed(_, _))),
        "an identifier without its backticks was accepted:\n{}",
        described(&problems)
    );
}

#[test]
fn an_identifier_carrying_a_capital_is_refused() {
    let planted = with("`text.wrap`", "`text.Wrap`");
    let problems = problems_of(&planted);
    assert!(
        problems
            .iter()
            .any(|p| matches!(p, Problem::IdentifierIsMalformed(_, _))),
        "an identifier with a capital in it was accepted, and two spellings of \
         one feature are two rows in every report:\n{}",
        described(&problems)
    );
}

#[test]
fn an_identifier_spelled_the_declared_way_is_not_refused() {
    let planted = with("`text.wrap`", "`text.wrap-2`");
    assert!(
        problems_of(&planted).is_empty(),
        "a hyphen inside a segment is part of the declared shape and was refused"
    );
}

#[test]
fn one_identifier_on_two_rows_is_refused() {
    let planted = with("`text.wrap`", "`text.runs`");
    let problems = problems_of(&planted);
    assert!(
        problems
            .iter()
            .any(|p| matches!(p, Problem::IdentifierIsDeclaredTwice(id) if id == "text.runs")),
        "one identifier was accepted on two rows:\n{}",
        described(&problems)
    );
}

#[test]
fn two_identifiers_on_two_rows_are_not_refused() {
    let planted = with("`text.wrap`", "`text.indent`");
    assert!(problems_of(&planted).is_empty());
}

#[test]
fn a_band_no_heading_declares_is_refused() {
    let planted = with(
        "| `text.wrap` | Text wrapped inside its cell | 1 | #37 |",
        "| `text.wrap` | Text wrapped inside its cell | 4 | #37 |",
    );
    let problems = problems_of(&planted);
    assert!(
        problems
            .iter()
            .any(|p| matches!(p, Problem::BandIsNotDeclared(_, band) if band == "4")),
        "a band nothing declares was accepted:\n{}",
        described(&problems)
    );
}

#[test]
fn a_band_a_heading_declares_is_not_refused() {
    let planted = with(
        "### Band 1: the ones every document has",
        "### Band 1: the ones every document has\n\n### Band 4: the rest",
    );
    let planted = planted.replace(
        "| `text.wrap` | Text wrapped inside its cell | 1 | #37 |",
        "| `text.wrap` | Text wrapped inside its cell | 4 | #37 |",
    );
    assert!(
        problems_of(&planted).is_empty(),
        "a band declared below the table that uses it was refused:\n{}",
        described(&problems_of(&planted))
    );
}

#[test]
fn a_deliverer_that_is_not_an_issue_reference_is_refused() {
    let planted = with(
        "| `text.wrap` | Text wrapped inside its cell | 1 | #37 |",
        "| `text.wrap` | Text wrapped inside its cell | 1 | the render milestone |",
    );
    let problems = problems_of(&planted);
    assert!(
        problems
            .iter()
            .any(|p| matches!(p, Problem::DelivererIsNotAnIssue(_, _))),
        "a deliverer nobody can resolve was accepted:\n{}",
        described(&problems)
    );
}

#[test]
fn a_deliverer_that_is_an_issue_reference_is_not_refused() {
    let planted = with("| 1 | #37 |", "| 1 | #4137 |");
    assert!(problems_of(&planted).is_empty());
}

#[test]
fn a_feature_nothing_delivers_outside_its_section_is_refused() {
    let planted = with(
        "| `text.wrap` | Text wrapped inside its cell | 1 | #37 |",
        "| `text.wrap` | Text wrapped inside its cell | 1 | none |",
    );
    let problems = problems_of(&planted);
    assert!(
        problems.iter().any(
            |p| matches!(p, Problem::UndeliveredOutsideItsSection(id, _) if id == "text.wrap")
        ),
        "a feature nothing delivers was accepted inside a table of delivered \
         features, where nobody counting the holes would find it:\n{}",
        described(&problems)
    );
}

#[test]
fn a_feature_something_delivers_inside_that_section_is_refused() {
    let planted = with(
        "| `pivot.table` | A pivot table | 1 | none |",
        "| `pivot.table` | A pivot table | 1 | #59 |",
    );
    let problems = problems_of(&planted);
    assert!(
        problems.iter().any(
            |p| matches!(p, Problem::DeliveredInsideThatSection(id, _) if id == "pivot.table")
        ),
        "a delivered feature was accepted in the section that exists for the \
         undelivered ones:\n{}",
        described(&problems)
    );
}

#[test]
fn a_feature_on_the_side_it_belongs_on_is_not_refused() {
    assert!(problems_of(SOUND).is_empty());
}

#[test]
fn a_list_declaring_no_band_is_refused() {
    let planted = with("### Band 1: the ones every document has\n", "");
    let problems = problems_of(&planted);
    assert!(
        problems
            .iter()
            .any(|p| matches!(p, Problem::NoBandIsDeclared)),
        "a list declaring no band was accepted, and every band on every row \
         would then pass:\n{}",
        described(&problems)
    );
}

#[test]
fn a_list_declaring_no_feature_is_refused() {
    let problems = problems_of("# A list\n\n### Band 1: the only one\n");
    assert!(
        problems
            .iter()
            .any(|p| matches!(p, Problem::NoFeatureIsDeclared)),
        "a list with no feature row was accepted:\n{}",
        described(&problems)
    );
}

#[test]
fn an_identifier_a_document_claims_and_this_list_does_not_declare_is_caught() {
    let (features, _) = read(SOUND);
    assert_eq!(
        undeclared(&features, &["text.runs", "text.blinking"]),
        vec!["text.blinking"],
        "a manifest claiming a feature nobody can report on was not caught, \
         which is the mismatch issue #27 reads this list for"
    );
}

#[test]
fn an_identifier_a_document_claims_and_this_list_declares_is_not_caught() {
    let (features, _) = read(SOUND);
    assert!(
        undeclared(&features, &["text.runs", "text.wrap", "pivot.table"]).is_empty(),
        "a manifest claiming only declared features was refused"
    );
}

#[test]
fn the_mismatch_check_reads_the_tracked_list_too() {
    let (features, _) = read(&tracked_document());
    assert!(
        undeclared(&features, &["cf.data-bar"]).is_empty(),
        "the tracked list does not declare cf.data-bar, which the conditional \
         formatting milestone is built around"
    );
    assert_eq!(
        undeclared(&features, &["cf.data-bars"]),
        vec!["cf.data-bars"],
        "a near-miss on a real identifier was accepted against the tracked list"
    );
}

#[test]
fn every_refusal_has_a_message_that_names_its_subject() {
    // A message is only produced by a failing run, so nothing else here reads
    // one. This does, because a line nobody has read is a line that says
    // `assertion failed` in the one moment it matters.
    let messages = [
        (
            Problem::RowIsNotFourCells(12, 3).describe(),
            vec!["line 12", "3 cell"],
        ),
        (
            Problem::IdentifierIsMalformed(12, "textwrap".to_owned()).describe(),
            vec!["line 12", "textwrap"],
        ),
        (
            Problem::IdentifierIsDeclaredTwice("text.runs".to_owned()).describe(),
            vec!["text.runs", "two rows"],
        ),
        (
            Problem::BandIsNotDeclared("text.runs".to_owned(), "9".to_owned()).describe(),
            vec!["text.runs", "band 9"],
        ),
        (
            Problem::DelivererIsNotAnIssue("text.runs".to_owned(), "soon".to_owned()).describe(),
            vec!["text.runs", "soon"],
        ),
        (
            Problem::UndeliveredOutsideItsSection("text.runs".to_owned(), "Text".to_owned())
                .describe(),
            vec!["text.runs", "Text", UNDELIVERED_SECTION],
        ),
        (
            Problem::DeliveredInsideThatSection("pivot.table".to_owned(), "#59".to_owned())
                .describe(),
            vec!["pivot.table", "#59"],
        ),
        (
            Problem::NoBandIsDeclared.describe(),
            vec!["no `### Band` heading"],
        ),
        (
            Problem::NoFeatureIsDeclared.describe(),
            vec!["no feature row"],
        ),
    ];

    for (message, expected) in messages {
        for wanted in expected {
            assert!(
                message.contains(wanted),
                "a refusal message does not name {wanted}: {message}"
            );
        }
    }
}
