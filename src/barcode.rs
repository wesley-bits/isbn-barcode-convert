//! UPC-A and EAN-8 checksum validation and repair.
//!
//! Both use the same weighted-sum-mod-10 checksum as an ISBN-13/EAN-13
//! barcode, just with a different digit count and the weights swapped
//! (3 first instead of 1 first). That swap isn't arbitrary: prepending a
//! single "0" to a UPC-A number turns it into its EAN-13 equivalent, and
//! that shifts every data digit one position over, flipping which ones
//! land on the odd/even weight.

use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum BarcodeError {
    BadLength { expected: usize, found: usize },
    BadCharacter(char),
    ChecksumMismatch { expected: u8, found: u8 },
}

impl fmt::Display for BarcodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BarcodeError::BadLength { expected, found } => {
                write!(f, "expected {expected} digits, got {found}")
            }
            BarcodeError::BadCharacter(c) => {
                write!(f, "unexpected character '{c}' (only digits are allowed)")
            }
            BarcodeError::ChecksumMismatch { expected, found } => write!(
                f,
                "checksum digit is wrong: expected {expected}, found {found} (use --lenient to repair it)"
            ),
        }
    }
}

impl std::error::Error for BarcodeError {}

fn digit_value(c: char) -> Result<u8, BarcodeError> {
    match c {
        '0'..='9' => Ok(c as u8 - b'0'),
        other => Err(BarcodeError::BadCharacter(other)),
    }
}

/// Weighted-sum-mod-10 check digit shared by UPC-A and EAN-8. `data` is
/// every digit except the check digit itself; the weight alternates
/// 3, 1, 3, 1, ... starting from the leftmost data digit.
fn weighted_check_digit(data: &[u8]) -> u8 {
    let sum: u32 = data
        .iter()
        .enumerate()
        .map(|(i, &d)| if i % 2 == 0 { 3 * d as u32 } else { d as u32 })
        .sum();
    ((10 - (sum % 10)) % 10) as u8
}

fn parse_and_repair(
    normalized: &str,
    expected_len: usize,
    lenient: bool,
) -> Result<Vec<u8>, BarcodeError> {
    let chars: Vec<char> = normalized.chars().collect();
    if chars.len() != expected_len {
        return Err(BarcodeError::BadLength {
            expected: expected_len,
            found: chars.len(),
        });
    }
    let mut digits = Vec::with_capacity(expected_len);
    for &c in &chars {
        digits.push(digit_value(c)?);
    }
    let expected = weighted_check_digit(&digits[..expected_len - 1]);
    let last = digits[expected_len - 1];
    if last != expected {
        if lenient {
            digits[expected_len - 1] = expected;
        } else {
            return Err(BarcodeError::ChecksumMismatch {
                expected,
                found: last,
            });
        }
    }
    Ok(digits)
}

/// Validates (and, if `lenient`, repairs) a 12-digit UPC-A. Returns the
/// digits with a correct check digit in the last position.
pub fn parse_upc_a(normalized: &str, lenient: bool) -> Result<[u8; 12], BarcodeError> {
    let digits = parse_and_repair(normalized, 12, lenient)?;
    Ok(digits.try_into().unwrap())
}

/// Validates (and, if `lenient`, repairs) an 8-digit EAN-8.
pub fn parse_ean_8(normalized: &str, lenient: bool) -> Result<[u8; 8], BarcodeError> {
    let digits = parse_and_repair(normalized, 8, lenient)?;
    Ok(digits.try_into().unwrap())
}

pub fn digits_to_string(digits: &[u8]) -> String {
    digits.iter().map(|&d| (b'0' + d) as char).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upc_a_valid_checksum_is_accepted() {
        assert!(parse_upc_a("036000291452", false).is_ok());
    }

    #[test]
    fn upc_a_strict_mode_rejects_bad_checksum() {
        assert_eq!(
            parse_upc_a("036000291451", false),
            Err(BarcodeError::ChecksumMismatch {
                expected: 2,
                found: 1
            })
        );
    }

    #[test]
    fn upc_a_lenient_mode_repairs_bad_checksum() {
        let digits = parse_upc_a("036000291451", true).unwrap();
        assert_eq!(digits_to_string(&digits), "036000291452");
    }

    #[test]
    fn ean_8_valid_checksum_is_accepted() {
        assert!(parse_ean_8("40170725", false).is_ok());
    }

    #[test]
    fn ean_8_strict_mode_rejects_bad_checksum() {
        assert_eq!(
            parse_ean_8("40170724", false),
            Err(BarcodeError::ChecksumMismatch {
                expected: 5,
                found: 4
            })
        );
    }

    #[test]
    fn ean_8_lenient_mode_repairs_bad_checksum() {
        let digits = parse_ean_8("40170724", true).unwrap();
        assert_eq!(digits_to_string(&digits), "40170725");
    }

    #[test]
    fn bad_length_is_reported() {
        assert_eq!(
            parse_upc_a("12345", false),
            Err(BarcodeError::BadLength {
                expected: 12,
                found: 5
            })
        );
    }

    #[test]
    fn non_digit_character_is_reported() {
        assert_eq!(
            parse_ean_8("4017072X", false),
            Err(BarcodeError::BadCharacter('X'))
        );
    }
}
