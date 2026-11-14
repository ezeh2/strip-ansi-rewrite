# Plan: Rewrite `strip-ansi` in Rust

## Goal

Create a Rust implementation of `strip-ansi` that matches the behavior of the installed npm package (`strip-ansi@7.2.0`) for the documented and tested input cases.

The initial deliverable should be a Rust library function and a small command-line tool.

## 1. Define the compatibility target

Document the expected behavior:

- Input is a UTF-8 string.
- ANSI escape sequences are removed.
- Normal text, Unicode, and newlines are preserved.
- CSI, OSC, and other sequences supported by the npm package are handled.
- Malformed or incomplete sequences follow the npm package's behavior.

The npm package will be treated as the reference implementation, or **oracle**, during development.

## 2. Build an oracle test harness

For each test input:

1. Run the original npm implementation.
2. Run the Rust implementation.
3. Compare their exact outputs.
4. Report any mismatch.

Test cases should include:

- Basic color, bold, underline, and reset codes
- Cursor movement sequences
- 256-color and RGB sequences
- OSC sequences and hyperlinks
- Multiple adjacent escape sequences
- Incomplete and malformed sequences
- Unicode, emoji, and combining characters
- Empty strings and newlines
- Very long strings

Keep a separate set of explicit expected-output tests so the Rust project can eventually be tested without Node.js.

## 3. Implement the Rust library

Create a core function with an API similar to:

```rust
pub fn strip_ansi(input: &str) -> String
```

Use a byte-oriented scanner with explicit parser states:

- Normal text
- Escape sequence
- CSI sequence
- OSC sequence
- String terminator handling

Avoid relying solely on a broad regular expression. ANSI sequences have multiple forms and terminators, so an explicit scanner should make behavior easier to inspect and test.

## 4. Add a command-line interface

Support both direct arguments and standard input:

```bash
strip-ansi "colored text"
printf '\033[31mError\033[0m\n' | strip-ansi
```

The CLI should:

- Read one or more arguments or stdin
- Write cleaned text to stdout
- Write errors to stderr
- Preserve input newlines
- Avoid adding unsolicited formatting
- Return conventional exit statuses

## 5. Differential testing

Generate a broad corpus and compare Rust output with npm output.

Include:

- Hand-written regression cases
- Random combinations of ANSI sequences
- Unicode and invalid-boundary cases
- Literal backslashes and ordinary text
- Incomplete escape sequences
- Large log-like inputs

Every discovered mismatch should become a permanent regression test, with the chosen behavior documented.

## 6. Fuzz and malformed-input testing

Exercise the parser with arbitrary byte patterns and unusual escape sequences.

Verify that it:

- Never panics
- Does not loop indefinitely
- Preserves valid UTF-8 text correctly
- Has bounded parser state and reasonable memory use

If the public API remains UTF-8-only, document how invalid byte input is handled by the CLI.

## 7. Benchmark and inspect behavior

Measure the Rust implementation on:

- Short colored strings
- Large terminal logs
- Strings with no ANSI sequences
- Strings containing many escape sequences

Record runtime, allocations where practical, memory behavior, and binary size. Do not claim a performance improvement until both implementations are measured on equivalent workloads.

## 8. Package and document the result

Prepare:

- A Rust library crate
- A standalone Rust CLI
- Usage examples
- A compatibility table
- Test and benchmark results
- Known differences and limitations

An optional later milestone is a Node.js binding using N-API, if drop-in use from JavaScript is required.

## Suggested milestones

1. Implement basic CSI stripping.
2. Add OSC and hyperlink handling.
3. Add the CLI.
4. Add npm-oracle differential tests.
5. Add fuzzing and malformed-input tests.
6. Add benchmarks and documentation.
7. Consider optional Node.js bindings.

## Recommended first demo

Build a Rust CLI that reads colored text from stdin, strips ANSI sequences, and produces the same output as npm `strip-ansi`. Present the comparison corpus, exact-match results, and any intentionally unsupported behavior.
