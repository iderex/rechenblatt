//! What a cell shows for what it contains, one row per format kind.
//!
//! The table below is the documentation issue #19 asks for: each row names
//! the document feature it stands for, by the identifier
//! `docs/fidelity-features.md` declares, the format code, the value, and the
//! text the incumbent shows for it. A failing row prints all four, so the
//! reader sees which behaviour moved and does not have to decode a diff.
//!
//! The expectations are what the incumbent shows for these codes and values,
//! written from the specification and from its documented behaviour rather
//! than read out of a document, which is a claim this file makes about
//! another program and says so. The corpus issue #26 builds is where each of
//! them meets a document with the same code in it, and a row that disagrees
//! with a document loses to the document.
//!
//! Every feature identifier a row names has to be one the feature list
//! declares, and the last test here reads that list to say so, so a row
//! cannot claim a feature the corpus does not measure.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use rechenblatt_model::number_format::{
    Colour, Epoch, Format, Locale, Unreadable, builtin, builtin_code, parse, serial_to_civil,
};

/// One row of the table.
struct Row {
    feature: &'static str,
    code: &'static str,
    value: f64,
    shows: &'static str,
}

const fn row(feature: &'static str, code: &'static str, value: f64, shows: &'static str) -> Row {
    Row {
        feature,
        code,
        value,
        shows,
    }
}

