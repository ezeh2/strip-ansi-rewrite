# Adversarial `strip-ansi` Inputs

This document collects malformed, boundary, unusually large, and otherwise hostile inputs for testing both the npm and Rust implementations. The cases are intended to expose panics, hangs, output mismatches, and excessive resource use; they are not expected to crash either implementation.

## Notation

The examples use these names for control characters:

```text
ESC = U+001B
BEL = U+0007
ST  = ESC followed by `\\`
CSI = ESC followed by `[`
OSC = ESC followed by `]`
```

Inputs should be generated from these descriptions in a temporary file or passed directly to a test process. Do not paste raw control sequences into a terminal unless the output is safely redirected.

## Boundary and malformed cases

| Case | Encoded input | Expected observation |
| --- | --- | --- |
| Empty | `""` | Returns an empty string. |
| Plain text | `"ordinary text"` | Returns the input unchanged. |
| Escape only | `ESC` | Returns normally; no hang or panic. |
| CSI introducer only | `ESC[` | Returns normally. |
| Partial CSI | `ESC[31` | Returns normally and matches the npm oracle. |
| OSC without terminator | `ESC]title` | Returns normally; does not consume unrelated later text. |
| Control string without ST | `ESC Ppayload` | Returns normally. |
| Terminator only | `ST` | Returns normally. |
| BEL only | `BEL` | Returns normally. |
| Repeated ESC | `ESC ESC ESC ESC` | Returns normally without quadratic behavior. |
| NUL in text | `before NUL after` | Preserves or removes it according to oracle behavior. |

## ANSI sequence variations

Test each sequence with empty, short, long, and unusual payloads:

```text
ESC[31mredESC[0m
ESC[?25lhidden cursorESC[?25h
ESC[1;38;2;255;0;128mcoloredESC[0m
ESC]0;window titleBEL
ESC]8;;https://example.comBELlinkESC]8;;BEL
ESC]8;;https://example.comSTlink
ESC Pprivate payloadST
ESC ^private payloadST
ESC _private payloadST
ESC kprivate payloadST
```

Also test:

- Empty CSI parameters
- Very long parameter lists
- Digits, semicolons, question marks, and greater-than signs
- Intermediate bytes before the final byte
- C1 CSI (`U+009B`)
- C1 OSC and control-string introducers
- UTF-8 encoded C1 controls
- Linux palette forms such as `ESC]P1234567` and `ESC]R`

## Unicode and text boundaries

Include:

```text
世界 🌍 café
combining: e + U+0301
zero-width: A + U+200B + B
line one\nline two\rline three
tabs\tand spaces
```

Place ANSI sequences before, between, and after multi-byte UTF-8 characters. Verify that stripping never leaves invalid UTF-8 or removes neighboring text accidentally.

## Size and resource cases

Generate, rather than commit, large inputs such as:

- One million ordinary characters
- One million ANSI introducers
- Long unterminated OSC payloads
- Repeated valid sequences alternating with text
- A large number of parameters in one CSI sequence

Run these cases with an explicit timeout and record peak memory where practical. Do not use unbounded input sizes in ordinary CI runs.

## Execution matrix

For each case, record:

| Field | Value |
| --- | --- |
| Case name |  |
| Input description |  |
| npm output |  |
| Rust output |  |
| Outputs identical? |  |
| npm exit status |  |
| Rust exit status |  |
| Panic or crash? |  |
| Timeout? |  |
| Resource observation |  |

The npm implementation is the compatibility oracle, but explicit expected outputs should be added for important regressions so Rust tests can run without Node.js.

## Safe test procedure

1. Generate inputs in a temporary directory.
2. Pass them as data to the test process; never execute them as shell syntax.
3. Capture stdout and stderr separately.
4. Apply a timeout to every invocation.
5. Compare exact output and exit status.
6. Save only small, reproducible failing cases to the repository.
7. Turn every discovered mismatch or crash into a permanent regression test.

## Completion checklist

- [ ] npm returns normally for every committed case.
- [ ] Rust returns normally for every committed case.
- [ ] Outputs match, or the difference is documented.
- [ ] Neither implementation hangs under the configured timeout.
- [ ] No panic or abnormal termination is observed.
- [ ] Large generated cases are run separately from the normal test suite.
