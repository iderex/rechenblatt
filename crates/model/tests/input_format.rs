//! The format of a document is read from its bytes, and the name it had on
//! disk plays no part.
//!
//! `docs/decisions/0004-input-formats.md` is the decision under test. The legs
//! below build every package they need in memory, with the parts a workbook
//! carries, rather than reading a document an application wrote: the corpus
//! issue #26 is where those arrive, and a detector that reads names out of a
//! central directory is proved by an archive that has one. The archives are
//! valid zip files, with a checksum per entry, so a leg that writes one to disk
//! writes something any archiver opens.
//!
//! The renamed-file legs are the ones the decision asks for. Each writes one
//! set of bytes under several names inside a directory the test made, reads
//! each file back, and requires the same answer from all of them. The detector
//! has no parameter for a name, so the legs prove the route a caller takes and
//! not only the signature of the function.

use std::fs;
use std::path::PathBuf;

use rechenblatt_model::format::{Decision, Format, detect};

// A zip writer small enough to read, producing stored entries with a checksum.
// A dependency for this would be a dependency the parsing side has not accepted,
// and `docs/decisions/0001-means.md` says one arrives with the issue that needs
// it; forty lines in a test is not that issue.

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            let low = crc & 1;
            crc >>= 1;
            if low == 1 {
                crc ^= 0xEDB8_8320;
            }
        }
    }
    !crc
}

