use std::env;
use std::process::ExitCode;

mod barcode;
mod isbn;

fn print_usage() {
    eprintln!("usage: isbnconv <isbn> [--lenient]");
    eprintln!("       isbnconv --check <isbn|upc-a|ean-8> [--lenient]");
    eprintln!("       isbnconv --file <path> [--check] [--lenient]");
    eprintln!();
    eprintln!("converts between ISBN-10 and ISBN-13 (the ISBN-13 digits are the same");
    eprintln!("digits printed under the barcode on the back cover). the source format");
    eprintln!("is detected from the digit count of the input, hyphens and spaces are");
    eprintln!("ignored.");
    eprintln!();
    eprintln!("--check also accepts 12-digit UPC-A and 8-digit EAN-8 barcodes, which");
    eprintln!("have no ISBN equivalent so they can only be checked, not converted.");
    eprintln!();
    eprintln!("--file reads one isbn (or barcode, with --check) per line from <path>");
    eprintln!("and processes each line independently; blank lines are skipped. a bad");
    eprintln!("line is reported on stderr and does not stop the rest, but the tool");
    eprintln!("exits nonzero if any line failed.");
    eprintln!();
    eprintln!("by default a wrong checksum digit is a hard error. --lenient replaces");
    eprintln!("it with the correct one instead of refusing to run.");
}

/// Converts or checks a single already-trimmed input, returning the text to
/// print on success (the converted number, or "valid") or an error message.
/// Shared by the single-argument path and `--file` batch mode so the two
/// can't drift apart on how a digit count maps to a format.
fn process_one(raw: &str, check_only: bool, lenient: bool) -> Result<String, String> {
    let normalized = isbn::normalize(raw);
    if check_only {
        match normalized.len() {
            8 => barcode::parse_ean_8(&normalized, lenient)
                .map(|_| "valid".to_string())
                .map_err(|e| e.to_string()),
            10 => isbn::parse_isbn10(&normalized, lenient)
                .map(|_| "valid".to_string())
                .map_err(|e| e.to_string()),
            12 => barcode::parse_upc_a(&normalized, lenient)
                .map(|_| "valid".to_string())
                .map_err(|e| e.to_string()),
            13 => isbn::parse_isbn13(&normalized, lenient)
                .map(|_| "valid".to_string())
                .map_err(|e| e.to_string()),
            n => Err(isbn::IsbnError::BadLength(n).to_string()),
        }
    } else {
        match normalized.len() {
            8 => Err(
                "8-digit input looks like an EAN-8 barcode, which has no ISBN form; \
                 use --check instead of conversion"
                    .to_string(),
            ),
            10 => isbn::isbn10_to_isbn13(raw, lenient).map_err(|e| e.to_string()),
            12 => Err(
                "12-digit input looks like a UPC-A barcode, which has no ISBN form; \
                 use --check instead of conversion"
                    .to_string(),
            ),
            13 => isbn::isbn13_to_isbn10(raw, lenient).map_err(|e| e.to_string()),
            n => Err(isbn::IsbnError::BadLength(n).to_string()),
        }
    }
}

/// Runs `process_one` over every non-blank line of `path`, printing one
/// result per line and continuing past failures. Exits nonzero if any
/// line failed, so the overall run can still be used in a script even
/// though individual bad lines don't abort it.
fn run_batch(path: &str, check_only: bool, lenient: bool) -> ExitCode {
    let contents = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: could not read '{path}': {e}");
            return ExitCode::FAILURE;
        }
    };

    let mut any_failed = false;
    for (line_no, line) in contents.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        match process_one(trimmed, check_only, lenient) {
            Ok(s) => println!("{trimmed}: {s}"),
            Err(e) => {
                any_failed = true;
                let label = if check_only { "invalid" } else { "error" };
                eprintln!("line {}: {trimmed}: {label}: {e}", line_no + 1);
            }
        }
    }

    if any_failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
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
    let mut file_arg: Option<String> = None;

    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--lenient" => lenient = true,
            "--check" => check_only = true,
            "-h" | "--help" => {
                print_usage();
                return ExitCode::SUCCESS;
            }
            "--file" => {
                let Some(path) = iter.next() else {
                    eprintln!("error: --file requires a path argument");
                    return ExitCode::FAILURE;
                };
                file_arg = Some(path);
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

    if isbn_arg.is_some() && file_arg.is_some() {
        eprintln!("error: pass either an isbn argument or --file, not both");
        return ExitCode::FAILURE;
    }

    if let Some(path) = file_arg {
        return run_batch(&path, check_only, lenient);
    }

    let Some(raw) = isbn_arg else {
        print_usage();
        return ExitCode::FAILURE;
    };

    match process_one(&raw, check_only, lenient) {
        Ok(s) => {
            println!("{s}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            if check_only {
                eprintln!("invalid: {e}");
            } else {
                eprintln!("error: {e}");
            }
            ExitCode::FAILURE
        }
    }
}
