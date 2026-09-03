//! Number formats: the grammar between what a cell contains and what it shows.
//!
//! A format code is a small language. Up to four sections separated by `;`,
//! each optionally guarded by a condition and coloured, holding digit
//! placeholders, a decimal point, group separators that also scale, a percent
//! sign, an exponent, a fraction bar, quoted and escaped literal text, a fill
//! directive and a width-skip directive, a text placeholder, and the date and
//! time codes. Issue #19 asks for all of that, and this module is the half of
//! it that needs no workbook: the parser over the code, the arithmetic over a
//! serial number under both epoch bases, and the formatter that puts a value
//! through a section.
//!
//! Two records decide what this does rather than the code deciding it.
//! `docs/decisions/0015-locale.md` gives the precedence between the format
//! string, the workbook, an explicit setting and a fallback fixed in that
//! record, and keeps the host out of it: nothing here reads an environment
//! variable or a regional setting, and the fallback `Locale::fallback` is the
//! one the record writes down. A locale identifier the format string itself
//! carries is reported on the parsed format so the caller can apply step 1 of
//! that precedence once locale data exists; the data is a dependency that
//! arrives with the commit needing it, and none is here.
//! `docs/decisions/0008-calculation.md` puts every rounding at the display
//! boundary, which is here: the value arrives unrounded, the section decides
//! how many digits are shown, and the rounding is done on the fifteen
//! significant decimal digits the incumbent shows for a general number, half
//! away from zero, on the decimal digits rather than on the binary value.
//!
//! What is deliberately reproduced. The older epoch base counts a day that
//! does not exist, the twenty-ninth of February 1900, and a serial of sixty is
//! shown as that day rather than corrected, because compatibility means
//! matching it. `serial_to_civil` is where that case lives and the test that
//! names it is in `crates/model/tests/number_format.rs`.
//!
//! Where it stops. A format the parser does not understand is a typed
//! `Unreadable` rather than a silent fallback; recording it as unrepresented
//! content is issue #18's register, which does not exist yet, so the caller
//! holds the error until it does. The width-dependent contraction of a general
//! number into a narrow column is a fitting question and belongs to the
//! renderer's fitting stage, so `General` here is the width-independent form.
//! The width-skip directive is rendered as one space, because the width of a
//! character is a font metric this component does not hold, and the fill
//! directive is reported beside the text rather than expanded, for the same
//! reason. Calendar and era codes are refused as unreadable rather than
//! approximated.

use std::fmt;

/// Why a format code could not be read. Every variant names where in the code
/// the reading stopped, so a refusal is locatable without a second run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unreadable {
    /// An empty format code, which formats nothing.
    Empty,
    /// More than the four sections the grammar allows.
    TooManySections { found: usize },
    /// A built-in format number the specification leaves to the locale, or
    /// does not define at all.
    BuiltInNotFixed { id: u16 },
    /// A quote opened at this character offset and never closed.
    UnterminatedQuote { at: usize },
    /// A bracket opened at this character offset and never closed.
    UnterminatedBracket { at: usize },
    /// A bracket whose content is neither a condition, a colour, a locale tag
    /// nor an elapsed-time code.
    UnknownBracket { at: usize, text: String },
    /// An escape at the end of the code, with nothing to escape.
    DanglingEscape { at: usize },
    /// A fill or skip directive at the end of the code, naming no character.
    DanglingDirective { at: usize },
    /// A calendar or era code, which this project does not read.
    CalendarCode { at: usize, code: char },
    /// A section mixing date codes with number placeholders, which the
    /// grammar does not allow.
    MixedDateAndNumber { section: usize },
    /// A colour number outside the palette of fifty-six.
    ColourOutOfRange { at: usize, number: u32 },
}

impl fmt::Display for Unreadable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Unreadable::Empty => write!(f, "the format code is empty"),
            Unreadable::TooManySections { found } => {
                write!(
                    f,
                    "the format code has {found} sections and the grammar allows four"
                )
            }
            Unreadable::BuiltInNotFixed { id } => write!(
                f,
                "built-in format {id} is not one the specification fixes; its meaning depends on the locale"
            ),
            Unreadable::UnterminatedQuote { at } => {
                write!(f, "a quote opened at character {at} is never closed")
            }
            Unreadable::UnterminatedBracket { at } => {
                write!(f, "a bracket opened at character {at} is never closed")
            }
            Unreadable::UnknownBracket { at, text } => write!(
                f,
                "the bracket at character {at} holds `{text}`, which is neither a condition, a colour, a locale tag nor an elapsed-time code"
            ),
            Unreadable::DanglingEscape { at } => {
                write!(f, "the escape at character {at} has nothing after it")
            }
            Unreadable::DanglingDirective { at } => {
                write!(f, "the directive at character {at} names no character")
            }
            Unreadable::CalendarCode { at, code } => write!(
                f,
                "the code `{code}` at character {at} selects a calendar or an era, which this project does not read"
            ),
            Unreadable::MixedDateAndNumber { section } => write!(
                f,
                "section {section} mixes date codes with number placeholders"
            ),
            Unreadable::ColourOutOfRange { at, number } => write!(
                f,
                "the colour number {number} at character {at} is outside the palette of fifty-six"
            ),
        }
    }
}

impl std::error::Error for Unreadable {}

/// A colour named in a section.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Colour {
    Black,
    Blue,
    Cyan,
    Green,
    Magenta,
    Red,
    White,
    Yellow,
    /// An index into the palette of fifty-six, one-based as the code writes it.
    Indexed(u8),
}

/// A condition guarding a section.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Condition {
    pub operator: Operator,
    pub operand: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
    Equal,
    NotEqual,
}

impl Condition {
    fn holds(self, value: f64) -> bool {
        match self.operator {
            Operator::Less => value < self.operand,
            Operator::LessOrEqual => value <= self.operand,
            Operator::Greater => value > self.operand,
            Operator::GreaterOrEqual => value >= self.operand,
            Operator::Equal => value == self.operand,
            Operator::NotEqual => value != self.operand,
        }
    }
}

/// A digit placeholder: `0` pads with a zero, `?` with a space, `#` with
/// nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placeholder {
    Zero,
    Space,
    Nothing,
}

/// A date or time code, with the number of letters that wrote it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateCode {
    /// `yy` or `yyyy`.
    Year {
        four: bool,
    },
    /// `m` to `mmmmm`: number, padded number, short name, long name, initial.
    Month {
        letters: u8,
    },
    /// `d` to `dddd`: number, padded number, short weekday, long weekday.
    Day {
        letters: u8,
    },
    Hour {
        padded: bool,
    },
    Minute {
        padded: bool,
    },
    Second {
        padded: bool,
    },
    /// `AM/PM` or `A/P`; `long` is the former.
    AmPm {
        long: bool,
        upper: bool,
    },
    /// `[h]`, `[m]`, `[s]`: a total that runs past its unit.
    ElapsedHours,
    ElapsedMinutes,
    ElapsedSeconds,
    /// `.0`, `.00`, `.000` after a seconds code.
    FractionalSeconds {
        digits: u8,
    },
}

/// One token of a section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Digit(Placeholder),
    Point,
    /// A `,` between digit placeholders, which groups.
    Group,
    /// A `,` after the last digit placeholder, which scales by a thousand.
    Scale,
    Percent,
    /// `E+` or `E-` with the sign it wrote; `e` is kept as written.
    Exponent {
        always_signed: bool,
        upper: bool,
    },
    /// The `/` of a fraction.
    FractionBar,
    /// Literal text, quoted, escaped or one of the characters the grammar
    /// passes through.
    Literal(String),
    /// `*x`: repeat `x` to fill the cell.
    Fill(char),
    /// `_x`: leave the width of `x`.
    Skip(char),
    /// `@`.
    Text,
    General,
    Date(DateCode),
}

