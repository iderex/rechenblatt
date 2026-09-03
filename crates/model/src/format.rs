//! Which format a document is, read from its bytes and never from its name.
//!
//! `docs/decisions/0004-input-formats.md` is the decision this module carries
//! out: one package format is accepted by the first release, the older binary
//! workbook is refused with a message saying it is deferred rather than
//! rejected, and two formats are named as undecided. Every one of those
//! answers is reached from the content, so a file whose extension lies about it
//! is detected as what it is.
//!
//! What this reads, and what it does not. The first bytes decide between the
//! two containers the formats above live in: a compound file, whose signature
//! is fixed, and a zip archive, whose local header signature is fixed. Inside a
//! zip archive the central directory at the end of the file names every part,
//! and the names are enough to tell a workbook package from a word-processing
//! one and an OpenDocument spreadsheet from an OpenDocument text. Not one byte
//! of any part is inflated here: a detector that decompressed would carry the
//! attack surface issue #16 exists to bound, before that issue has bounded it.
//! What is read of a part's content is one stored entry, `mimetype`, which the
//! OpenDocument package format requires to be uncompressed.
//!
//! The detector allocates nothing. It walks a borrowed slice, so the memory it
//! holds is what the caller handed it, and the walk advances by at least the
//! size of one directory header per step, so its time is bounded by the length
//! of the slice.
//!
//! Where it stops. A package is recognised by the conventional names of its
//! parts and not by the relationship that formally identifies the main part,
//! because reading that relationship means inflating it. A package that names
//! its workbook part unconventionally is therefore reported as a package this
//! project cannot name, and the reader issue #16 builds, which does inflate, is
//! where the formal identification lands.

use std::fmt;

/// The formats detection tells apart.
///
/// Every variant is something a detector can say from bytes alone, and every
/// one of them carries a decision in `docs/decisions/0004-input-formats.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// A SpreadsheetML package, the current format of the incumbent suite.
    /// `macro_enabled` says whether the package carries a macro project part.
    SpreadsheetMl { macro_enabled: bool },
    /// The older binary workbook, a compound file holding streams.
    OlderBinaryWorkbook,
    /// The binary SpreadsheetML workbook: the same package with its workbook
    /// part written as binary records instead of markup.
    BinarySpreadsheetMl,
    /// An OpenDocument spreadsheet, or the template form of one.
    OpenDocumentSpreadsheet,
    /// An Office Open XML package whose main part is a word-processing or a
    /// presentation part.
    OtherOfficePackage,
    /// An OpenDocument package whose declared type is not a spreadsheet.
    OtherOpenDocument,
    /// A zip archive that names none of the parts this project can name.
    UnknownPackage,
    /// A zip archive whose central directory cannot be read: truncated, or
    /// pointing outside its own bytes.
    DamagedPackage,
    /// Bytes that begin with neither container signature.
    Unrecognised,
}

/// What the decision record says about a format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// The first release reads it.
    Accepted,
    /// The first release does not read it, and a later milestone reconsiders.
    Deferred,
    /// No release has decided whether to read it.
    Undecided,
    /// Not a spreadsheet, or not anything this project can name.
    Refused,
}

impl Format {
    /// The decision the record takes for this format.
    #[must_use]
    pub fn decision(self) -> Decision {
        match self {
            Format::SpreadsheetMl { .. } => Decision::Accepted,
            Format::OlderBinaryWorkbook => Decision::Deferred,
            Format::BinarySpreadsheetMl | Format::OpenDocumentSpreadsheet => Decision::Undecided,
            Format::OtherOfficePackage
            | Format::OtherOpenDocument
            | Format::UnknownPackage
            | Format::DamagedPackage
            | Format::Unrecognised => Decision::Refused,
        }
    }

    /// The format as a diagnostic names it.
    #[must_use]
    pub fn describe(self) -> &'static str {
        match self {
            Format::SpreadsheetMl {
                macro_enabled: false,
            } => "a SpreadsheetML workbook package",
            Format::SpreadsheetMl {
                macro_enabled: true,
            } => "a macro-enabled SpreadsheetML workbook package",
            Format::OlderBinaryWorkbook => "the older binary spreadsheet format, a compound file",
            Format::BinarySpreadsheetMl => "a binary SpreadsheetML workbook",
            Format::OpenDocumentSpreadsheet => "an OpenDocument spreadsheet",
            Format::OtherOfficePackage => "an Office Open XML package that is not a spreadsheet",
            Format::OtherOpenDocument => "an OpenDocument package that is not a spreadsheet",
            Format::UnknownPackage => "a zip archive naming no part this project can name",
            Format::DamagedPackage => "a zip archive whose central directory cannot be read",
            Format::Unrecognised => "not a format this project recognises",
        }
    }
}

/// The byte that decided the format, so a refusal points into the document
/// rather than at it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Evidence {
    /// Offset into the bytes that were handed in.
    pub offset: usize,
    /// What was found there.
    pub what: &'static str,
}

/// What detection found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Detection {
    pub format: Format,
    pub evidence: Evidence,
}

