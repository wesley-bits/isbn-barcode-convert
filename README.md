# isbn-barcode-convert

Every book has two ISBNs: the old 10-digit one and the current 13-digit
one. The 13-digit form isn't a separate identifier bolted onto the book —
it's exactly the number encoded in the EAN-13 barcode printed on the back
cover, with a `978` or `979` "Bookland" prefix in front of the old
9-digit publisher/title code. Convert one and you're also computing (or
checking) the barcode's check digit.

This is a small command-line tool for converting between the two forms
and for checking whether a given ISBN's check digit is actually correct.
No network access, no lookups against a database of real books — it's
pure checksum arithmetic.

## Usage

```
isbnconv <isbn> [--lenient]
isbnconv --check <isbn> [--lenient]
```

The source format (ISBN-10 vs ISBN-13) is detected from how many digits
the input has after stripping hyphens and spaces.

```
$ isbnconv 0-262-01153-0
9780262011530

$ isbnconv 978-0-262-01153-0
0262011530

$ isbnconv --check 0-262-01153-0
valid
```

### Strict by default

A check digit is a promise that a number was copied correctly. By
default this tool holds you to that promise: if the check digit doesn't
match what the rest of the number implies, conversion refuses to guess
and exits with an error.

```
$ isbnconv 0-262-01153-1
error: checksum digit is wrong: expected 0, found 1 (use --lenient to repair it)
```

Pass `--lenient` when you want the tool to fix the check digit instead
of rejecting the input — useful when you already trust the source (a
scanner, a spreadsheet someone typed the first nine digits of by hand)
and just want the correct trailing digit filled in:

```
$ isbnconv --lenient 0-262-01153-1
9780262011530
```

Note that `--lenient` only affects the check digit. It does not accept
the wrong number of digits, letters other than a trailing `X`, or a
978-prefixed-only conversion path in reverse (a 979-prefixed ISBN-13 has
no ISBN-10 form at all — that prefix was introduced after ISBN-10 was
retired, and no amount of leniency invents one).

## Building

Standard library only, no external crates:

```
cargo build --release
```

## Status

Early skeleton. ISBN-10 and ISBN-13 conversion and validation work.
Not yet covered: other barcode families (UPC-A, EAN-8), batch/file
input, and a proper test suite beyond the handful of unit tests in
`src/isbn.rs`.

## License

MIT, see [LICENSE](LICENSE).