/// What kind of value a section formats, decided by the tokens it holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Number,
    Date,
    Text,
    General,
}

/// One section of a format code.
#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    pub condition: Option<Condition>,
    pub colour: Option<Colour>,
    pub kind: Kind,
    pub tokens: Vec<Token>,
}

/// A parsed format code.
#[derive(Debug, Clone, PartialEq)]
pub struct Format {
    pub sections: Vec<Section>,
    /// The locale identifier a `[$-xxx]` tag carried, if any: step 1 of the
    /// precedence in `docs/decisions/0015-locale.md`, reported for the caller
    /// to resolve because no locale data lives here.
    pub locale_tag: Option<u32>,
}

/// The two epoch bases a workbook can declare.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Epoch {
    /// Serial 1 is the first of January 1900, and serial 60 is a day that
    /// does not exist.
    Windows1900,
    /// Serial 0 is the first of January 1904.
    Mac1904,
}

/// The separators and names a run formats with. Step 4 of the precedence in
/// `docs/decisions/0015-locale.md` is `Locale::fallback`; the other steps hand
/// in a different one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Locale {
    pub decimal: char,
    pub group: char,
    pub months_long: [&'static str; 12],
    pub months_short: [&'static str; 12],
    pub days_long: [&'static str; 7],
    pub days_short: [&'static str; 7],
    pub am: &'static str,
    pub pm: &'static str,
}

impl Locale {
    /// The fallback fixed in `docs/decisions/0015-locale.md`: a point, a
    /// comma, English names.
    #[must_use]
    pub fn fallback() -> Locale {
        Locale {
            decimal: '.',
            group: ',',
            months_long: [
                "January",
                "February",
                "March",
                "April",
                "May",
                "June",
                "July",
                "August",
                "September",
                "October",
                "November",
                "December",
            ],
            months_short: [
                "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
            ],
            days_long: [
                "Sunday",
                "Monday",
                "Tuesday",
                "Wednesday",
                "Thursday",
                "Friday",
                "Saturday",
            ],
            days_short: ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"],
            am: "AM",
            pm: "PM",
        }
    }
}

/// A fill directive, reported beside the text for the renderer to expand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fill {
    /// Character offset in the text where the fill goes.
    pub at: usize,
    pub with: char,
}

/// What a section produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Formatted {
    pub text: String,
    pub colour: Option<Colour>,
    pub fill: Option<Fill>,
}

/// The format codes the specification fixes by number. Every other number is
/// left to the locale by the specification and is refused here as
/// `Unreadable::BuiltInNotFixed`.
#[must_use]
pub fn builtin_code(id: u16) -> Option<&'static str> {
    Some(match id {
        0 => "General",
        1 => "0",
        2 => "0.00",
        3 => "#,##0",
        4 => "#,##0.00",
        9 => "0%",
        10 => "0.00%",
        11 => "0.00E+00",
        12 => "# ?/?",
        13 => "# ??/??",
        14 => "mm-dd-yy",
        15 => "d-mmm-yy",
        16 => "d-mmm",
        17 => "mmm-yy",
        18 => "h:mm AM/PM",
        19 => "h:mm:ss AM/PM",
        20 => "h:mm",
        21 => "h:mm:ss",
        22 => "m/d/yy h:mm",
        37 => "#,##0 ;(#,##0)",
        38 => "#,##0 ;[Red](#,##0)",
        39 => "#,##0.00;(#,##0.00)",
        40 => "#,##0.00;[Red](#,##0.00)",
        45 => "mm:ss",
        46 => "[h]:mm:ss",
        47 => "mmss.0",
        48 => "##0.0E+0",
        49 => "@",
        _ => return None,
    })
}

/// Parse a built-in format by its number.
///
/// # Errors
///
/// A number the specification does not fix.
pub fn builtin(id: u16) -> Result<Format, Unreadable> {
    match builtin_code(id) {
        Some(code) => parse(code),
        None => Err(Unreadable::BuiltInNotFixed { id }),
    }
}

/// Parse a format code.
///
/// # Errors
///
/// Any shape the grammar does not allow, named with where it was met.
pub fn parse(code: &str) -> Result<Format, Unreadable> {
    if code.is_empty() {
        return Err(Unreadable::Empty);
    }
    let chars: Vec<char> = code.chars().collect();
    let mut cursor = Cursor { chars, at: 0 };
    let mut sections = Vec::new();
    let mut locale_tag = None;
    loop {
        let (section, tag) = read_section(&mut cursor, sections.len())?;
        if tag.is_some() {
            locale_tag = locale_tag.or(tag);
        }
        sections.push(section);
        if cursor.peek() == Some(';') {
            cursor.bump();
            if cursor.peek().is_none() {
                // A trailing separator writes an empty final section, which
                // shows nothing for the values it would take.
                sections.push(Section {
                    condition: None,
                    colour: None,
                    kind: Kind::Number,
                    tokens: Vec::new(),
                });
                break;
            }
        } else {
            break;
        }
    }
    if sections.len() > 4 {
        return Err(Unreadable::TooManySections {
            found: sections.len(),
        });
    }
    Ok(Format {
        sections,
        locale_tag,
    })
}

struct Cursor {
    chars: Vec<char>,
    at: usize,
}

impl Cursor {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.at).copied()
    }

    fn peek_ahead(&self, by: usize) -> Option<char> {
        self.chars.get(self.at.checked_add(by)?).copied()
    }

    fn bump(&mut self) {
        self.at = self.at.saturating_add(1);
    }

    fn take(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.bump();
        Some(c)
    }

    /// How many of `letter`, in either case, stand at the cursor.
    fn run_of(&self, letter: char) -> usize {
        self.chars
            .get(self.at..)
            .map(|rest| {
                rest.iter()
                    .take_while(|c| c.eq_ignore_ascii_case(&letter))
                    .count()
            })
            .unwrap_or(0)
    }

    fn skip(&mut self, count: usize) {
        self.at = self.at.saturating_add(count);
    }

    fn starts_with_ignoring_case(&self, text: &str) -> bool {
        let mut by = 0usize;
        for expected in text.chars() {
            match self.peek_ahead(by) {
                Some(found) if found.eq_ignore_ascii_case(&expected) => {}
                _ => return false,
            }
            by = by.saturating_add(1);
        }
        true
    }
}

/// The characters the grammar passes through as literal text without quotes.
const PASSED_THROUGH: &str = "$+-/():!^&'~{}<>= ";

