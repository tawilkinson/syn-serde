# rust2json

Parse a Rust source file into a `syn_serde::File` and print out a JSON
representation of the syntax tree.

## Usage

```text
cargo run -- <input_path> [output_path]
cargo run -- --compact <input_path> [output_path]
```

The `--compact` flag generates JSON output without span information, resulting in
significantly smaller and more readable output when location data is not needed.

## Examples

Regular output with spans:
```text
cargo run -- src/main.rs
```

Compact output without spans:
```text
cargo run -- --compact src/main.rs
```

The regular output is the same as [rust2json_main.json](rust2json_main.json).