/// A document the first release does not read, with the reason and the byte.
///
/// The path is not here, on purpose. This component takes bytes and never a
/// path, which `docs/decisions/0014-input-boundary.md` decides, so the caller
/// that opened the path is the one that writes it in front of this message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Refusal {
    pub format: Format,
    pub evidence: Evidence,
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let outcome = match self.format.decision() {
            Decision::Deferred => {
                "this release does not read it yet, and a later milestone reconsiders that"
            }
            Decision::Undecided => "no release has decided whether to read it",
            Decision::Refused | Decision::Accepted => "nothing here reads it",
        };
        write!(
            f,
            "the document is {}: {}. What decided that is {} at byte {}. \
             docs/decisions/0004-input-formats.md is where the formats are decided.",
            self.format.describe(),
            outcome,
            self.evidence.what,
            self.evidence.offset
        )
    }
}

impl std::error::Error for Refusal {}

impl Detection {
    /// The detection as the reader needs it: the accepted format, or the
    /// refusal that says why not.
    ///
    /// # Errors
    ///
    /// Any format the record does not accept.
    pub fn accepted(self) -> Result<Detection, Refusal> {
        match self.format.decision() {
            Decision::Accepted => Ok(self),
            Decision::Deferred | Decision::Undecided | Decision::Refused => Err(Refusal {
                format: self.format,
                evidence: self.evidence,
            }),
        }
    }
}

/// Detect the format of a document from its bytes.
///
/// The name the document had on disk plays no part, because there is no
/// parameter for it.
#[must_use]
pub fn detect(bytes: &[u8]) -> Detection {
    const COMPOUND_FILE: [u8; 8] = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
    const LOCAL_HEADER: [u8; 4] = [0x50, 0x4B, 0x03, 0x04];

    if bytes.starts_with(&COMPOUND_FILE) {
        return Detection {
            format: Format::OlderBinaryWorkbook,
            evidence: Evidence {
                offset: 0,
                what: "the compound file signature",
            },
        };
    }
    if !bytes.starts_with(&LOCAL_HEADER) {
        return Detection {
            format: Format::Unrecognised,
            evidence: Evidence {
                offset: 0,
                what: "the first bytes, which are neither a zip local header nor a compound file signature",
            },
        };
    }
    match central_directory(bytes) {
        Some(directory) => classify(bytes, directory),
        None => Detection {
            format: Format::DamagedPackage,
            evidence: Evidence {
                offset: bytes.len(),
                what: "the end of the bytes, before a readable end-of-central-directory record",
            },
        },
    }
}

/// The parts a package can be recognised by, each with the offset of the
/// central directory entry that named it.
#[derive(Default)]
struct Named {
    workbook_markup: Option<usize>,
    workbook_binary: Option<usize>,
    macro_project: Option<usize>,
    word_or_presentation: Option<usize>,
    mimetype: Option<(usize, MimeType)>,
}

#[derive(Clone, Copy)]
enum MimeType {
    Spreadsheet,
    Other,
}

fn classify(bytes: &[u8], directory: usize) -> Detection {
    let mut named = Named::default();
    let mut entries = 0usize;
    for entry in Entries::new(bytes, directory) {
        entries += 1;
        match entry.name {
            b"xl/workbook.xml" => named.workbook_markup.get_or_insert(entry.offset),
            b"xl/workbook.bin" => named.workbook_binary.get_or_insert(entry.offset),
            b"xl/vbaProject.bin" => named.macro_project.get_or_insert(entry.offset),
            b"mimetype" => {
                let kind = match stored_content(bytes, &entry) {
                    Some(b"application/vnd.oasis.opendocument.spreadsheet")
                    | Some(b"application/vnd.oasis.opendocument.spreadsheet-template") => {
                        MimeType::Spreadsheet
                    }
                    _ => MimeType::Other,
                };
                named.mimetype.get_or_insert((entry.offset, kind));
                continue;
            }
            name if name.starts_with(b"word/") || name.starts_with(b"ppt/") => {
                named.word_or_presentation.get_or_insert(entry.offset)
            }
            _ => continue,
        };
    }

    let at = |offset: usize, what: &'static str| Evidence { offset, what };

    if let Some(offset) = named.workbook_markup {
        let macro_enabled = named.macro_project.is_some();
        return Detection {
            format: Format::SpreadsheetMl { macro_enabled },
            evidence: at(offset, "the workbook part named in the central directory"),
        };
    }
    if let Some(offset) = named.workbook_binary {
        return Detection {
            format: Format::BinarySpreadsheetMl,
            evidence: at(
                offset,
                "the binary workbook part named in the central directory",
            ),
        };
    }
    if let Some((offset, kind)) = named.mimetype {
        return match kind {
            MimeType::Spreadsheet => Detection {
                format: Format::OpenDocumentSpreadsheet,
                evidence: at(offset, "the mimetype entry declaring a spreadsheet"),
            },
            MimeType::Other => Detection {
                format: Format::OtherOpenDocument,
                evidence: at(
                    offset,
                    "the mimetype entry declaring something other than a spreadsheet",
                ),
            },
        };
    }
    if let Some(offset) = named.word_or_presentation {
        return Detection {
            format: Format::OtherOfficePackage,
            evidence: at(
                offset,
                "a word-processing or presentation part named in the central directory",
            ),
        };
    }
    Detection {
        format: Format::UnknownPackage,
        evidence: at(
            directory,
            if entries == 0 {
                "a central directory naming no part at all"
            } else {
                "a central directory naming no part this project can name"
            },
        ),
    }
}