fn read_section(cursor: &mut Cursor, index: usize) -> Result<(Section, Option<u32>), Unreadable> {
    let mut condition = None;
    let mut colour = None;
    let mut locale_tag = None;
    let mut tokens: Vec<Token> = Vec::new();
    let mut saw_date = false;
    let mut saw_number = false;
    let mut saw_text = false;
    let mut saw_general = false;

    while let Some(c) = cursor.peek() {
        if c == ';' {
            break;
        }
        let at = cursor.at;
        match c {
            '"' => {
                cursor.bump();
                let mut text = String::new();
                loop {
                    match cursor.take() {
                        Some('"') => break,
                        Some(inner) => text.push(inner),
                        None => return Err(Unreadable::UnterminatedQuote { at }),
                    }
                }
                tokens.push(Token::Literal(text));
            }
            '\\' => {
                cursor.bump();
                match cursor.take() {
                    Some(escaped) => tokens.push(Token::Literal(escaped.to_string())),
                    None => return Err(Unreadable::DanglingEscape { at }),
                }
            }
            '*' | '_' => {
                cursor.bump();
                match cursor.take() {
                    Some(named) if c == '*' => tokens.push(Token::Fill(named)),
                    Some(named) => tokens.push(Token::Skip(named)),
                    None => return Err(Unreadable::DanglingDirective { at }),
                }
            }
            '[' => {
                cursor.bump();
                let mut text = String::new();
                loop {
                    match cursor.take() {
                        Some(']') => break,
                        Some(inner) => text.push(inner),
                        None => return Err(Unreadable::UnterminatedBracket { at }),
                    }
                }
                match read_bracket(&text, at)? {
                    Bracket::Condition(found) => condition = Some(found),
                    Bracket::Colour(found) => colour = Some(found),
                    Bracket::Locale { currency, tag } => {
                        if !currency.is_empty() {
                            tokens.push(Token::Literal(currency));
                        }
                        locale_tag = locale_tag.or(tag);
                    }
                    Bracket::Elapsed(code) => {
                        saw_date = true;
                        tokens.push(Token::Date(code));
                    }
                }
            }
            '0' => {
                cursor.bump();
                saw_number = true;
                tokens.push(Token::Digit(Placeholder::Zero));
            }
            '?' => {
                cursor.bump();
                saw_number = true;
                tokens.push(Token::Digit(Placeholder::Space));
            }
            '#' => {
                cursor.bump();
                saw_number = true;
                tokens.push(Token::Digit(Placeholder::Nothing));
            }
            '.' => {
                cursor.bump();
                // A point after a seconds code, followed by zeros, is the
                // fractional-seconds code rather than a decimal point.
                let after_seconds =
                    matches!(tokens.last(), Some(Token::Date(DateCode::Second { .. })));
                let zeros = cursor.run_of('0');
                if after_seconds && zeros > 0 {
                    cursor.skip(zeros);
                    tokens.push(Token::Date(DateCode::FractionalSeconds {
                        digits: u8::try_from(zeros.min(3)).unwrap_or(3),
                    }));
                } else {
                    tokens.push(Token::Point);
                }
            }
            ',' => {
                cursor.bump();
                tokens.push(Token::Group);
            }
            '%' => {
                cursor.bump();
                saw_number = true;
                tokens.push(Token::Percent);
            }
            '/' => {
                cursor.bump();
                if saw_date && !saw_number {
                    tokens.push(Token::Literal("/".to_string()));
                } else {
                    tokens.push(Token::FractionBar);
                }
            }
            '@' => {
                cursor.bump();
                saw_text = true;
                tokens.push(Token::Text);
            }
            'E' | 'e' => {
                let sign = cursor.peek_ahead(1);
                if matches!(sign, Some('+') | Some('-')) && !saw_date {
                    cursor.skip(2);
                    saw_number = true;
                    tokens.push(Token::Exponent {
                        always_signed: sign == Some('+'),
                        upper: c == 'E',
                    });
                } else {
                    // A bare `e` is the era code the grammar reserves.
                    return Err(Unreadable::CalendarCode { at, code: c });
                }
            }
            'G' | 'g' if cursor.starts_with_ignoring_case("General") => {
                cursor.skip(7);
                saw_general = true;
                tokens.push(Token::General);
            }
            'A' | 'a'
                if cursor.starts_with_ignoring_case("AM/PM")
                    || cursor.starts_with_ignoring_case("A/P") =>
            {
                let long = cursor.starts_with_ignoring_case("AM/PM");
                let upper = c == 'A';
                cursor.skip(if long { 5 } else { 3 });
                saw_date = true;
                tokens.push(Token::Date(DateCode::AmPm { long, upper }));
            }
            'y' | 'Y' => {
                let run = cursor.run_of('y');
                cursor.skip(run);
                saw_date = true;
                tokens.push(Token::Date(DateCode::Year { four: run > 2 }));
            }
            'm' | 'M' => {
                let run = cursor.run_of('m');
                cursor.skip(run);
                saw_date = true;
                // Month for now; a second pass turns it into a minute where it
                // stands beside an hour or a second.
                tokens.push(Token::Date(DateCode::Month {
                    letters: u8::try_from(run.min(5)).unwrap_or(5),
                }));
            }
            'd' | 'D' => {
                let run = cursor.run_of('d');
                cursor.skip(run);
                saw_date = true;
                tokens.push(Token::Date(DateCode::Day {
                    letters: u8::try_from(run.min(4)).unwrap_or(4),
                }));
            }
            'h' | 'H' => {
                let run = cursor.run_of('h');
                cursor.skip(run);
                saw_date = true;
                tokens.push(Token::Date(DateCode::Hour { padded: run > 1 }));
            }
            's' | 'S' => {
                let run = cursor.run_of('s');
                cursor.skip(run);
                saw_date = true;
                tokens.push(Token::Date(DateCode::Second { padded: run > 1 }));
            }
            'b' | 'B' | 'g' | 'G' => {
                return Err(Unreadable::CalendarCode { at, code: c });
            }
            other if PASSED_THROUGH.contains(other) => {
                cursor.bump();
                tokens.push(Token::Literal(other.to_string()));
            }
            other => {
                // Anything else is carried through as text, which is what the
                // incumbent does with a letter the grammar does not claim.
                cursor.bump();
                tokens.push(Token::Literal(other.to_string()));
            }
        }
    }

    if saw_date && saw_number {
        return Err(Unreadable::MixedDateAndNumber { section: index });
    }
    if saw_date {
        // A comma or a point among date codes is punctuation, not a grouping
        // or a decimal point.
        for token in &mut tokens {
            match token {
                Token::Group => *token = Token::Literal(",".to_string()),
                Token::Point => *token = Token::Literal(".".to_string()),
                _ => {}
            }
        }
    }
    resolve_minutes(&mut tokens);
    classify_commas(&mut tokens);

    let kind = if saw_date {
        Kind::Date
    } else if saw_text && !saw_number {
        Kind::Text
    } else if saw_general && !saw_number {
        Kind::General
    } else {
        Kind::Number
    };

    Ok((
        Section {
            condition,
            colour,
            kind,
            tokens,
        },
        locale_tag,
    ))
}

enum Bracket {
    Condition(Condition),
    Colour(Colour),
    Locale { currency: String, tag: Option<u32> },
    Elapsed(DateCode),
}