/// A zip archive holding the given entries, stored, in the given order.
fn archive(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut directory = Vec::new();
    for (name, data) in entries {
        let name = name.as_bytes();
        let size = u32::try_from(data.len()).expect("a test entry fits");
        let crc = crc32(data);
        let local = u32::try_from(out.len()).expect("a test archive fits");

        out.extend_from_slice(&[0x50, 0x4B, 0x03, 0x04, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        out.extend_from_slice(&crc.to_le_bytes());
        out.extend_from_slice(&size.to_le_bytes());
        out.extend_from_slice(&size.to_le_bytes());
        out.extend_from_slice(&u16::try_from(name.len()).expect("short name").to_le_bytes());
        out.extend_from_slice(&[0, 0]);
        out.extend_from_slice(name);
        out.extend_from_slice(data);

        directory
            .extend_from_slice(&[0x50, 0x4B, 0x01, 0x02, 20, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        directory.extend_from_slice(&crc.to_le_bytes());
        directory.extend_from_slice(&size.to_le_bytes());
        directory.extend_from_slice(&size.to_le_bytes());
        directory.extend_from_slice(&u16::try_from(name.len()).expect("short name").to_le_bytes());
        directory.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        directory.extend_from_slice(&local.to_le_bytes());
        directory.extend_from_slice(name);
    }
    let directory_at = u32::try_from(out.len()).expect("a test archive fits");
    let directory_size = u32::try_from(directory.len()).expect("a test archive fits");
    let count = u16::try_from(entries.len()).expect("few entries");
    out.extend_from_slice(&directory);
    out.extend_from_slice(&[0x50, 0x4B, 0x05, 0x06, 0, 0, 0, 0]);
    out.extend_from_slice(&count.to_le_bytes());
    out.extend_from_slice(&count.to_le_bytes());
    out.extend_from_slice(&directory_size.to_le_bytes());
    out.extend_from_slice(&directory_at.to_le_bytes());
    out.extend_from_slice(&[0, 0]);
    out
}

/// The same archive, ending in the zip64 records a large archive carries, with
/// the ordinary end record saying its fields overflowed.
fn archive_zip64(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut out = archive(entries);
    let end = out.len() - 22;
    let directory_at = u64::from(u32::from_le_bytes([
        out[end + 16],
        out[end + 17],
        out[end + 18],
        out[end + 19],
    ]));
    let directory_size = u64::from(u32::from_le_bytes([
        out[end + 12],
        out[end + 13],
        out[end + 14],
        out[end + 15],
    ]));
    let count = u64::from(u16::from_le_bytes([out[end + 10], out[end + 11]]));
    out.truncate(end);

    let zip64_at = u64::try_from(out.len()).expect("a test archive fits");
    out.extend_from_slice(&[0x50, 0x4B, 0x06, 0x06]);
    out.extend_from_slice(&44u64.to_le_bytes());
    out.extend_from_slice(&[45, 0, 45, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    out.extend_from_slice(&count.to_le_bytes());
    out.extend_from_slice(&count.to_le_bytes());
    out.extend_from_slice(&directory_size.to_le_bytes());
    out.extend_from_slice(&directory_at.to_le_bytes());

    out.extend_from_slice(&[0x50, 0x4B, 0x06, 0x07, 0, 0, 0, 0]);
    out.extend_from_slice(&zip64_at.to_le_bytes());
    out.extend_from_slice(&[1, 0, 0, 0]);

    out.extend_from_slice(&[0x50, 0x4B, 0x05, 0x06, 0, 0, 0, 0, 0xFF, 0xFF, 0xFF, 0xFF]);
    out.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0, 0]);
    out
}

// The packages. Each holds the parts the format it stands for carries, with
// the markup reduced to what makes the part well-formed.

const CONTENT_TYPES: &[u8] = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"/>";
const PACKAGE_RELS: &[u8] = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"/>";
const WORKBOOK: &[u8] = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><workbook xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\"><sheets><sheet name=\"Sheet1\" sheetId=\"1\" r:id=\"rId1\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\"/></sheets></workbook>";
const WORKBOOK_RELS: &[u8] = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"><Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet\" Target=\"worksheets/sheet1.xml\"/></Relationships>";
const SHEET: &[u8] = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><worksheet xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\"><sheetData><row r=\"1\"><c r=\"A1\"><v>1</v></c></row></sheetData></worksheet>";
const DOCUMENT: &[u8] = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><document xmlns=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"><body/></document>";
const ODF_CONTENT: &[u8] = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><office:document-content xmlns:office=\"urn:oasis:names:tc:opendocument:xmlns:office:1.0\"/>";

fn workbook_package() -> Vec<u8> {
    archive(&[
        ("[Content_Types].xml", CONTENT_TYPES),
        ("_rels/.rels", PACKAGE_RELS),
        ("xl/workbook.xml", WORKBOOK),
        ("xl/_rels/workbook.xml.rels", WORKBOOK_RELS),
        ("xl/worksheets/sheet1.xml", SHEET),
    ])
}

fn macro_enabled_package() -> Vec<u8> {
    archive(&[
        ("[Content_Types].xml", CONTENT_TYPES),
        ("_rels/.rels", PACKAGE_RELS),
        ("xl/workbook.xml", WORKBOOK),
        ("xl/_rels/workbook.xml.rels", WORKBOOK_RELS),
        ("xl/worksheets/sheet1.xml", SHEET),
        (
            "xl/vbaProject.bin",
            &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1],
        ),
    ])
}

fn binary_workbook_package() -> Vec<u8> {
    archive(&[
        ("[Content_Types].xml", CONTENT_TYPES),
        ("_rels/.rels", PACKAGE_RELS),
        ("xl/workbook.bin", &[0x83, 0x01, 0x00]),
        ("xl/worksheets/sheet1.bin", &[0x81, 0x01, 0x00]),
    ])
}

fn word_processing_package() -> Vec<u8> {
    archive(&[
        ("[Content_Types].xml", CONTENT_TYPES),
        ("_rels/.rels", PACKAGE_RELS),
        ("word/document.xml", DOCUMENT),
    ])
}

fn open_document(mimetype: &[u8]) -> Vec<u8> {
    archive(&[("mimetype", mimetype), ("content.xml", ODF_CONTENT)])
}

/// A compound file header: the signature the older binary workbook begins
/// with, then the rest of a 512-byte header holding no streams. It is what
/// the detector reads and it is not an openable workbook, which is stated
/// here rather than left to be assumed.
fn compound_file() -> Vec<u8> {
    let mut out = vec![0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
    out.resize(512, 0);
    out
}

// What each package is detected as.

#[test]
fn a_workbook_package_is_accepted() {
    let found = detect(&workbook_package());
    assert_eq!(
        found.format,
        Format::SpreadsheetMl {
            macro_enabled: false
        }
    );
    assert_eq!(found.format.decision(), Decision::Accepted);
    assert!(found.accepted().is_ok());
}

#[test]
fn a_macro_enabled_workbook_package_is_the_same_format_with_the_project_seen() {
    let found = detect(&macro_enabled_package());
    assert_eq!(
        found.format,
        Format::SpreadsheetMl {
            macro_enabled: true
        }
    );
    assert_eq!(found.format.decision(), Decision::Accepted);
}

#[test]
fn the_evidence_for_a_workbook_package_points_at_its_workbook_part() {
    let bytes = workbook_package();
    let found = detect(&bytes);
    let entry = &bytes[found.evidence.offset..];
    assert!(
        entry.starts_with(&[0x50, 0x4B, 0x01, 0x02]),
        "{:?}",
        found.evidence
    );
    assert!(
        entry[46..].starts_with(b"xl/workbook.xml"),
        "the entry at the offset names {:?}",
        String::from_utf8_lossy(&entry[46..61])
    );
}

#[test]
fn the_older_binary_workbook_is_refused_as_deferred() {
    let found = detect(&compound_file());
    assert_eq!(found.format, Format::OlderBinaryWorkbook);
    assert_eq!(found.format.decision(), Decision::Deferred);
    let refusal = found.accepted().expect_err("not accepted");
    let message = refusal.to_string();
    assert!(
        message.contains("older binary spreadsheet format"),
        "{message}"
    );
    assert!(message.contains("does not read it yet"), "{message}");
    assert!(message.contains("at byte 0"), "{message}");
}

#[test]
fn the_binary_workbook_package_is_refused_as_undecided() {
    let found = detect(&binary_workbook_package());
    assert_eq!(found.format, Format::BinarySpreadsheetMl);
    assert_eq!(found.format.decision(), Decision::Undecided);
    let message = found.accepted().expect_err("not accepted").to_string();
    assert!(
        message.contains("binary SpreadsheetML workbook"),
        "{message}"
    );
    assert!(message.contains("no release has decided"), "{message}");
}

#[test]
fn an_open_document_spreadsheet_is_refused_as_undecided() {
    for mimetype in [
        &b"application/vnd.oasis.opendocument.spreadsheet"[..],
        &b"application/vnd.oasis.opendocument.spreadsheet-template"[..],
    ] {
        let found = detect(&open_document(mimetype));
        assert_eq!(found.format, Format::OpenDocumentSpreadsheet);
        assert_eq!(found.format.decision(), Decision::Undecided);
        assert_eq!(
            found.evidence.what,
            "the mimetype entry declaring a spreadsheet"
        );
    }
}

#[test]
fn an_open_document_that_is_not_a_spreadsheet_is_refused() {
    let found = detect(&open_document(b"application/vnd.oasis.opendocument.text"));
    assert_eq!(found.format, Format::OtherOpenDocument);
    assert_eq!(found.format.decision(), Decision::Refused);
    let message = found.accepted().expect_err("not accepted").to_string();
    assert!(
        message.contains("OpenDocument package that is not a spreadsheet"),
        "{message}"
    );
}

#[test]
fn a_compressed_mimetype_entry_is_not_read_and_the_package_is_not_a_spreadsheet() {
    // The OpenDocument format requires the entry stored. One that is compressed
    // cannot be read without inflating it, so the package is judged by what
    // else it names, which here is nothing.
    let mut bytes = open_document(b"application/vnd.oasis.opendocument.spreadsheet");
    bytes[8] = 8; // the local header's method: deflated
    let directory = bytes.len() - 22;
    let at = u32::from_le_bytes([
        bytes[directory + 16],
        bytes[directory + 17],
        bytes[directory + 18],
        bytes[directory + 19],
    ]);
    let at = usize::try_from(at).expect("fits");
    bytes[at + 10] = 8; // the central directory's method: deflated
    let found = detect(&bytes);
    assert_eq!(found.format, Format::OtherOpenDocument);
}

#[test]
fn a_word_processing_package_is_refused_as_not_a_spreadsheet() {
    let found = detect(&word_processing_package());
    assert_eq!(found.format, Format::OtherOfficePackage);
    assert_eq!(found.format.decision(), Decision::Refused);
    let message = found.accepted().expect_err("not accepted").to_string();
    assert!(message.contains("not a spreadsheet"), "{message}");
}

#[test]
fn a_zip_archive_naming_nothing_this_project_knows_is_refused() {
    let found = detect(&archive(&[("notes.txt", b"nothing to see")]));
    assert_eq!(found.format, Format::UnknownPackage);
    assert_eq!(
        found.evidence.what,
        "a central directory naming no part this project can name"
    );
}

#[test]
fn an_empty_zip_archive_is_refused_and_says_it_is_empty() {
    let mut bytes = vec![0x50, 0x4B, 0x03, 0x04];
    bytes.extend_from_slice(&archive(&[])[..]);
    // A bare end record has no local header in front of it, so one is put
    // there for the leg to reach the directory walk at all.
    let found = detect(&bytes);
    assert_eq!(found.format, Format::UnknownPackage);
    assert_eq!(
        found.evidence.what,
        "a central directory naming no part at all"
    );
}

#[test]
fn a_truncated_zip_archive_is_refused_as_damaged() {
    let bytes = workbook_package();
    let cut = &bytes[..bytes.len() - 30];
    let found = detect(cut);
    assert_eq!(found.format, Format::DamagedPackage);
    assert_eq!(found.evidence.offset, cut.len());
}

#[test]
fn a_zip_archive_too_short_to_hold_an_end_record_is_refused_as_damaged() {
    assert_eq!(
        detect(&[0x50, 0x4B, 0x03, 0x04, 0, 0]).format,
        Format::DamagedPackage
    );
}

#[test]
fn an_end_record_pointing_past_itself_is_refused_as_damaged() {
    let mut bytes = workbook_package();
    let end = bytes.len() - 22;
    bytes[end + 16..end + 20].copy_from_slice(&0xFFFF_FFF0u32.to_le_bytes());
    assert_eq!(detect(&bytes).format, Format::DamagedPackage);
}

#[test]
fn a_zip64_archive_is_walked_through_its_zip64_records() {
    let bytes = archive_zip64(&[
        ("[Content_Types].xml", CONTENT_TYPES),
        ("_rels/.rels", PACKAGE_RELS),
        ("xl/workbook.xml", WORKBOOK),
    ]);
    assert_eq!(
        detect(&bytes).format,
        Format::SpreadsheetMl {
            macro_enabled: false
        }
    );
}

#[test]
fn a_zip64_archive_with_a_broken_locator_is_refused_as_damaged() {
    let mut bytes = archive_zip64(&[("xl/workbook.xml", WORKBOOK)]);
    let locator = bytes.len() - 22 - 20;
    bytes[locator] = 0; // the locator signature
    assert_eq!(detect(&bytes).format, Format::DamagedPackage);

    let mut bytes = archive_zip64(&[("xl/workbook.xml", WORKBOOK)]);
    let locator = bytes.len() - 22 - 20;
    bytes[locator + 8..locator + 16].copy_from_slice(&u64::MAX.to_le_bytes());
    assert_eq!(detect(&bytes).format, Format::DamagedPackage);

    let mut bytes = archive_zip64(&[("xl/workbook.xml", WORKBOOK)]);
    let zip64 = bytes.len() - 22 - 20 - 56;
    bytes[zip64] = 0; // the zip64 end record signature
    assert_eq!(detect(&bytes).format, Format::DamagedPackage);
}

#[test]
fn bytes_that_are_neither_container_are_unrecognised() {
    for bytes in [
        &b""[..],
        b"hello",
        b"<?xml version=\"1.0\"?>",
        &[0xD0, 0xCF, 0x11][..],
    ] {
        let found = detect(bytes);
        assert_eq!(found.format, Format::Unrecognised, "{bytes:?}");
        assert_eq!(found.format.decision(), Decision::Refused);
        let message = found.accepted().expect_err("not accepted").to_string();
        assert!(
            message.contains("not a format this project recognises"),
            "{message}"
        );
    }
}

#[test]
fn every_refusal_names_the_format_the_byte_and_the_record() {
    let refused: Vec<Vec<u8>> = vec![
        compound_file(),
        binary_workbook_package(),
        open_document(b"application/vnd.oasis.opendocument.spreadsheet"),
        open_document(b"application/vnd.oasis.opendocument.text"),
        word_processing_package(),
        archive(&[("notes.txt", b"nothing")]),
        vec![0x50, 0x4B, 0x03, 0x04],
        b"plain text".to_vec(),
    ];
    for bytes in refused {
        let found = detect(&bytes);
        let refusal = found.accepted().expect_err("every one of these is refused");
        let message = refusal.to_string();
        assert!(message.contains(found.format.describe()), "{message}");
        assert!(
            message.contains(&format!("at byte {}", found.evidence.offset)),
            "{message}"
        );
        assert!(
            message.contains("docs/decisions/0004-input-formats.md"),
            "{message}"
        );
        assert_eq!(refusal.format, found.format);
        assert_eq!(refusal.evidence, found.evidence);
    }
}

// The renamed-file legs.

/// A scratch directory that removes itself, under the temporary directory this
/// run was given, so the legs write nowhere else.
struct Scratch(PathBuf);

impl Scratch {
    fn new(label: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "rechenblatt-input-format-{}-{label}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("cannot create a scratch directory");
        Scratch(dir)
    }

    /// Writes the bytes under the name and reads them back through the path,
    /// which is the route a caller on the host side takes.
    fn detect_as(&self, name: &str, bytes: &[u8]) -> Format {
        let path = self.0.join(name);
        fs::write(&path, bytes).expect("cannot write a scratch file");
        let read = fs::read(&path).expect("cannot read a scratch file back");
        detect(&read).format
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn a_workbook_package_is_detected_whatever_it_is_called() {
    let scratch = Scratch::new("workbook-renamed");
    let bytes = workbook_package();
    for name in ["book.xlsx", "book.xls", "book.ods", "book.txt", "book"] {
        assert_eq!(
            scratch.detect_as(name, &bytes),
            Format::SpreadsheetMl {
                macro_enabled: false
            },
            "{name} holds a workbook package and was not detected as one"
        );
    }
}

#[test]
fn an_older_binary_workbook_is_refused_whatever_it_is_called() {
    let scratch = Scratch::new("compound-renamed");
    let bytes = compound_file();
    for name in ["book.xls", "book.xlsx", "book.xlsm", "book"] {
        assert_eq!(
            scratch.detect_as(name, &bytes),
            Format::OlderBinaryWorkbook,
            "{name} holds a compound file and was not detected as one"
        );
    }
}

#[test]
fn an_open_document_spreadsheet_is_refused_whatever_it_is_called() {
    let scratch = Scratch::new("open-document-renamed");
    let bytes = open_document(b"application/vnd.oasis.opendocument.spreadsheet");
    for name in ["book.ods", "book.xlsx", "book.zip"] {
        assert_eq!(
            scratch.detect_as(name, &bytes),
            Format::OpenDocumentSpreadsheet,
            "{name} holds an OpenDocument spreadsheet and was not detected as one"
        );
    }
}