/// One central directory entry, borrowed from the bytes.
struct Entry<'a> {
    /// Offset of the entry's header in the bytes.
    offset: usize,
    name: &'a [u8],
    method: u16,
    uncompressed_size: u32,
    local_header: u32,
}

/// The central directory, walked one entry at a time until the signature stops
/// matching or the bytes end. Each step advances by at least a header, so the
/// walk ends.
struct Entries<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> Entries<'a> {
    fn new(bytes: &'a [u8], directory: usize) -> Self {
        Entries {
            bytes,
            position: directory,
        }
    }
}

impl<'a> Iterator for Entries<'a> {
    type Item = Entry<'a>;

    fn next(&mut self) -> Option<Entry<'a>> {
        const SIGNATURE: [u8; 4] = [0x50, 0x4B, 0x01, 0x02];
        const HEADER: usize = 46;

        let header = self.bytes.get(self.position..self.position + HEADER)?;
        if header[..4] != SIGNATURE {
            return None;
        }
        let name_length = usize::from(u16_at(header, 28));
        let extra_length = usize::from(u16_at(header, 30));
        let comment_length = usize::from(u16_at(header, 32));
        let name_start = self.position + HEADER;
        let name = self.bytes.get(name_start..name_start + name_length)?;
        let entry = Entry {
            offset: self.position,
            name,
            method: u16_at(header, 10),
            uncompressed_size: u32_at(header, 24),
            local_header: u32_at(header, 42),
        };
        self.position = name_start + name_length + extra_length + comment_length;
        Some(entry)
    }
}

/// The content of an entry that is stored rather than compressed, read through
/// its local header. `None` for a compressed entry or one pointing outside the
/// bytes, because neither can be read without trusting the archive.
fn stored_content<'a>(bytes: &'a [u8], entry: &Entry<'_>) -> Option<&'a [u8]> {
    const SIGNATURE: [u8; 4] = [0x50, 0x4B, 0x03, 0x04];
    const HEADER: usize = 30;

    if entry.method != 0 {
        return None;
    }
    let start = usize::try_from(entry.local_header).ok()?;
    let header = bytes.get(start..start.checked_add(HEADER)?)?;
    if header[..4] != SIGNATURE {
        return None;
    }
    let name_length = usize::from(u16_at(header, 26));
    let extra_length = usize::from(u16_at(header, 28));
    let data = start
        .checked_add(HEADER)?
        .checked_add(name_length)?
        .checked_add(extra_length)?;
    let size = usize::try_from(entry.uncompressed_size).ok()?;
    bytes.get(data..data.checked_add(size)?)
}

/// The offset of the central directory, found through the end-of-central-
/// directory record and, where that record says the archive is too large for
/// its fields, through the zip64 record it points at.
fn central_directory(bytes: &[u8]) -> Option<usize> {
    const END_SIGNATURE: [u8; 4] = [0x50, 0x4B, 0x05, 0x06];
    const END_LENGTH: usize = 22;
    const LONGEST_COMMENT: usize = 0xFFFF;
    const LOCATOR_SIGNATURE: [u8; 4] = [0x50, 0x4B, 0x06, 0x07];
    const LOCATOR_LENGTH: usize = 20;
    const ZIP64_SIGNATURE: [u8; 4] = [0x50, 0x4B, 0x06, 0x06];
    const ZIP64_LENGTH: usize = 56;

    let last = bytes.len().checked_sub(END_LENGTH)?;
    let earliest = last.saturating_sub(LONGEST_COMMENT);
    let end = (earliest..=last)
        .rev()
        .find(|&at| bytes[at..at + 4] == END_SIGNATURE)?;
    let record = &bytes[end..end + END_LENGTH];
    let entries = u16_at(record, 10);
    let offset = u32_at(record, 16);

    if entries != 0xFFFF && offset != 0xFFFF_FFFF {
        let offset = usize::try_from(offset).ok()?;
        return (offset < end).then_some(offset);
    }

    let locator_at = end.checked_sub(LOCATOR_LENGTH)?;
    let locator = &bytes[locator_at..end];
    if locator[..4] != LOCATOR_SIGNATURE {
        return None;
    }
    let zip64_at = usize::try_from(u64_at(locator, 8)).ok()?;
    let zip64 = bytes.get(zip64_at..zip64_at.checked_add(ZIP64_LENGTH)?)?;
    if zip64[..4] != ZIP64_SIGNATURE {
        return None;
    }
    let offset = usize::try_from(u64_at(zip64, 48)).ok()?;
    (offset < zip64_at).then_some(offset)
}

fn u16_at(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}

fn u32_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
}

fn u64_at(bytes: &[u8], at: usize) -> u64 {
    let mut word = [0u8; 8];
    word.copy_from_slice(&bytes[at..at + 8]);
    u64::from_le_bytes(word)
}