fn read_bracket(text: &str, at: usize) -> Result<Bracket, Unreadable> {
    let lower = text.to_ascii_lowercase();
    let named = match lower.as_str() {
        "black" => Some(Colour::Black),
        "blue" => Some(Colour::Blue),
        "cyan" => Some(Colour::Cyan),
        "green" => Some(Colour::Green),
        "magenta" => Some(Colour::Magenta),
        "red" => Some(Colour::Red),
        "white" => Some(Colour::White),
        "yellow" => Some(Colour::Yellow),
        _ => None,
    };
    if let Some(colour) = named {
        return Ok(Bracket::Colour(colour));
    }
    if let Some(number) = lower
        .strip_prefix("color")
        .or_else(|| lower.strip_prefix("colour"))
        && let Ok(number) = number.parse::<u32>()
    {
        return match u8::try_from(number) {
            Ok(index) if (1..=56).contains(&index) => Ok(Bracket::Colour(Colour::Indexed(index))),
            _ => Err(Unreadable::ColourOutOfRange { at, number }),
        };
    }
    if let Some(rest) = text.strip_prefix('$') {
        let (currency, tag) = match rest.rsplit_once('-') {
            Some((currency, hex)) => (currency.to_string(), u32::from_str_radix(hex, 16).ok()),
            None => (rest.to_string(), None),
        };
        return Ok(Bracket::Locale { currency, tag });
    }
    if lower.chars().all(|c| c == 'h') && !lower.is_empty() {
        return Ok(Bracket::Elapsed(DateCode::ElapsedHours));
    }
    if lower.chars().all(|c| c == 'm') && !lower.is_empty() {
        return Ok(Bracket::Elapsed(DateCode::ElapsedMinutes));
    }
    if lower.chars().all(|c| c == 's') && !lower.is_empty() {
        return Ok(Bracket::Elapsed(DateCode::ElapsedSeconds));
    }
    let (operator, rest) = if let Some(rest) = text.strip_prefix("<=") {
        (Operator::LessOrEqual, rest)
    } else if let Some(rest) = text.strip_prefix(">=") {
        (Operator::GreaterOrEqual, rest)
    } else if let Some(rest) = text.strip_prefix("<>") {
        (Operator::NotEqual, rest)
    } else if let Some(rest) = text.strip_prefix('<') {
        (Operator::Less, rest)
    } else if let Some(rest) = text.strip_prefix('>') {
        (Operator::Greater, rest)
    } else if let Some(rest) = text.strip_prefix('=') {
        (Operator::Equal, rest)
    } else {
        return Err(Unreadable::UnknownBracket {
            at,
            text: text.to_string(),
        });
    };
    match rest.trim().parse::<f64>() {
        Ok(operand) => Ok(Bracket::Condition(Condition { operator, operand })),
        Err(_) => Err(Unreadable::UnknownBracket {
            at,
            text: text.to_string(),
        }),
    }
}

/// An `m` beside an hour or a second is a minute.
fn resolve_minutes(tokens: &mut [Token]) {
    let codes: Vec<Option<DateCode>> = tokens
        .iter()
        .map(|t| match t {
            Token::Date(code) => Some(*code),
            _ => None,
        })
        .collect();
    let mut previous: Option<DateCode> = None;
    for (index, token) in tokens.iter_mut().enumerate() {
        if let Token::Date(DateCode::Month { letters }) = token {
            let next = codes
                .iter()
                .skip(index.saturating_add(1))
                .flatten()
                .next()
                .copied();
            let after_hour = matches!(
                previous,
                Some(DateCode::Hour { .. }) | Some(DateCode::ElapsedHours)
            );
            let before_second = matches!(next, Some(DateCode::Second { .. }));
            if (after_hour || before_second) && *letters <= 2 {
                *token = Token::Date(DateCode::Minute {
                    padded: *letters == 2,
                });
            }
        }
        if let Token::Date(code) = token {
            previous = Some(*code);
        }
    }
}

/// A comma with a digit placeholder after it in the integer part groups; any
/// other comma scales by a thousand, whether it follows the integer digits
/// directly or the fraction digits.
fn classify_commas(tokens: &mut [Token]) {
    let point = tokens
        .iter()
        .position(|t| {
            matches!(
                t,
                Token::Point | Token::Exponent { .. } | Token::FractionBar
            )
        })
        .unwrap_or(tokens.len());
    let digit_positions: Vec<usize> = tokens
        .iter()
        .enumerate()
        .filter(|(index, t)| *index < point && matches!(t, Token::Digit(_)))
        .map(|(index, _)| index)
        .collect();
    for (index, token) in tokens.iter_mut().enumerate() {
        if !matches!(token, Token::Group) {
            continue;
        }
        let groups = index < point && digit_positions.iter().any(|at| *at > index);
        if !groups {
            *token = Token::Scale;
        }
    }
}

// The decimal representation formatting works on. Fifteen significant digits,
// as the incumbent shows for a general number, so the rounding a section
// applies is decimal rounding on those digits and not binary rounding on the
// value.

#[derive(Debug, Clone)]
struct Decimal {
    /// Significant digits, most significant first, no trailing zeros. Empty
    /// for zero.
    digits: Vec<u8>,
    /// The value is `0.d1 d2 ... dn` times ten to this power.
    exponent: i32,
}

const SIGNIFICANT: usize = 15;

impl Decimal {
    fn of(value: f64) -> Decimal {
        if value == 0.0 || !value.is_finite() {
            return Decimal {
                digits: Vec::new(),
                exponent: 0,
            };
        }
        // `{:.14e}` writes fifteen significant digits, correctly rounded from
        // the binary value.
        let written = format!("{:.*e}", SIGNIFICANT.saturating_sub(1), value.abs());
        let (mantissa, exponent) = written.split_once('e').unwrap_or((written.as_str(), "0"));
        let digits: Vec<u8> = mantissa
            .chars()
            .filter_map(|c| c.to_digit(10))
            .filter_map(|d| u8::try_from(d).ok())
            .collect();
        let exponent = exponent.parse::<i32>().unwrap_or(0).saturating_add(1);
        let mut decimal = Decimal { digits, exponent };
        decimal.trim();
        decimal
    }

    fn is_zero(&self) -> bool {
        self.digits.is_empty()
    }

    fn trim(&mut self) {
        while self.digits.last() == Some(&0) {
            self.digits.pop();
        }
        if self.digits.is_empty() {
            self.exponent = 0;
        }
    }

    fn scale(&mut self, by: i32) {
        if !self.is_zero() {
            self.exponent = self.exponent.saturating_add(by);
        }
    }

    /// Round half away from zero to `places` digits after the point.
    fn round_to_places(&mut self, places: usize) {
        let places = i32::try_from(places).unwrap_or(i32::MAX);
        let keep = self.exponent.saturating_add(places);
        self.round_to_significant(keep);
    }

    /// Keep the first `keep` significant digits, rounding half away from
    /// zero. A `keep` at or below zero rounds to nothing or to one unit.
    fn round_to_significant(&mut self, keep: i32) {
        if self.is_zero() {
            return;
        }
        if keep < 0 {
            self.digits.clear();
            self.exponent = 0;
            return;
        }
        let keep = usize::try_from(keep).unwrap_or(usize::MAX);
        if keep >= self.digits.len() {
            return;
        }
        let round_up = self.digits.get(keep).is_some_and(|d| *d >= 5);
        self.digits.truncate(keep);
        if round_up {
            let mut carry = true;
            for digit in self.digits.iter_mut().rev() {
                if !carry {
                    break;
                }
                if *digit == 9 {
                    *digit = 0;
                } else {
                    *digit = digit.saturating_add(1);
                    carry = false;
                }
            }
            if carry {
                self.digits.insert(0, 1);
                self.exponent = self.exponent.saturating_add(1);
                self.digits.truncate(keep.saturating_add(1));
            }
        }
        self.trim();
    }

    /// The integer digits, no leading zeros; empty for a value below one.
    fn integer_digits(&self) -> Vec<u8> {
        if self.exponent <= 0 {
            return Vec::new();
        }
        let count = usize::try_from(self.exponent).unwrap_or(0);
        let mut out: Vec<u8> = self.digits.iter().take(count).copied().collect();
        while out.len() < count {
            out.push(0);
        }
        out
    }

    /// The first `places` digits after the point, zero padded.
    fn fraction_digits(&self, places: usize) -> Vec<u8> {
        let mut out = Vec::new();
        let leading_zeros = usize::try_from(self.exponent.saturating_neg()).unwrap_or(0);
        let skip = usize::try_from(self.exponent).unwrap_or(0);
        out.extend(std::iter::repeat_n(0u8, leading_zeros.min(places)));
        out.extend(self.digits.iter().skip(skip).copied());
        out.truncate(places);
        while out.len() < places {
            out.push(0);
        }
        out
    }

