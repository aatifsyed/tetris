# `tetris`
## Running
```sh
cargo run < input.txt > output.txt
```
or
```sh
cargo run -- --infile input.txt --output output.txt
```

```console
$ cargo run -- --help
DRW Tetris
==========

For each line in the input, interpret that line as a comma-separated sequence of INPUT_BLOCK, where
INPUT_BLOCK : { 'Q', 'Z', 'S', 'T', 'I', 'L', 'J' } + DIGIT

Each INPUT_BLOCK is placed on a 10 * 103 GRID at INPUT_BLOCK.DIGIT position, and dropped.
Rows clear in typical tetris style.

After the sequence has been processed, print the height of the tallest occupied row.


Usage: tetris [OPTIONS]

Options:
  -i, --infile <INFILE>    The input file (defaults to stdin)
  -o, --outfile <OUTFILE>  The output file (defaults to stdout)
  -h, --help               Print help information
```

## Developing
### Lint
```sh
cargo clippy && cargo +nightly udeps
```

### Test
Unit tests are in `src`, integration tests are in `tests`
```sh
cargo test
```
