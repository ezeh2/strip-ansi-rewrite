# Rust `strip-ansi` rewrite

This project is a dependency-free Rust implementation of the installed npm package `strip-ansi@7.2.0`.

## Run the Rust CLI

```sh
cargo run --offline --bin strip-ansi -- $'\033[31mError\033[0m\n'
printf '\033[32mOK\033[0m\n' | cargo run --offline --bin strip-ansi
```

The CLI accepts arguments or reads UTF-8 text from standard input.

## Test

Run the Rust unit tests:

```sh
cargo test --offline
```

Run differential tests against the installed npm package:

```sh
npm run test:differential
```

The differential suite uses npm `strip-ansi` as an oracle and compares exact output for representative CSI, OSC, C1, control-string, malformed, Unicode, and plain-text inputs.

This is an active compatibility rewrite. Passing the current corpus does not prove complete compatibility with every possible ANSI sequence.