    /// The value as a floating-point number again, for the fraction search.
    fn to_f64(&self) -> f64 {
        let mut text = String::from("0.");
        for digit in &self.digits {
            text.push(char::from(b'0'.saturating_add(*digit)));
        }
        text.push_str(&format!("e{}", self.exponent));
        text.parse::<f64>().unwrap_or(0.0)
    }
}

// Selecting the section a value goes through.

fn section_for_number(format: &Format, value: f64) -> Option<(&Section, bool)> {
    let numeric: Vec<&Section> = format
        .sections
        .iter()
        .filter(|s| s.kind != Kind::Text)
        .collect();
    if numeric.is_empty() {
        // A format holding only a text section shows a number through it.
        return format.sections.first().map(|s| (s, true));
    }
    if numeric.iter().any(|s| s.condition.is_some()) {
        // A section chosen by its condition formats the value as it is. The
        // sign is kept, which is a claim about the incumbent rather than a
        // reading of a document, and the corpus is where it is checked.
        for section in &numeric {
            if let Some(condition) = section.condition
                && condition.holds(value)
            {
                return Some((section, true));
            }
        }
        return numeric
            .iter()
            .find(|s| s.condition.is_none())
            .or_else(|| numeric.last())
            .map(|s| (*s, true));
    }
    let count = numeric.len();
    let (index, keep_sign) = if count >= 3 {
        if value > 0.0 {
            (0, true)
        } else if value < 0.0 {
            (1, false)
        } else {
            (2, true)
        }
    } else if count == 2 {
        if value < 0.0 { (1, false) } else { (0, true) }
    } else {
        (0, true)
    };
    numeric.get(index).map(|s| (*s, keep_sign))
}

impl Format {
    /// Put a number through the format.
    ///
    /// The text is what the section produced. Where the value cannot be shown,
    /// because the section is a date and the serial is outside the calendar,
    /// the text is `None`, which a renderer shows as the cell filled with
    /// hashes.
    #[must_use]
    pub fn apply_number(&self, value: f64, epoch: Epoch, locale: &Locale) -> Option<Formatted> {
        let (section, keep_sign) = section_for_number(self, value)?;
        let mut sink = Sink::default();
        let magnitude = if keep_sign { value } else { value.abs() };
        match section.kind {
            Kind::Number => format_number(section, magnitude, locale, &mut sink),
            Kind::General => format_general(section, magnitude, locale, &mut sink),
            Kind::Date => format_date(section, magnitude, epoch, locale, &mut sink)?,
            Kind::Text => {
                // A text section reached by a number shows the number as a
                // general number in place of the text.
                let general = general_text(magnitude, locale);
                emit_text_section(section, &general, &mut sink);
            }
        }
        Some(sink.finish(section.colour))
    }

    /// Put a text value through the format.
    #[must_use]
    pub fn apply_text(&self, text: &str) -> Formatted {
        let mut sink = Sink::default();
        let section = self
            .sections
            .iter()
            .find(|s| s.kind == Kind::Text)
            .or_else(|| {
                self.sections
                    .first()
                    .filter(|s| s.tokens.iter().any(|t| matches!(t, Token::Text)))
            });
        match section {
            Some(section) => {
                emit_text_section(section, text, &mut sink);
                sink.finish(section.colour)
            }
            None => {
                sink.push_str(text);
                sink.finish(None)
            }
        }
    }
}

#[derive(Default)]
struct Sink {
    text: String,
    fill: Option<Fill>,
}

impl Sink {
    fn push(&mut self, c: char) {
        self.text.push(c);
    }

    fn push_str(&mut self, s: &str) {
        self.text.push_str(s);
    }

    fn literal(&mut self, token: &Token) -> bool {
        match token {
            Token::Literal(text) => self.push_str(text),
            Token::Skip(_) => self.push(' '),
            Token::Fill(with) => {
                if self.fill.is_none() {
                    self.fill = Some(Fill {
                        at: self.text.chars().count(),
                        with: *with,
                    });
                }
            }
            _ => return false,
        }
        true
    }

    fn finish(self, colour: Option<Colour>) -> Formatted {
        Formatted {
            text: self.text,
            colour,
            fill: self.fill,
        }
    }
}

fn emit_text_section(section: &Section, text: &str, sink: &mut Sink) {
    for token in &section.tokens {
        if !sink.literal(token) && matches!(token, Token::Text) {
            sink.push_str(text);
        }
    }
}

/// A general number: up to fifteen significant digits, no trailing zeros,
/// positional from one ten-thousandth up to but not including ten to the
/// fifteen, and scientific outside that. The contraction into a narrow column is the
/// renderer's fitting stage and is not modelled here.
fn general_text(value: f64, locale: &Locale) -> String {
    let mut out = String::new();
    if value < 0.0 {
        out.push('-');
    }
    let decimal = Decimal::of(value);
    if decimal.is_zero() {
        out.push('0');
        return out;
    }
    if decimal.exponent > 15 || decimal.exponent < -3 {
        let mut mantissa = decimal.clone();
        let power = decimal.exponent.saturating_sub(1);
        mantissa.exponent = 1;
        let places = mantissa.digits.len().saturating_sub(1);
        push_fixed(&mantissa, 1, places, false, locale, &mut out);
        out.push('E');
        out.push(if power < 0 { '-' } else { '+' });
        out.push_str(&format!("{:02}", power.abs()));
        return out;
    }
    let places = usize::try_from(
        i32::try_from(decimal.digits.len())
            .unwrap_or(0)
            .saturating_sub(decimal.exponent),
    )
    .unwrap_or(0);
    push_fixed(&decimal, 1, places, false, locale, &mut out);
    out
}

/// Write a decimal positionally with at least `min_integer` integer digits and
/// exactly `places` fraction digits.
fn push_fixed(
    decimal: &Decimal,
    min_integer: usize,
    places: usize,
    group: bool,
    locale: &Locale,
    out: &mut String,
) {
    let mut integer = decimal.integer_digits();
    while integer.len() < min_integer {
        integer.insert(0, 0);
    }
    let total = integer.len();
    for (index, digit) in integer.iter().enumerate() {
        out.push(char::from(b'0'.saturating_add(*digit)));
        let remaining = total.saturating_sub(index).saturating_sub(1);
        if group && remaining > 0 && remaining % 3 == 0 {
            out.push(locale.group);
        }
    }
    if places > 0 {
        out.push(locale.decimal);
        for digit in decimal.fraction_digits(places) {
            out.push(char::from(b'0'.saturating_add(digit)));
        }
    }
}

fn format_general(section: &Section, value: f64, locale: &Locale, sink: &mut Sink) {
    for token in &section.tokens {
        if !sink.literal(token) && matches!(token, Token::General) {
            sink.push_str(&general_text(value, locale));
        }
    }
}

