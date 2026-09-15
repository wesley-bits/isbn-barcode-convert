//! ISBN-10 / ISBN-13 checksum validation and conversion.
//!
//! ISBN-13 is a plain EAN-13 barcode number (prefix 978 or 979), so the
//! check digit math here is the same math printed under every barcode.

use std::fmt;

#[derive(Debug)]
pub enum IsbnError {
    BadLength(usize),
    BadCharacter(char),
    ChecksumMismatch { expected: u8, found: String },
    NotConvertible13to10,
}

impl fmt::Display for IsbnError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IsbnError::BadLength(n) => write!(f, "expected 10 or 13 digits, got {n}"),
            IsbnError::BadCharacter(c) => write!(
                f,
                "unexpected character '{c}' (only digits and a trailing X are allowed)"
            ),
            IsbnError::ChecksumMismatch { expected, found } => write!(
                f,
                "checksum digit is wrong: expected {}, found {found} (use --lenient to repair it)",
                check_digit_to_char(*expected)
            ),
            IsbnError::NotConvertible13to10 => {
                write!(f, "only ISBN-13s starting with 978 have an ISBN-10 equivalent")
            }
        }
    }
}

impl std::error::Error for IsbnError {}

fn check_digit_to_char(d: u8) -> char {
    if d == 10 {
        'X'
    } else {
        (b'0' + d) as char
    }
}

/// Strips hyphens and whitespace, which is all the "formatting" ISBNs
/// legitimately carry. Anything else is left in place so it can be
/// reported as a bad character rather than silently dropped.
pub fn normalize(raw: &str) -> String {
    raw.chars()
        .filter(|c| !c.is_whitespace() && *c != '-')
        .map(|c| c.to_ascii_uppercase())
        .collect()
}

fn digit_value(c: char, allow_x: bool) -> Result<u8, IsbnError> {
    match c {
        '0'..='9' => Ok(c as u8 - b'0'),
        'X' if allow_x => Ok(10),
        other => Err(IsbnError::BadCharacter(other)),
    }
}

fn isbn10_check_digit(first_nine: &[u8]) -> u8 {
    let sum: u32 = first_nine
        .iter()
        .enumerate()
        .map(|(i, &d)| (10 - i as u32) * d as u32)
        .sum();
    ((11 - (sum % 11)) % 11) as u8
}

fn isbn13_check_digit(first_twelve: &[u8]) -> u8 {
    let sum: u32 = first_twelve
        .iter()
        .enumerate()
        .map(|(i, &d)| if i % 2 == 0 { d as u32 } else { 3 * d as u32 })
        .sum();
    ((10 - (sum % 10)) % 10) as u8
}

/// Parses a normalized 10-character string into digits, validating the
/// checksum unless `lenient` is set, in which case a wrong check digit
/// is silently replaced with the correct one instead of being rejected.
pub fn parse_isbn10(normalized: &str, lenient: bool) -> Result<[u8; 10], IsbnError> {
    let chars: Vec<char> = normalized.chars().collect();
    if chars.len() != 10 {
        return Err(IsbnError::BadLength(chars.len()));
    }
    let mut digits = [0u8; 10];
    for (i, &c) in chars.iter().enumerate() {
        digits[i] = digit_value(c, i == 9)?;
    }
    let expected = isbn10_check_digit(&digits[0..9]);
    if digits[9] != expected {
        if lenient {
            digits[9] = expected;
        } else {
            return Err(IsbnError::ChecksumMismatch {
                expected,
                found: normalized[9..].to_string(),
            });
        }
    }
    Ok(digits)
}

pub fn parse_isbn13(normalized: &str, lenient: bool) -> Result<[u8; 13], IsbnError> {
    let chars: Vec<char> = normalized.chars().collect();
    if chars.len() != 13 {
        return Err(IsbnError::BadLength(chars.len()));
    }
    let mut digits = [0u8; 13];
    for (i, &c) in chars.iter().enumerate() {
        digits[i] = digit_value(c, false)?;
    }
    let expected = isbn13_check_digit(&digits[0..12]);
    if digits[12] != expected {
        if lenient {
            digits[12] = expected;
        } else {
            return Err(IsbnError::ChecksumMismatch {
                expected,
                found: normalized[12..].to_string(),
            });
        }
    }
    Ok(digits)
}

pub fn isbn10_to_isbn13(input: &str, lenient: bool) -> Result<String, IsbnError> {
    let normalized = normalize(input);
    let digits = parse_isbn10(&normalized, lenient)?;

    let mut first_twelve = [0u8; 12];
    first_twelve[0] = 9;
    first_twelve[1] = 7;
    first_twelve[2] = 8;
    first_twelve[3..12].copy_from_slice(&digits[0..9]);
    let check = isbn13_check_digit(&first_twelve);

    let mut out = String::with_capacity(13);
    for d in first_twelve.iter() {
        out.push((b'0' + d) as char);
    }
    out.push((b'0' + check) as char);
    Ok(out)
}

pub fn isbn13_to_isbn10(input: &str, lenient: bool) -> Result<String, IsbnError> {
    let normalized = normalize(input);
    let digits = parse_isbn13(&normalized, lenient)?;

    if digits[0..3] != [9, 7, 8] {
        return Err(IsbnError::NotConvertible13to10);
    }
    let first_nine = &digits[3..12];
    let check = isbn10_check_digit(first_nine);

    let mut out = String::with_capacity(10);
    for d in first_nine.iter() {
        out.push((b'0' + d) as char);
    }
    out.push(check_digit_to_char(check));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_pair_converts_both_ways() {
        // Structure and Interpretation of Computer Programs, 2nd ed.
        assert_eq!(isbn10_to_isbn13("0-262-01153-0", false).unwrap(), "9780262011530");
        assert_eq!(isbn13_to_isbn10("978-0-262-01153-0", false).unwrap(), "0262011530");
    }

    #[test]
    fn strict_mode_rejects_bad_checksum() {
        assert!(isbn10_to_isbn13("0-262-01153-1", false).is_err());
    }

    #[test]
    fn lenient_mode_repairs_bad_checksum() {
        assert_eq!(isbn10_to_isbn13("0-262-01153-1", true).unwrap(), "9780262011530");
    }

    #[test]
    fn non_978_thirteen_digit_has_no_ten_digit_form() {
        // 979 is the newer Bookland prefix; it predates ISBN-10 entirely.
        let isbn13 = isbn10_to_isbn13("0-262-01153-0", false).unwrap();
        assert!(isbn13.starts_with("978"));
        let fake_979 = format!("979{}", &isbn13[3..]);
        // lenient: true just to skip past the checksum, which changes
        // when the prefix does; the case under test is the prefix check.
        assert!(matches!(
            isbn13_to_isbn10(&fake_979, true),
            Err(IsbnError::NotConvertible13to10)
        ));
    }
}
