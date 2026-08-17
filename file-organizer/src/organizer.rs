use crate::categorize::Rules;
use chrono::{DateTime, Local};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// One planned move: file currently at `from`, would end up at `to`.
pub struct PlannedMove {
    pub from: PathBuf,
    pub to: PathBuf,
}

/// How to bucket files into subfolders.
pub enum Mode {
    ByType,
    ByDate,
}

/// Scan `dir` (non-recursive by default; pass recursive=true to descend)
/// and build the list of moves that would organize it. Does not touch disk.
pub fn plan(
    dir: &Path,
    mode: &Mode,
    rules: &Rules,
    recursive: bool,
) -> std::io::Result<Vec<PlannedMove>> {
    let mut moves = Vec::new();
    scan_dir(dir, dir, mode, rules, recursive, &mut moves)?;
    Ok(moves)
}

fn scan_dir(
    root: &Path,
    dir: &Path,
    mode: &Mode,
    rules: &Rules,
    recursive: bool,
    moves: &mut Vec<PlannedMove>,
) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            // Never descend into folders we created ourselves — avoids
            // re-organizing already-organized output on repeated runs.
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if is_managed_folder(&name) {
                continue;
            }
            if recursive {
                scan_dir(root, &path, mode, rules, recursive, moves)?;
            }
            continue;
        }

        if !file_type.is_file() {
            continue;
        }

        // Skip dotfiles (.DS_Store, .gitignore, etc.) — rarely what someone
        // wants swept into a category folder.
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with('.') {
                continue;
            }
        }

        let bucket = match mode {
            Mode::ByType => {
                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("");
                if ext.is_empty() {
                    "Other".to_string()
                } else {
                    rules.categorize(ext)
                }
            }
            Mode::ByDate => date_bucket(&path)?,
        };

        let dest_dir = root.join(&bucket);
        let dest_path = unique_dest(&dest_dir, &path);
        moves.push(PlannedMove { from: path, to: dest_path });
    }
    Ok(())
}

fn is_managed_folder(name: &str) -> bool {
    matches!(
        name,
        "Images" | "Documents" | "Spreadsheets" | "Presentations" | "Video"
            | "Audio" | "Archives" | "Code" | "Installers" | "Other"
    ) || name.chars().take(4).all(|c| c.is_ascii_digit()) // e.g. "2025-06" date buckets
}

fn date_bucket(path: &Path) -> std::io::Result<String> {
    let metadata = fs::metadata(path)?;
    let modified: SystemTime = metadata.modified()?;
    let datetime: DateTime<Local> = modified.into();
    Ok(datetime.format("%Y-%m").to_string())
}

/// If `dest_dir/filename` already exists, append " (1)", " (2)", etc.
/// until we find a name that's free — never silently overwrite a file.
fn unique_dest(dest_dir: &Path, original: &Path) -> PathBuf {
    let file_name = original.file_name().unwrap();
    let mut candidate = dest_dir.join(file_name);
    if !candidate.exists() {
        return candidate;
    }

    let stem = original.file_stem().unwrap().to_string_lossy().to_string();
    let ext = original.extension().map(|e| e.to_string_lossy().to_string());

    let mut n = 1;
    loop {
        let new_name = match &ext {
            Some(ext) => format!("{stem} ({n}).{ext}"),
            None => format!("{stem} ({n})"),
        };
        candidate = dest_dir.join(new_name);
        if !candidate.exists() {
            return candidate;
        }
        n += 1;
    }
}

/// Execute a planned set of moves, creating destination folders as needed.
/// Returns the list of moves that actually succeeded (for the undo log).
pub fn execute(moves: &[PlannedMove]) -> Vec<(PathBuf, PathBuf)> {
    let mut done = Vec::new();
    for m in moves {
        if let Some(parent) = m.to.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!("Failed to create {parent:?}: {e}");
                continue;
            }
        }
        match fs::rename(&m.from, &m.to) {
            Ok(_) => done.push((m.from.clone(), m.to.clone())),
            Err(e) => eprintln!("Failed to move {:?} -> {:?}: {e}", m.from, m.to),
        }
    }
    done
}