/// What a number section holds, read once before the value is placed.
struct Shape {
    integer: Vec<Placeholder>,
    fraction: Vec<Placeholder>,
    exponent: Vec<Placeholder>,
    numerator: Vec<Placeholder>,
    /// Denominator placeholders, or the literal digits of a fixed denominator.
    denominator: Vec<Placeholder>,
    fixed_denominator: Option<u64>,
    group: bool,
    scale: i32,
    percent: i32,
    has_point: bool,
    has_exponent: bool,
    has_fraction: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Part {
    Integer,
    Fraction,
    Exponent,
    Numerator,
    Denominator,
}

fn shape_of(section: &Section) -> Shape {
    let has_fraction = section
        .tokens
        .iter()
        .any(|t| matches!(t, Token::FractionBar));
    let bar = section
        .tokens
        .iter()
        .position(|t| matches!(t, Token::FractionBar));
    // The numerator is the run of placeholders directly before the bar.
    let numerator_start = bar.map(|bar| {
        let mut start = bar;
        while start > 0
            && section
                .tokens
                .get(start.saturating_sub(1))
                .is_some_and(|t| matches!(t, Token::Digit(_)))
        {
            start = start.saturating_sub(1);
        }
        start
    });
    let mut shape = Shape {
        integer: Vec::new(),
        fraction: Vec::new(),
        exponent: Vec::new(),
        numerator: Vec::new(),
        denominator: Vec::new(),
        fixed_denominator: None,
        group: false,
        scale: 0,
        percent: 0,
        has_point: false,
        has_exponent: false,
        has_fraction,
    };
    let mut part = Part::Integer;
    let mut fixed = String::new();
    for (index, token) in section.tokens.iter().enumerate() {
        if numerator_start.is_some_and(|start| index == start) && part == Part::Integer {
            part = Part::Numerator;
        }
        match token {
            Token::Digit(placeholder) => match part {
                Part::Integer => shape.integer.push(*placeholder),
                Part::Fraction => shape.fraction.push(*placeholder),
                Part::Exponent => shape.exponent.push(*placeholder),
                Part::Numerator => shape.numerator.push(*placeholder),
                Part::Denominator => shape.denominator.push(*placeholder),
            },
            Token::Literal(text) if part == Part::Denominator => {
                if text.chars().all(|c| c.is_ascii_digit()) {
                    fixed.push_str(text);
                } else {
                    part = Part::Integer;
                }
            }
            Token::Point => {
                shape.has_point = true;
                part = Part::Fraction;
            }
            Token::Exponent { .. } => {
                shape.has_exponent = true;
                part = Part::Exponent;
            }
            Token::FractionBar => part = Part::Denominator,
            Token::Group => shape.group = true,
            Token::Scale => shape.scale = shape.scale.saturating_add(1),
            Token::Percent => shape.percent = shape.percent.saturating_add(1),
            _ => {}
        }
    }
    if !fixed.is_empty() {
        shape.fixed_denominator = fixed.parse::<u64>().ok().filter(|d| *d > 0);
    }
    shape
}

fn format_number(section: &Section, value: f64, locale: &Locale, sink: &mut Sink) {
    let shape = shape_of(section);
    let mut decimal = Decimal::of(value);
    decimal.scale(shape.percent.saturating_mul(2));
    decimal.scale(shape.scale.saturating_mul(-3));
    let negative = value < 0.0 && !decimal.is_zero();

    if shape.has_fraction {
        format_fraction(section, &shape, decimal.to_f64(), negative, locale, sink);
        return;
    }

    let mut power: i32 = 0;
    if shape.has_exponent {
        let width = i32::try_from(shape.integer.len().max(1)).unwrap_or(1);
        if !decimal.is_zero() {
            let magnitude = decimal.exponent.saturating_sub(1);
            power = magnitude.div_euclid(width).saturating_mul(width);
            decimal.exponent = decimal.exponent.saturating_sub(power);
            decimal.round_to_places(shape.fraction.len());
            // Rounding can carry into one more integer digit than the mantissa
            // allows; then the exponent moves up one width.
            if decimal.exponent > width {
                power = power.saturating_add(width);
                decimal.exponent = decimal.exponent.saturating_sub(width);
            }
        }
    } else {
        decimal.round_to_places(shape.fraction.len());
    }
    let negative = negative && !decimal.is_zero();

    let integer = decimal.integer_digits();
    let fraction = decimal.fraction_digits(shape.fraction.len());
    let integer_text = place_integer(&integer, &shape.integer, shape.group, locale);
    let fraction_text = place_fraction(&fraction, &shape.fraction);

    let mut part = Part::Integer;
    let mut integer_written = false;
    let mut fraction_written = false;
    let mut exponent_written = false;
    let mut sign_written = false;
    for token in &section.tokens {
        if sink.literal(token) {
            continue;
        }
        match token {
            Token::Digit(_) => match part {
                Part::Integer => {
                    if !integer_written {
                        if negative && !sign_written {
                            sink.push('-');
                            sign_written = true;
                        }
                        sink.push_str(&integer_text);
                        integer_written = true;
                    }
                }
                Part::Fraction => {
                    if !fraction_written {
                        sink.push_str(&fraction_text);
                        fraction_written = true;
                    }
                }
                Part::Exponent => {
                    if !exponent_written {
                        let digits: Vec<u8> = power
                            .abs()
                            .to_string()
                            .chars()
                            .filter_map(|c| c.to_digit(10))
                            .filter_map(|d| u8::try_from(d).ok())
                            .collect();
                        sink.push_str(&place_integer(&digits, &shape.exponent, false, locale));
                        exponent_written = true;
                    }
                }
                Part::Numerator | Part::Denominator => {}
            },
            Token::Point => {
                if negative && !sign_written {
                    sink.push('-');
                    sign_written = true;
                }
                if !integer_written && shape.integer.is_empty() {
                    integer_written = true;
                }
                // The point stays even when every fraction digit was dropped,
                // which is what the incumbent shows for `#.##` and three.
                sink.push(locale.decimal);
                part = Part::Fraction;
            }
            Token::Exponent {
                always_signed,
                upper,
            } => {
                sink.push(if *upper { 'E' } else { 'e' });
                if power < 0 {
                    sink.push('-');
                } else if *always_signed {
                    sink.push('+');
                }
                part = Part::Exponent;
            }
            Token::Percent => sink.push('%'),
            Token::Group | Token::Scale | Token::Text | Token::General | Token::Date(_) => {}
            Token::FractionBar | Token::Literal(_) | Token::Fill(_) | Token::Skip(_) => {}
        }
    }
    if negative && !sign_written {
        // A section with no placeholder at all still shows the sign.
        let mut text = String::from("-");
        text.push_str(&sink.text);
        sink.text = text;
    }
}

/// Lay integer digits over their placeholders, right aligned, with the extra
/// digits at the left and the group separator between digits.
fn place_integer(
    digits: &[u8],
    placeholders: &[Placeholder],
    group: bool,
    locale: &Locale,
) -> String {
    let width = placeholders.len().max(digits.len());
    let pad = width.saturating_sub(digits.len());
    let mut cells: Vec<Option<char>> = Vec::with_capacity(width);
    for index in 0..width {
        if index < pad {
            // A padding position: its placeholder is the one at the same
            // position from the right.
            let from_right = width.saturating_sub(index).saturating_sub(1);
            let placeholder = placeholders
                .len()
                .checked_sub(from_right.saturating_add(1))
                .and_then(|at| placeholders.get(at));
            cells.push(match placeholder {
                Some(Placeholder::Zero) => Some('0'),
                Some(Placeholder::Space) => Some(' '),
                _ => None,
            });
        } else {
            let digit = digits.get(index.saturating_sub(pad)).copied().unwrap_or(0);
            cells.push(Some(char::from(b'0'.saturating_add(digit))));
        }
    }
    let mut out = String::new();
    let digit_count = cells
        .iter()
        .filter(|c| c.is_some_and(|c| c.is_ascii_digit()))
        .count();
    let mut digits_seen = 0usize;
    for cell in cells {
        let Some(c) = cell else {
            continue;
        };
        out.push(c);
        if c.is_ascii_digit() {
            digits_seen = digits_seen.saturating_add(1);
            let remaining = digit_count.saturating_sub(digits_seen);
            if group && remaining > 0 && remaining % 3 == 0 {
                out.push(locale.group);
            }
        }
    }
    out
}

/// Lay fraction digits over their placeholders, dropping the trailing zeros a
/// `#` allows to go.
fn place_fraction(digits: &[u8], placeholders: &[Placeholder]) -> String {
    let mut cells: Vec<Option<char>> = placeholders
        .iter()
        .enumerate()
        .map(|(index, _)| {
            let digit = digits.get(index).copied().unwrap_or(0);
            Some(char::from(b'0'.saturating_add(digit)))
        })
        .collect();
    for (index, placeholder) in placeholders.iter().enumerate().rev() {
        let digit = digits.get(index).copied().unwrap_or(0);
        if digit != 0 {
            break;
        }
        match placeholder {
            Placeholder::Nothing => {
                if let Some(cell) = cells.get_mut(index) {
                    *cell = None;
                }
            }
            Placeholder::Space => {
                if let Some(cell) = cells.get_mut(index) {
                    *cell = Some(' ');
                }
            }
            Placeholder::Zero => break,
        }
    }
    cells.into_iter().flatten().collect()
}

fn format_fraction(
    section: &Section,
    shape: &Shape,
    value: f64,
    negative: bool,
    locale: &Locale,
    sink: &mut Sink,
) {
    let magnitude = value.abs();
    let has_integer = !shape.integer.is_empty();
    let whole = if has_integer { magnitude.floor() } else { 0.0 };
    let remainder = magnitude - whole;
    let (numerator, denominator) = match shape.fixed_denominator {
        Some(fixed) => {
            let d = fixed as f64;
            ((remainder * d).round(), d)
        }
        None => best_fraction(remainder, shape.denominator.len()),
    };
    let (whole, numerator) = if numerator >= denominator && has_integer {
        (whole + 1.0, 0.0)
    } else {
        (whole, numerator)
    };
    let integer_digits = digits_of(whole);
    let numerator_digits = digits_of(numerator);
    let denominator_digits = digits_of(denominator);
    let show_fraction = numerator != 0.0;

    let mut part = Part::Integer;
    let mut written = [false; 3];
    if negative {
        sink.push('-');
    }
    let numerator_start = section
        .tokens
        .iter()
        .position(|t| matches!(t, Token::FractionBar))
        .map(|bar| {
            let mut start = bar;
            while start > 0
                && section
                    .tokens
                    .get(start.saturating_sub(1))
                    .is_some_and(|t| matches!(t, Token::Digit(_)))
            {
                start = start.saturating_sub(1);
            }
            start
        });
    for (index, token) in section.tokens.iter().enumerate() {
        if numerator_start.is_some_and(|start| start == index) && part == Part::Integer {
            part = Part::Numerator;
        }
        match token {
            Token::Digit(_) => match part {
                Part::Integer => {
                    if !written[0] {
                        if has_integer && (whole != 0.0 || !show_fraction) {
                            sink.push_str(&place_integer(
                                &integer_digits,
                                &shape.integer,
                                shape.group,
                                locale,
                            ));
                        } else {
                            sink.push_str(&place_integer(&[], &shape.integer, false, locale));
                        }
                        written[0] = true;
                    }
                }
                Part::Numerator => {
                    if !written[1] {
                        if show_fraction {
                            sink.push_str(&pad_left(&numerator_digits, &shape.numerator));
                        } else {
                            sink.push_str(&" ".repeat(shape.numerator.len()));
                        }
                        written[1] = true;
                    }
                }
                Part::Denominator => {
                    if !written[2] {
                        if show_fraction {
                            sink.push_str(&pad_right(&denominator_digits, &shape.denominator));
                        } else {
                            sink.push_str(&" ".repeat(shape.denominator.len()));
                        }
                        written[2] = true;
                    }
                }
                Part::Fraction | Part::Exponent => {}
            },
            Token::FractionBar => {
                sink.push(if show_fraction { '/' } else { ' ' });
                part = Part::Denominator;
            }
            Token::Literal(text)
                if part == Part::Denominator && text.chars().all(|c| c.is_ascii_digit()) =>
            {
                if !written[2] {
                    if show_fraction {
                        sink.push_str(text);
                    } else {
                        sink.push_str(&" ".repeat(text.chars().count()));
                    }
                    written[2] = true;
                }
            }
            other => {
                sink.literal(other);
            }
        }
    }
}

fn digits_of(value: f64) -> Vec<u8> {
    if value == 0.0 {
        return Vec::new();
    }
    format!("{value:.0}")
        .chars()
        .filter_map(|c| c.to_digit(10))
        .filter_map(|d| u8::try_from(d).ok())
        .collect()
}

fn pad_left(digits: &[u8], placeholders: &[Placeholder]) -> String {
    let mut out = String::new();
    let pad = placeholders.len().saturating_sub(digits.len());
    for placeholder in placeholders.iter().take(pad) {
        match placeholder {
            Placeholder::Zero => out.push('0'),
            Placeholder::Space => out.push(' '),
            Placeholder::Nothing => {}
        }
    }
    if digits.is_empty() {
        out.push('0');
    }
    for digit in digits {
        out.push(char::from(b'0'.saturating_add(*digit)));
    }
    out
}

fn pad_right(digits: &[u8], placeholders: &[Placeholder]) -> String {
    let mut out = String::new();
    for digit in digits {
        out.push(char::from(b'0'.saturating_add(*digit)));
    }
    let pad = placeholders.len().saturating_sub(digits.len());
    for placeholder in placeholders.iter().rev().take(pad) {
        match placeholder {
            Placeholder::Zero => out.push('0'),
            Placeholder::Space => out.push(' '),
            Placeholder::Nothing => {}
        }
    }
    out
}

/// The fraction with the smallest denominator up to `digits` wide that comes
/// closest to `value`, ties going to the smaller denominator.
fn best_fraction(value: f64, digits: usize) -> (f64, f64) {
    let mut limit = 1.0;
    for _ in 0..digits.max(1) {
        limit *= 10.0;
    }
    let mut best = (0.0, 1.0);
    let mut best_error = f64::INFINITY;
    let mut denominator = 1.0;
    while denominator < limit {
        let numerator = (value * denominator).round();
        let error = (value - numerator / denominator).abs();
        if error < best_error - 1e-12 {
            best_error = error;
            best = (numerator, denominator);
            if error == 0.0 {
                break;
            }
        }
        denominator += 1.0;
    }
    best
}

// Dates.

/// A calendar date and the time of day, as a serial number resolves to under
/// an epoch base.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Civil {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    /// Zero is Sunday.
    pub weekday: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
    /// Thousandths of a second.
    pub millisecond: u32,
}

