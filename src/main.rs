use std::env;
use std::process::ExitCode;

mod barcode;
mod isbn;

fn print_usage() {
    eprintln!("usage: isbnconv <isbn> [--lenient]");
    eprintln!("       isbnconv --check <isbn|upc-a|ean-8> [--lenient]");
    eprintln!();
    eprintln!("converts between ISBN-10 and ISBN-13 (the ISBN-13 digits are the same");
    eprintln!("digits printed under the barcode on the back cover). the source format");
    eprintln!("is detected from the digit count of the input, hyphens and spaces are");
    eprintln!("ignored.");
    eprintln!();
    eprintln!("--check also accepts 12-digit UPC-A and 8-digit EAN-8 barcodes, which");
    eprintln!("have no ISBN equivalent so they can only be checked, not converted.");
    eprintln!();
    eprintln!("by default a wrong checksum digit is a hard error. --lenient replaces");
    eprintln!("it with the correct one instead of refusing to run.");
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        print_usage();
        return ExitCode::FAILURE;
    }

    let mut lenient = false;
    let mut check_only = false;
    let mut isbn_arg: Option<String> = None;

    for arg in args {
        match arg.as_str() {
            "--lenient" => lenient = true,
            "--check" => check_only = true,
            "-h" | "--help" => {
                print_usage();
                return ExitCode::SUCCESS;
            }
            other => {
                if isbn_arg.is_some() {
                    eprintln!("error: unexpected extra argument '{other}'");
                    return ExitCode::FAILURE;
                }
                isbn_arg = Some(other.to_string());
            }
        }
    }

    let Some(raw) = isbn_arg else {
        print_usage();
        return ExitCode::FAILURE;
    };

    let normalized = isbn::normalize(&raw);

    if check_only {
        let result: Result<(), String> = match normalized.len() {
            8 => barcode::parse_ean_8(&normalized, lenient)
                .map(|_| ())
                .map_err(|e| e.to_string()),
            10 => isbn::parse_isbn10(&normalized, lenient)
                .map(|_| ())
                .map_err(|e| e.to_string()),
            12 => barcode::parse_upc_a(&normalized, lenient)
                .map(|_| ())
                .map_err(|e| e.to_string()),
            13 => isbn::parse_isbn13(&normalized, lenient)
                .map(|_| ())
                .map_err(|e| e.to_string()),
            n => Err(isbn::IsbnError::BadLength(n).to_string()),
        };
        match result {
            Ok(()) => {
                println!("valid");
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("invalid: {e}");
                ExitCode::FAILURE
            }
        }
    } else {
        let result: Result<String, String> = match normalized.len() {
            8 => Err(
                "8-digit input looks like an EAN-8 barcode, which has no ISBN form; \
                 use --check instead of conversion"
                    .to_string(),
            ),
            10 => isbn::isbn10_to_isbn13(&raw, lenient).map_err(|e| e.to_string()),
            12 => Err(
                "12-digit input looks like a UPC-A barcode, which has no ISBN form; \
                 use --check instead of conversion"
                    .to_string(),
            ),
            13 => isbn::isbn13_to_isbn10(&raw, lenient).map_err(|e| e.to_string()),
            n => Err(isbn::IsbnError::BadLength(n).to_string()),
        };
        match result {
            Ok(converted) => {
                println!("{converted}");
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("error: {e}");
                ExitCode::FAILURE
            }
        }
    }
}
