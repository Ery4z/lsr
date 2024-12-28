# LSR - Thomas BOLTEAU 2025

## Description

Simple rewrite of basic functionnality of the `ls` command in Rust.

## Usage

```bash

cargo build --release

./target/release/lsr --help

```

## Help

```text

Usage: lsr [OPTIONS] [path]

Arguments:
  [path]

Options:
  -a             List the hidden files
  -l             List the metadata properties
  -c             Colorize the output
  -h, --help     Print help
  -V, --version  Print version

```

## Tests

```bash
cargo test
```

## Example

List the hidden files in the current directory with metadata properties and coloring the output:

```bash

./target/release/lsr -lac

```