/// The last serial the calendar reaches: the last day of the year 9999.
const LAST_SERIAL_1900: f64 = 2_958_465.0;

/// Resolve a serial number to a date and time.
///
/// Under `Epoch::Windows1900` the day that does not exist is reproduced: a
/// serial of sixty is the twenty-ninth of February 1900, and every serial
/// from sixty-one on is one day later than the calendar would put it, which
/// is how the incumbent counts and the reason `format.date-epoch` is a
/// feature the corpus measures. A serial of zero is the zeroth of January
/// 1900, shown as such. A negative serial or one past the calendar is `None`.
#[must_use]
pub fn serial_to_civil(serial: f64, epoch: Epoch, millisecond_precision: bool) -> Option<Civil> {
    if !serial.is_finite() || serial < 0.0 {
        return None;
    }
    let last = match epoch {
        Epoch::Windows1900 => LAST_SERIAL_1900,
        Epoch::Mac1904 => LAST_SERIAL_1900 - 1462.0,
    };
    if serial > last + 1.0 {
        return None;
    }
    // Round the time of day to the precision that is shown, so a value a
    // hair below the next second shows that second rather than the one before.
    let units_per_day = if millisecond_precision {
        86_400_000.0
    } else {
        86_400.0
    };
    let total_units = (serial * units_per_day).round();
    let days = (total_units / units_per_day).floor();
    let units_in_day = total_units - days * units_per_day;
    let seconds_in_day = if millisecond_precision {
        (units_in_day / 1000.0).floor()
    } else {
        units_in_day
    };
    let millisecond = if millisecond_precision {
        units_in_day - seconds_in_day * 1000.0
    } else {
        0.0
    };
    let hour = (seconds_in_day / 3600.0).floor();
    let minute = ((seconds_in_day - hour * 3600.0) / 60.0).floor();
    let second = seconds_in_day - hour * 3600.0 - minute * 60.0;

    let day_number = to_u32(days)?;
    let (year, month, day) = match epoch {
        Epoch::Windows1900 => match day_number {
            0 => (1900, 1, 0),
            1..=59 => {
                civil_from_unix_days(i64::from(day_number).wrapping_sub(25_569).wrapping_add(1))?
            }
            60 => (1900, 2, 29),
            _ => civil_from_unix_days(i64::from(day_number).wrapping_sub(25_569))?,
        },
        Epoch::Mac1904 => civil_from_unix_days(i64::from(day_number).wrapping_sub(24_107))?,
    };
    let weekday = match epoch {
        Epoch::Windows1900 => day_number.wrapping_add(6) % 7,
        Epoch::Mac1904 => day_number.wrapping_add(5) % 7,
    };
    Some(Civil {
        year,
        month,
        day,
        weekday,
        hour: to_u32(hour)?,
        minute: to_u32(minute)?,
        second: to_u32(second)?,
        millisecond: to_u32(millisecond)?,
    })
}

