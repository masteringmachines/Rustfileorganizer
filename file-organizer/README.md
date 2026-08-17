# file-organizer

A CLI tool that sorts messy directories into subfolders — by file type or
by modified date — with a dry-run mode and an undo log so it's safe to
point at a real Downloads folder.

## Usage

```bash
# Preview what would happen (recommended first run)
cargo run --release -- ~/Downloads --dry-run

# Actually organize by type (Images/, Documents/, Video/, etc.)
cargo run --release -- ~/Downloads

# Organize by modified date instead (2025-06/, 2025-07/, ...)
cargo run --release -- ~/Downloads --by-date

# Include subfolders (skips folders the tool created itself)
cargo run --release -- ~/Downloads --recursive

# Made a mistake? Undo the last run in that directory
cargo run --release -- ~/Downloads --undo

# Use your own category rules instead of the built-in defaults
cargo run --release -- ~/Downloads --config rules.example.toml
```

## How it works

1. `organizer::plan` scans the target directory (non-recursive unless
   `--recursive`) and builds a list of planned moves — nothing touches
   disk yet. Name collisions get `(1)`, `(2)`, etc. appended rather than
   overwriting anything.
2. `--dry-run` just prints that plan.
3. Otherwise `organizer::execute` performs the moves and `undo::record`
   appends each one to a `.organizer-log.txt` file in the target
   directory.
4. `--undo` reads that log and moves everything back, most recent first,
   then clears the log.

Dotfiles are always skipped. Folders the tool itself creates (category
names, or `YYYY-MM` date buckets) are never re-descended into on repeat
runs, so running it twice in a row is a no-op rather than a mess.

## Customizing categories

Copy `rules.example.toml`, edit it, and pass `--config your-rules.toml`.
Any extension not covered by your config (or the built-in defaults) lands
in an `Other/` folder.

## Note

This was written without a local Rust toolchain to compile against, so
while the logic has been reviewed carefully, do run `cargo build` first
and check for any dependency-version drift (particularly around `chrono`
or `toml`'s API) before trusting it on real files. `--dry-run` is your
friend for the same reason.

## Possible extensions

- `--min-age <days>` to only organize files older than N days
- A `--watch` mode that organizes new files as they land, via `notify`
- Duplicate detection (hash-based) alongside the name-collision handling