/// The table, under the 1900 epoch and the fallback locale.
fn table() -> Vec<Row> {
    vec![
        // Sections: positive, negative, zero and the sign each one carries.
        row("format.sections", "0", 5.0, "5"),
        row("format.sections", "0", -5.0, "-5"),
        row("format.sections", "0;(0)", -5.0, "(5)"),
        row("format.sections", "0;(0)", 5.0, "5"),
        row("format.sections", "0;(0);\"zero\"", 0.0, "zero"),
        row("format.sections", "0;-0;0", 0.0, "0"),
        row("format.sections", "#,##0 ;(#,##0)", -1234.0, "(1,234)"),
        row("format.sections", "#,##0 ;(#,##0)", 1234.0, "1,234 "),
        // Decimal places round half away from zero on the decimal digits.
        row("format.sections", "0.00", 1234.567, "1234.57"),
        row("format.sections", "0.00", 1.005, "1.01"),
        row("format.sections", "0", 2.5, "3"),
        row("format.sections", "0", -2.5, "-3"),
        row("format.sections", "0.0", 0.05, "0.1"),
        row("format.sections", "#.##", 0.5, ".5"),
        row("format.sections", "#.##", 3.0, "3."),
        row("format.sections", "0.##", 3.0, "3."),
        row("format.sections", "0.00", 0.0, "0.00"),
        row("format.sections", "#", 0.0, ""),
        row("format.sections", "000", 7.0, "007"),
        row("format.sections", "???", 7.0, "  7"),
        row("format.sections", "0.???", 1.5, "1.5  "),
        // Grouping and scaling by the comma.
        row("format.locale-separators", "#,##0", 1234567.0, "1,234,567"),
        row("format.locale-separators", "#,##0.00", 1234.5, "1,234.50"),
        row("format.locale-separators", "#,##0", 123.0, "123"),
        row("format.locale-separators", "#,##0", -1234.5, "-1,235"),
        row("format.locale-separators", "0,", 12345.0, "12"),
        row("format.locale-separators", "0.0,", 12345.0, "12.3"),
        row(
            "format.locale-separators",
            "#,##0,,\"M\"",
            12345678.0,
            "12M",
        ),
        row("format.locale-separators", "#,##0.0,,", 1500000.0, "1.5"),
        // Percent.
        row("format.sections", "0%", 0.256, "26%"),
        row("format.sections", "0.00%", 0.256, "25.60%"),
        row("format.sections", "0.0%", 1.0, "100.0%"),
        // Literal text, quoted, escaped and passed through.
        row("format.literal-text", "\"EUR \"0.00", 12.5, "EUR 12.50"),
        row("format.literal-text", "0.00\" kg\"", 12.5, "12.50 kg"),
        row("format.literal-text", "\\€0", 5.0, "€5"),
        row("format.literal-text", "0 \"items\"", 3.0, "3 items"),
        row("format.literal-text", "$#,##0.00", 1234.5, "$1,234.50"),
        row("format.literal-text", "+0", 5.0, "+5"),
        row(
            "format.literal-text",
            "[$€-407]#,##0.00",
            1234.5,
            "€1,234.50",
        ),
        // Fill and skip.
        row("format.fill-and-repeat", "_(#,##0_)", 1234.0, " 1,234 "),
        row(
            "format.fill-and-repeat",
            "_(#,##0_);(#,##0)",
            -1234.0,
            "(1,234)",
        ),
        row("format.fill-and-repeat", "0*-", 5.0, "5"),
        row("format.fill-and-repeat", "*-0", 5.0, "5"),
        // Conditions on the value.
        row(
            "format.condition",
            "[<100]\"small\";[>=100]\"large\"",
            5.0,
            "small",
        ),
        row(
            "format.condition",
            "[<100]\"small\";[>=100]\"large\"",
            500.0,
            "large",
        ),
        row(
            "format.condition",
            "[<1000]0;[>=1000]0.0,\"K\"",
            999.0,
            "999",
        ),
        row(
            "format.condition",
            "[<1000]0;[>=1000]0.0,\"K\"",
            1500.0,
            "1.5K",
        ),
        row("format.condition", "[=0]\"none\";0", 0.0, "none"),
        row("format.condition", "[=0]\"none\";0", 7.0, "7"),
        row("format.condition", "[<>0]0.0;\"-\"", 0.0, "-"),
        // Scientific.
        row("format.scientific", "0.00E+00", 1234.567, "1.23E+03"),
        row("format.scientific", "0.00E+00", 0.000123, "1.23E-04"),
        row("format.scientific", "0.00E-00", 1234.567, "1.23E03"),
        row("format.scientific", "0.00E-00", 0.000123, "1.23E-04"),
        row("format.scientific", "0.00E+00", 0.0, "0.00E+00"),
        row("format.scientific", "##0.0E+0", 1234.567, "1.2E+3"),
        row("format.scientific", "##0.0E+0", 12345.67, "12.3E+3"),
        row("format.scientific", "##0.0E+0", 123456.7, "123.5E+3"),
        row("format.scientific", "0.00e+00", 1234.567, "1.23e+03"),
        row("format.scientific", "0.00E+00", -1234.567, "-1.23E+03"),
        // Fractions.
        row("format.fraction", "# ?/?", 0.5, " 1/2"),
        row("format.fraction", "# ?/?", 2.5, "2 1/2"),
        row("format.fraction", "# ?/?", 0.333, " 1/3"),
        // The numerator is padded on the left to its placeholders and the
        // denominator on the right, which is the rule the `?` states; these
        // two rows are derived from it rather than observed.
        row("format.fraction", "# ??/??", 0.333, "  1/3 "),
        row("format.fraction", "# ???/???", 355.0 / 113.0, "3  16/113"),
        row("format.fraction", "?/?", 2.5, "5/2"),
        row("format.fraction", "# ?/8", 0.5, " 4/8"),
        row("format.fraction", "# ?/?", 2.0, "2    "),
        row("format.fraction", "# ?/?", -2.5, "-2 1/2"),
        // Dates under the 1900 epoch.
        row("format.date-epoch", "yyyy-mm-dd", 45000.0, "2023-03-15"),
        row("format.date-epoch", "mm-dd-yy", 45000.0, "03-15-23"),
        row("format.date-epoch", "d-mmm-yy", 45000.0, "15-Mar-23"),
        row("format.date-epoch", "d-mmm", 45000.0, "15-Mar"),
        row("format.date-epoch", "mmm-yy", 45000.0, "Mar-23"),
        row(
            "format.date-epoch",
            "dddd, mmmm d, yyyy",
            45000.0,
            "Wednesday, March 15, 2023",
        ),
        row("format.date-epoch", "ddd", 45000.0, "Wed"),
        row("format.date-epoch", "mmmmm", 45000.0, "M"),
        row("format.date-epoch", "m/d/yyyy", 1.0, "1/1/1900"),
        row("format.date-epoch", "m/d/yyyy", 0.0, "1/0/1900"),
        row("format.date-epoch", "yyyy-mm-dd", 59.0, "1900-02-28"),
        row("format.date-epoch", "yyyy-mm-dd", 60.0, "1900-02-29"),
        row("format.date-epoch", "yyyy-mm-dd", 61.0, "1900-03-01"),
        row("format.date-epoch", "dddd", 1.0, "Sunday"),
        row("format.date-epoch", "dddd", 61.0, "Thursday"),
        row("format.date-epoch", "yyyy-mm-dd", 2958465.0, "9999-12-31"),
        // Times.
        row("format.date-epoch", "h:mm", 0.75, "18:00"),
        row("format.date-epoch", "h:mm AM/PM", 0.75, "6:00 PM"),
        row("format.date-epoch", "h:mm am/pm", 0.25, "6:00 am"),
        row("format.date-epoch", "h:mm A/P", 0.75, "6:00 P"),
        row("format.date-epoch", "hh:mm:ss", 0.5, "12:00:00"),
        row("format.date-epoch", "h:mm:ss AM/PM", 0.5, "12:00:00 PM"),
        row("format.date-epoch", "h:mm:ss AM/PM", 0.0, "12:00:00 AM"),
        row("format.date-epoch", "m/d/yy h:mm", 45000.5, "3/15/23 12:00"),
        row("format.date-epoch", "mm:ss", 0.000694444444444444, "01:00"),
        row("format.date-epoch", "mmss.0", 0.0006944444444, "0100.0"),
        row(
            "format.date-epoch",
            "hh:mm:ss.000",
            0.000011574074074,
            "00:00:01.000",
        ),
        row("format.date-epoch", "h:mm:ss", 0.9999999, "0:00:00"),
        row(
            "format.date-epoch",
            "yyyy-mm-dd h:mm:ss",
            45000.9999999,
            "2023-03-16 0:00:00",
        ),
        // Elapsed time runs past its unit.
        row("format.elapsed-time", "[h]:mm:ss", 1.5, "36:00:00"),
        row("format.elapsed-time", "[h]:mm", 0.75, "18:00"),
        row("format.elapsed-time", "[mm]:ss", 1.0, "1440:00"),
        row("format.elapsed-time", "[s]", 1.0, "86400"),
        row("format.elapsed-time", "[h]", 2.0, "48"),
        // General.
        row("format.builtin-by-number", "General", 1234.5, "1234.5"),
        row("format.builtin-by-number", "General", 0.1 + 0.2, "0.3"),
        row(
            "format.builtin-by-number",
            "General",
            1.0 / 3.0,
            "0.333333333333333",
        ),
        row("format.builtin-by-number", "General", 0.0, "0"),
        row("format.builtin-by-number", "General", -1.5, "-1.5"),
        row(
            "format.builtin-by-number",
            "General",
            100000000000.0,
            "100000000000",
        ),
        row(
            "format.builtin-by-number",
            "General",
            1234567890123456.0,
            "1.23456789012346E+15",
        ),
        row("format.builtin-by-number", "General", 0.00001, "1E-05"),
        row("format.builtin-by-number", "General", 0.0001, "0.0001"),
        row(
            "format.builtin-by-number",
            "General",
            123456789012345.0,
            "123456789012345",
        ),
    ]
}

