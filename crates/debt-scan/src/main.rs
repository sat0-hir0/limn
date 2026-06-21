//! debt-scan — internal dev tool for tracking technical debt.
//!
//! Inspired by ow-my-coach's `scripts/debt/scan.mjs`, ported to Rust and
//! pared down to editor's surface area. Invoke with:
//!
//! ```sh
//! cargo run -p debt-scan -- scan
//! cargo run -p debt-scan -- gate
//! cargo run -p debt-scan -- gate --update-baseline
//! ```

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

mod report;
mod scanner;

#[derive(Parser)]
#[command(version, about = "Track technical debt in the editor workspace.")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Walk the repo, count debt categories, and write reports.
    Scan {
        /// Repo root (default: current directory).
        #[arg(long, default_value = ".")]
        root: PathBuf,
    },
    /// Run scan and compare against `docs/debt/baseline.json`.
    Gate {
        #[arg(long, default_value = ".")]
        root: PathBuf,
        /// Overwrite baseline.json with the current counts (use sparingly).
        #[arg(long)]
        update_baseline: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Scan { root } => run_scan(&root),
        Command::Gate {
            root,
            update_baseline,
        } => run_gate(&root, update_baseline),
    }
}

fn run_scan(root: &Path) -> Result<()> {
    let counts = scanner::scan(root)?;
    report::write_reports(root, &counts).context("write reports")?;
    print_counts(&counts);
    Ok(())
}

fn run_gate(root: &Path, update_baseline: bool) -> Result<()> {
    let counts = scanner::scan(root)?;
    report::write_reports(root, &counts).context("write reports")?;

    let baseline_path = root.join("docs/debt/baseline.json");
    if update_baseline {
        report::write_baseline(&baseline_path, &counts).context("write baseline")?;
        println!("baseline updated -> {}", baseline_path.display());
        return Ok(());
    }

    let baseline = report::read_baseline(&baseline_path).context("read baseline")?;
    let diff = report::diff(&baseline, &counts);
    print_diff(&diff);

    let regressed: Vec<&String> = diff
        .iter()
        .filter_map(|(k, d)| if *d > 0 { Some(k) } else { None })
        .collect();
    if regressed.is_empty() {
        println!("\ndebt-scan gate: ok (no category over baseline)");
        Ok(())
    } else {
        anyhow::bail!(
            "debt-scan gate: regressions in {} category/categories: {}",
            regressed.len(),
            regressed
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
}

fn print_counts(counts: &BTreeMap<String, u64>) {
    println!("debt-scan results:");
    for (k, v) in counts {
        println!("  {k:30} {v:>6}");
    }
}

fn print_diff(diff: &BTreeMap<String, i64>) {
    println!("\ndebt-scan diff vs baseline:");
    for (k, v) in diff {
        let sign = if *v > 0 { "+" } else { "" };
        println!("  {k:30} {sign}{v}");
    }
}
