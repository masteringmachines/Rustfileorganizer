use std::fs;
use std::path::{Path, PathBuf};

const LOG_NAME: &str = ".organizer-log.txt";

/// Append executed moves to a log file in `dir`, one "from\tto" per line.
/// Simple tab-separated text log — no need for a real DB for this use case.
pub fn record(dir: &Path, moves: &[(PathBuf, PathBuf)]) -> std::io::Result<()> {
    if moves.is_empty() {
        return Ok(());
    }
    let log_path = dir.join(LOG_NAME);
    let mut contents = String::new();
    for (from, to) in moves {
        contents.push_str(&format!("{}\t{}\n", from.display(), to.display()));
    }
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;
    file.write_all(contents.as_bytes())
}

/// Read the log and move every file back to where it came from, most
/// recent moves first. Clears the log on success so undo isn't repeatable
/// against stale entries.
pub fn undo(dir: &Path) -> std::io::Result<usize> {
    let log_path = dir.join(LOG_NAME);
    let contents = fs::read_to_string(&log_path)?;

    let mut reverted = 0;
    for line in contents.lines().rev() {
        let mut parts = line.splitn(2, '\t');
        let (Some(from), Some(to)) = (parts.next(), parts.next()) else {
            continue;
        };
        let from = PathBuf::from(from);
        let to = PathBuf::from(to);

        if to.exists() {
            if let Err(e) = fs::rename(&to, &from) {
                eprintln!("Failed to undo {to:?} -> {from:?}: {e}");
                continue;
            }
            reverted += 1;
        } else {
            eprintln!("Skipping undo, file missing: {to:?}");
        }
    }

    fs::remove_file(&log_path)?;
    Ok(reverted)
}