fn show(code: &str, value: f64) -> Result<Option<String>, Unreadable> {
    let format = parse(code)?;
    Ok(format
        .apply_number(value, Epoch::Windows1900, &Locale::fallback())
        .map(|f| f.text))
}

#[test]
fn every_row_of_the_table_shows_what_the_incumbent_shows() {
    let mut failures = Vec::new();
    for row in table() {
        match show(row.code, row.value) {
            Ok(Some(text)) if text == row.shows => {}
            Ok(Some(text)) => failures.push(format!(
                "{feature}: `{code}` with {value:?} shows {text:?}, and the incumbent shows {shows:?}",
                feature = row.feature,
                code = row.code,
                value = row.value,
                shows = row.shows
            )),
            Ok(None) => failures.push(format!(
                "{feature}: `{code}` with {value:?} shows nothing, and the incumbent shows {shows:?}",
                feature = row.feature,
                code = row.code,
                value = row.value,
                shows = row.shows
            )),
            Err(error) => failures.push(format!(
                "{feature}: `{code}` could not be read: {error}",
                feature = row.feature,
                code = row.code
            )),
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} rows disagree:\n{}",
        failures.len(),
        table().len(),
        failures
            .iter()
            .map(|line| format!("  {line}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

// The features named above are features the corpus measures.

fn feature_list() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("docs")
        .join("fidelity-features.md")
}

#[test]
fn every_feature_a_row_names_is_one_the_feature_list_declares() {
    let text = fs::read_to_string(feature_list())
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", feature_list().display()));
    let declared: BTreeSet<String> = text
        .lines()
        .filter(|line| line.starts_with("| `"))
        .filter_map(|line| line.split('`').nth(1))
        .map(str::to_owned)
        .collect();
    let named: BTreeSet<&str> = table().iter().map(|row| row.feature).collect();
    let undeclared: Vec<&str> = named
        .iter()
        .copied()
        .filter(|feature| !declared.contains(*feature))
        .collect();
    assert!(
        undeclared.is_empty(),
        "rows name features the list does not declare: {undeclared:?}"
    );
}

#[test]
fn every_feature_issue_19_delivers_has_a_row() {
    let text = fs::read_to_string(feature_list())
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", feature_list().display()));
    let delivered_here: Vec<String> = text
        .lines()
        .filter(|line| line.starts_with("| `format.") && line.trim_end().ends_with("| #19 |"))
        .filter_map(|line| line.split('`').nth(1))
        .map(str::to_owned)
        .collect();
    let named: BTreeSet<&str> = table().iter().map(|row| row.feature).collect();
    let missing: Vec<&String> = delivered_here
        .iter()
        .filter(|feature| !named.contains(feature.as_str()))
        .filter(|feature| feature.as_str() != "format.colour")
        .collect();
    // `format.colour` has no text to show and is covered by the colour test
    // below rather than by a row.
    assert!(
        missing.is_empty(),
        "the feature list delivers these through #19 and no row here covers them: {missing:?}"
    );
    assert!(
        !delivered_here.is_empty(),
        "the feature list names nothing for #19"
    );
}

// Colour, the locale tag, the built-in numbers, text, and both epochs.

#[test]
fn a_colour_named_in_a_section_is_reported_beside_the_text() {
    let format = parse("[Red]0;[Blue]-0;[Color12]0").expect("a readable code");
    let locale = Locale::fallback();
    let shown = format
        .apply_number(5.0, Epoch::Windows1900, &locale)
        .expect("shown");
    assert_eq!(
        (shown.text.as_str(), shown.colour),
        ("5", Some(Colour::Red))
    );
    let shown = format
        .apply_number(-5.0, Epoch::Windows1900, &locale)
        .expect("shown");
    assert_eq!(
        (shown.text.as_str(), shown.colour),
        ("-5", Some(Colour::Blue))
    );
    let shown = format
        .apply_number(0.0, Epoch::Windows1900, &locale)
        .expect("shown");
    assert_eq!(
        (shown.text.as_str(), shown.colour),
        ("0", Some(Colour::Indexed(12)))
    );
}

#[test]
fn a_colour_number_outside_the_palette_is_unreadable() {
    assert_eq!(
        parse("[Color57]0"),
        Err(Unreadable::ColourOutOfRange { at: 0, number: 57 })
    );
}

#[test]
fn a_fill_directive_is_reported_at_its_position_rather_than_expanded() {
    let format = parse("0*-\"x\"").expect("a readable code");
    let shown = format
        .apply_number(12.0, Epoch::Windows1900, &Locale::fallback())
        .expect("shown");
    assert_eq!(shown.text, "12x");
    let fill = shown.fill.expect("a fill");
    assert_eq!((fill.at, fill.with), (2, '-'));
}

#[test]
fn a_locale_tag_in_the_code_is_reported_for_the_caller_to_resolve() {
    let format = parse("[$-407]#,##0.00").expect("a readable code");
    assert_eq!(format.locale_tag, Some(0x407));
    let format = parse("[$€-2]#,##0.00").expect("a readable code");
    assert_eq!(format.locale_tag, Some(2));
    let format = parse("0.00").expect("a readable code");
    assert_eq!(format.locale_tag, None);
}

#[test]
fn the_separators_come_from_the_locale_handed_in_and_not_from_the_host() {
    let german = Locale {
        decimal: ',',
        group: '.',
        ..Locale::fallback()
    };
    let format = parse("#,##0.00").expect("a readable code");
    let shown = format
        .apply_number(1234.5, Epoch::Windows1900, &german)
        .expect("shown");
    assert_eq!(shown.text, "1.234,50");
    let shown = format
        .apply_number(1234.5, Epoch::Windows1900, &Locale::fallback())
        .expect("shown");
    assert_eq!(shown.text, "1,234.50");
}

#[test]
fn the_month_names_come_from_the_locale_handed_in() {
    let mut names = Locale::fallback();
    names.months_short = [
        "Jan", "Feb", "Mär", "Apr", "Mai", "Jun", "Jul", "Aug", "Sep", "Okt", "Nov", "Dez",
    ];
    let format = parse("d. mmm yyyy").expect("a readable code");
    let shown = format
        .apply_number(45000.0, Epoch::Windows1900, &names)
        .expect("shown");
    assert_eq!(shown.text, "15. Mär 2023");
}

#[test]
fn every_built_in_number_the_specification_fixes_is_readable() {
    let fixed: Vec<u16> = (0..=49).filter(|id| builtin_code(*id).is_some()).collect();
    assert_eq!(
        fixed,
        vec![
            0, 1, 2, 3, 4, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 37, 38, 39, 40,
            45, 46, 47, 48, 49
        ]
    );
    for id in fixed {
        assert!(builtin(id).is_ok(), "built-in {id} did not parse");
    }
}

#[test]
fn a_built_in_number_the_specification_leaves_to_the_locale_is_unreadable() {
    for id in [5u16, 8, 23, 36, 41, 44, 50, 163] {
        assert_eq!(builtin(id), Err(Unreadable::BuiltInNotFixed { id }));
    }
}

#[test]
fn built_in_formats_show_what_their_codes_show() {
    let locale = Locale::fallback();
    let cases: [(u16, f64, &str); 8] = [
        (1, 3.7, "4"),
        (2, 3.7, "3.70"),
        (4, 1234.5, "1,234.50"),
        (9, 0.5, "50%"),
        (14, 45000.0, "03-15-23"),
        (22, 45000.5, "3/15/23 12:00"),
        (38, -1234.0, "(1,234)"),
        (46, 1.5, "36:00:00"),
    ];
    for (id, value, shows) in cases {
        let shown = builtin(id)
            .expect("fixed")
            .apply_number(value, Epoch::Windows1900, &locale)
            .expect("shown");
        assert_eq!(shown.text, shows, "built-in {id} with {value}");
    }
    let shown = builtin(38)
        .expect("fixed")
        .apply_number(-1234.0, Epoch::Windows1900, &locale)
        .expect("shown");
    assert_eq!(shown.colour, Some(Colour::Red));
}

#[test]
fn text_goes_through_the_text_section_or_through_the_placeholder() {
    let format = parse("0.00;-0.00;0.00;\"Name: \"@").expect("a readable code");
    assert_eq!(format.apply_text("abc").text, "Name: abc");
    let format = parse("@\" units\"").expect("a readable code");
    assert_eq!(format.apply_text("kg").text, "kg units");
    let format = parse("0.00").expect("a readable code");
    assert_eq!(format.apply_text("abc").text, "abc");
    let format = parse("@").expect("a readable code");
    assert_eq!(
        format
            .apply_number(1.5, Epoch::Windows1900, &Locale::fallback())
            .expect("shown")
            .text,
        "1.5"
    );
}

// The two epoch bases and the day that does not exist.

#[test]
fn the_1904_epoch_starts_on_the_first_of_january_1904() {
    let format = parse("yyyy-mm-dd").expect("a readable code");
    let locale = Locale::fallback();
    let shown = format
        .apply_number(0.0, Epoch::Mac1904, &locale)
        .expect("shown");
    assert_eq!(shown.text, "1904-01-01");
    let shown = format
        .apply_number(45000.0 - 1462.0, Epoch::Mac1904, &locale)
        .expect("shown");
    assert_eq!(shown.text, "2023-03-15");
    let format = parse("dddd").expect("a readable code");
    assert_eq!(
        format
            .apply_number(0.0, Epoch::Mac1904, &locale)
            .expect("shown")
            .text,
        "Friday"
    );
}

#[test]
fn the_1904_epoch_has_no_day_that_does_not_exist() {
    let format = parse("yyyy-mm-dd").expect("a readable code");
    let locale = Locale::fallback();
    let shown = format
        .apply_number(59.0, Epoch::Mac1904, &locale)
        .expect("shown");
    assert_eq!(shown.text, "1904-02-29");
    let shown = format
        .apply_number(60.0, Epoch::Mac1904, &locale)
        .expect("shown");
    assert_eq!(shown.text, "1904-03-01");
}

#[test]
fn the_1900_epoch_counts_the_twenty_ninth_of_february_1900_which_did_not_happen() {
    // The leap-year bug of the older base, reproduced deliberately. Serial 60
    // is a day the calendar never had, and every later serial is one day
    // ahead of the calendar because of it. Compatibility means matching it.
    let civil = serial_to_civil(60.0, Epoch::Windows1900, false).expect("in range");
    assert_eq!((civil.year, civil.month, civil.day), (1900, 2, 29));
    let before = serial_to_civil(59.0, Epoch::Windows1900, false).expect("in range");
    assert_eq!((before.year, before.month, before.day), (1900, 2, 28));
    let after = serial_to_civil(61.0, Epoch::Windows1900, false).expect("in range");
    assert_eq!((after.year, after.month, after.day), (1900, 3, 1));
    // The weekday sequence does not skip the phantom day either: the first of
    // January 1900 was a Monday, and the older base shows it as a Sunday.
    assert_eq!(
        serial_to_civil(1.0, Epoch::Windows1900, false)
            .expect("in range")
            .weekday,
        0
    );
}

#[test]
fn a_serial_outside_the_calendar_shows_nothing() {
    let format = parse("yyyy-mm-dd").expect("a readable code");
    let locale = Locale::fallback();
    assert_eq!(format.apply_number(-1.0, Epoch::Windows1900, &locale), None);
    assert_eq!(
        format.apply_number(2958466.5, Epoch::Windows1900, &locale),
        None
    );
    assert_eq!(serial_to_civil(f64::NAN, Epoch::Windows1900, false), None);
}

// What the parser refuses, each named with where it stopped.

#[test]
fn what_the_parser_refuses_is_named_with_where_it_stopped() {
    assert_eq!(parse(""), Err(Unreadable::Empty));
    assert_eq!(
        parse("0;0;0;0;0"),
        Err(Unreadable::TooManySections { found: 5 })
    );
    assert_eq!(
        parse("0.00\"abc"),
        Err(Unreadable::UnterminatedQuote { at: 4 })
    );
    assert_eq!(
        parse("[Red0.00"),
        Err(Unreadable::UnterminatedBracket { at: 0 })
    );
    assert_eq!(
        parse("[Pink]0"),
        Err(Unreadable::UnknownBracket {
            at: 0,
            text: "Pink".to_string()
        })
    );
    assert_eq!(parse("0.00\\"), Err(Unreadable::DanglingEscape { at: 4 }));
    assert_eq!(parse("0*"), Err(Unreadable::DanglingDirective { at: 1 }));
    assert_eq!(
        parse("ge"),
        Err(Unreadable::CalendarCode { at: 0, code: 'g' })
    );
    assert_eq!(
        parse("yyyy 0.00"),
        Err(Unreadable::MixedDateAndNumber { section: 0 })
    );
    assert_eq!(
        parse("0;yyyy 0"),
        Err(Unreadable::MixedDateAndNumber { section: 1 })
    );
    for error in [
        Unreadable::Empty,
        Unreadable::BuiltInNotFixed { id: 5 },
        Unreadable::MixedDateAndNumber { section: 1 },
    ] {
        assert!(!error.to_string().is_empty());
    }
}

#[test]
fn a_trailing_separator_writes_an_empty_section_that_shows_nothing() {
    let format = parse("0;").expect("a readable code");
    assert_eq!(format.sections.len(), 2);
    let shown = format
        .apply_number(-5.0, Epoch::Windows1900, &Locale::fallback())
        .expect("shown");
    assert_eq!(shown.text, "");
}

#[test]
fn the_minute_is_told_from_the_month_by_its_neighbours() {
    let locale = Locale::fallback();
    let shown = parse("m")
        .expect("readable")
        .apply_number(45000.5, Epoch::Windows1900, &locale)
        .expect("shown");
    assert_eq!(shown.text, "3");
    let shown = parse("h:m")
        .expect("readable")
        .apply_number(45000.5, Epoch::Windows1900, &locale)
        .expect("shown");
    assert_eq!(shown.text, "12:0");
    let shown = parse("m:ss")
        .expect("readable")
        .apply_number(45000.5, Epoch::Windows1900, &locale)
        .expect("shown");
    assert_eq!(shown.text, "0:00");
}

#[test]
fn a_parsed_format_can_be_read_back_as_sections() {
    let format: Format = parse("[>100][Green]#,##0.00;[Red]-0").expect("readable");
    assert_eq!(format.sections.len(), 2);
    let first = format.sections.first().expect("two sections");
    assert!(first.condition.is_some());
    assert_eq!(first.colour, Some(Colour::Green));
    let second = format.sections.get(1).expect("two sections");
    assert_eq!(second.colour, Some(Colour::Red));
}
