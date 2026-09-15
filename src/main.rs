use std::env;
use std::process::ExitCode;

mod isbn;

fn print_usage() {
    eprintln!("usage: isbnconv <isbn> [--lenient]");
    eprintln!("       isbnconv --check <isbn> [--lenient]");
    eprintln!();
    eprintln!("converts between ISBN-10 and ISBN-13 (the ISBN-13 digits are the same");
    eprintln!("digits printed under the barcode on the back cover). the source format");
    eprintln!("is detected from the digit count of the input, hyphens and spaces are");
    eprintln!("ignored.");
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
        let result = match normalized.len() {
            10 => isbn::parse_isbn10(&normalized, lenient).map(|_| ()),
            13 => isbn::parse_isbn13(&normalized, lenient).map(|_| ()),
            n => Err(isbn::IsbnError::BadLength(n)),
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
        let result = match normalized.len() {
            10 => isbn::isbn10_to_isbn13(&raw, lenient),
            13 => isbn::isbn13_to_isbn10(&raw, lenient),
            n => Err(isbn::IsbnError::BadLength(n)),
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
