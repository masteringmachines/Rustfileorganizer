mod categorize;
mod organizer;
mod undo;

use categorize::Rules;
use clap::Parser;
use organizer::Mode;
use std::path::PathBuf;

/// Organize files in a directory into subfolders by type or by date.
#[derive(Parser, Debug)]
#[command(name = "file-organizer", version, about)]
struct Args {
    /// Directory to organize
    #[arg(default_value = ".")]
    dir: PathBuf,

    /// Sort into folders by modified date (YYYY-MM) instead of file type
    #[arg(long)]
    by_date: bool,

    /// Descend into subdirectories too (skips folders the organizer made itself)
    #[arg(long)]
    recursive: bool,

    /// Show what would happen without moving anything
    #[arg(long)]
    dry_run: bool,

    /// Path to a TOML file overriding category -> extension rules
    #[arg(long)]
    config: Option<PathBuf>,

    /// Undo the last run in this directory
    #[arg(long)]
    undo: bool,
}

fn main() {
    let args = Args::parse();

    if !args.dir.is_dir() {
        eprintln!("{:?} is not a directory", args.dir);
        std::process::exit(1);
    }

    if args.undo {
        match undo::undo(&args.dir) {
            Ok(n) => println!("Reverted {n} file(s)."),
            Err(e) => {
                eprintln!("Nothing to undo, or undo failed: {e}");
                std::process::exit(1);
            }
        }
        return;
    }

    let rules = Rules::load(args.config.as_deref());
    let mode = if args.by_date { Mode::ByDate } else { Mode::ByType };

    let planned = match organizer::plan(&args.dir, &mode, &rules, args.recursive) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to scan {:?}: {e}", args.dir);
            std::process::exit(1);
        }
    };

    if planned.is_empty() {
        println!("Nothing to organize — no matching files found.");
        return;
    }

    if args.dry_run {
        println!("Dry run — {} file(s) would move:", planned.len());
        for m in &planned {
            println!("  {} -> {}", m.from.display(), m.to.display());
        }
        println!("\nRun without --dry-run to apply.");
        return;
    }

    let done = organizer::execute(&planned);
    if let Err(e) = undo::record(&args.dir, &done) {
        eprintln!("Warning: couldn't write undo log: {e}");
    }

    println!("Moved {} of {} file(s).", done.len(), planned.len());
    if !done.is_empty() {
        println!("Run with --undo to reverse this.");
    }
}