fn to_u32(value: f64) -> Option<u32> {
    if value < 0.0 || value > f64::from(u32::MAX) {
        return None;
    }
    // Truncation is the intent: the value is already integral.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "checked against the range of u32 on the two lines above, and the value is integral"
    )]
    Some(value as u32)
}

/// The proleptic Gregorian date for a count of days since 1970-01-01.
///
/// The arithmetic is the civil-from-days algorithm over integers, and every
/// operand is bounded by the caller: a serial is refused above the year 9999
/// before this is reached, so no operation here can leave the range of i64.
#[allow(
    clippy::arithmetic_side_effects,
    clippy::integer_division,
    reason = "every operand is bounded by the serial ceiling checked in serial_to_civil, the year is below ten thousand, and the algorithm is integer division by construction"
)]
fn civil_from_unix_days(days: i64) -> Option<(i32, u32, u32)> {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };
    Some((
        i32::try_from(year).ok()?,
        u32::try_from(m).ok()?,
        u32::try_from(d).ok()?,
    ))
}

fn format_date(
    section: &Section,
    value: f64,
    epoch: Epoch,
    locale: &Locale,
    sink: &mut Sink,
) -> Option<()> {
    let twelve_hour = section
        .tokens
        .iter()
        .any(|t| matches!(t, Token::Date(DateCode::AmPm { .. })));
    let fractional = section.tokens.iter().find_map(|t| match t {
        Token::Date(DateCode::FractionalSeconds { digits }) => Some(*digits),
        _ => None,
    });
    let shows_seconds = section.tokens.iter().any(|t| {
        matches!(
            t,
            Token::Date(DateCode::Second { .. }) | Token::Date(DateCode::ElapsedSeconds)
        )
    });
    // The time is rounded to what is shown: to the fractional digits, else to
    // the second, else to the minute.
    let rounding_units = if let Some(digits) = fractional {
        let mut per_second = 1.0;
        for _ in 0..digits {
            per_second *= 10.0;
        }
        86_400.0 * per_second
    } else if shows_seconds {
        86_400.0
    } else {
        1440.0
    };
    let rounded = (value * rounding_units).round() / rounding_units;
    let civil = serial_to_civil(rounded, epoch, fractional.is_some())?;

    let elapsed_seconds = (rounded * 86_400.0).round();
    for token in &section.tokens {
        if sink.literal(token) {
            continue;
        }
        let Token::Date(code) = token else {
            continue;
        };
        match code {
            DateCode::Year { four } => {
                if *four {
                    sink.push_str(&format!("{:04}", civil.year));
                } else {
                    sink.push_str(&format!("{:02}", civil.year.rem_euclid(100)));
                }
            }
            DateCode::Month { letters } => match letters {
                1 => sink.push_str(&civil.month.to_string()),
                2 => sink.push_str(&format!("{:02}", civil.month)),
                3 => sink.push_str(month_name(&locale.months_short, civil.month)),
                4 => sink.push_str(month_name(&locale.months_long, civil.month)),
                _ => {
                    if let Some(initial) =
                        month_name(&locale.months_long, civil.month).chars().next()
                    {
                        sink.push(initial);
                    }
                }
            },
            DateCode::Day { letters } => match letters {
                1 => sink.push_str(&civil.day.to_string()),
                2 => sink.push_str(&format!("{:02}", civil.day)),
                3 => sink.push_str(
                    locale
                        .days_short
                        .get(usize::try_from(civil.weekday).unwrap_or(0))
                        .copied()
                        .unwrap_or(""),
                ),
                _ => sink.push_str(
                    locale
                        .days_long
                        .get(usize::try_from(civil.weekday).unwrap_or(0))
                        .copied()
                        .unwrap_or(""),
                ),
            },
            DateCode::Hour { padded } => {
                let hour = if twelve_hour {
                    match civil.hour % 12 {
                        0 => 12,
                        h => h,
                    }
                } else {
                    civil.hour
                };
                if *padded {
                    sink.push_str(&format!("{hour:02}"));
                } else {
                    sink.push_str(&hour.to_string());
                }
            }
            DateCode::Minute { padded } => {
                if *padded {
                    sink.push_str(&format!("{:02}", civil.minute));
                } else {
                    sink.push_str(&civil.minute.to_string());
                }
            }
            DateCode::Second { padded } => {
                if *padded {
                    sink.push_str(&format!("{:02}", civil.second));
                } else {
                    sink.push_str(&civil.second.to_string());
                }
            }
            DateCode::FractionalSeconds { digits } => {
                sink.push(locale.decimal);
                let text = format!("{:03}", civil.millisecond);
                sink.push_str(text.get(..usize::from(*digits)).unwrap_or(&text));
            }
            DateCode::AmPm { long, upper } => {
                let is_pm = civil.hour >= 12;
                let word = if *long {
                    if is_pm { locale.pm } else { locale.am }
                } else if is_pm {
                    "P"
                } else {
                    "A"
                };
                if *upper {
                    sink.push_str(&word.to_uppercase());
                } else {
                    sink.push_str(&word.to_lowercase());
                }
            }
            DateCode::ElapsedHours => {
                sink.push_str(&format!("{:.0}", (elapsed_seconds / 3600.0).floor()));
            }
            DateCode::ElapsedMinutes => {
                sink.push_str(&format!("{:.0}", (elapsed_seconds / 60.0).floor()));
            }
            DateCode::ElapsedSeconds => {
                sink.push_str(&format!("{elapsed_seconds:.0}"));
            }
        }
    }
    Some(())
}

fn month_name<'a>(names: &'a [&'static str; 12], month: u32) -> &'a str {
    usize::try_from(month)
        .ok()
        .and_then(|m| m.checked_sub(1))
        .and_then(|index| names.get(index))
        .copied()
        .unwrap_or("")
}
