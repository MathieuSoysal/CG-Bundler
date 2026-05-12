//! `--watch` command: rebuild on filesystem changes until interrupted.

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

use colored::Colorize;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use cg_bundler::BundlerError;

use crate::cli::Cli;
use crate::commands::bundle;

/// Run the watch command described by `cli`.
///
/// Performs an initial build, then enters an event loop that triggers
/// rebuilds whenever a Rust source file changes. Returns when Ctrl+C is
/// received or when the watcher channel disconnects.
///
/// # Errors
/// Returns an [`BundlerError`] if the source directory does not exist or if
/// the OS-level watcher / signal handler cannot be installed.
pub fn run(cli: &Cli) -> Result<(), BundlerError> {
    print_banner(cli);

    let watch_path = resolve_src_dir(cli)?;
    let shutdown_rx = install_shutdown_handler()?;

    perform_initial_build(cli);

    let (event_rx, _watcher) = install_file_watcher(&watch_path)?;
    run_event_loop(cli, &event_rx, &shutdown_rx);

    println!("{} Watch mode stopped.", "🛑".red());
    Ok(())
}

fn print_banner(cli: &Cli) {
    println!("{} Starting watch mode...", "🔍".green());
    println!("{} Watching directory: {}", "📁".blue(), cli.src_dir);
    if let Some(output) = &cli.output {
        println!("{} Output file: {}", "📄".blue(), output.display());
    } else {
        println!("{} Output: stdout", "📄".blue());
    }
    println!("{} Debounce delay: {}ms", "⏱️".blue(), cli.debounce);
    println!("{} Press Ctrl+C to stop\n", "ℹ️".yellow());
}

fn resolve_src_dir(cli: &Cli) -> Result<PathBuf, BundlerError> {
    let watch_path = cli.get_project_path().join(&cli.src_dir);
    if !watch_path.exists() {
        return Err(BundlerError::Io {
            source: std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Source directory '{}' does not exist", cli.src_dir),
            ),
            path: Some(watch_path),
        });
    }
    Ok(watch_path)
}

fn install_shutdown_handler() -> Result<Receiver<()>, BundlerError> {
    let (tx, rx) = mpsc::channel();
    ctrlc::set_handler(move || {
        let _ = tx.send(());
    })
    .map_err(|e| BundlerError::Io {
        source: std::io::Error::other(e.to_string()),
        path: None,
    })?;
    Ok(rx)
}

fn perform_initial_build(cli: &Cli) {
    if let Err(e) = bundle::run(cli) {
        eprintln!("{} Initial build failed: {}", "❌".red(), e);
    } else {
        println!("{} Initial build successful!\n", "✅".green());
    }
}

type WatchEvent = Result<Event, notify::Error>;

fn install_file_watcher(
    watch_path: &std::path::Path,
) -> Result<(Receiver<WatchEvent>, RecommendedWatcher), BundlerError> {
    let (tx, rx) = mpsc::channel();
    let mut watcher = notify::recommended_watcher(tx).map_err(|e| BundlerError::Io {
        source: std::io::Error::other(e.to_string()),
        path: None,
    })?;

    watcher
        .watch(watch_path, RecursiveMode::Recursive)
        .map_err(|e| BundlerError::Io {
            source: std::io::Error::other(e.to_string()),
            path: Some(watch_path.to_path_buf()),
        })?;

    Ok((rx, watcher))
}

fn run_event_loop(cli: &Cli, events: &Receiver<WatchEvent>, shutdown: &Receiver<()>) {
    let debounce = Duration::from_millis(cli.debounce);
    let mut last_event = Instant::now();

    loop {
        if shutdown.try_recv() == Ok(()) {
            println!("\n{} Received shutdown signal", "🛑".yellow());
            break;
        }

        match events.recv_timeout(Duration::from_millis(100)) {
            Ok(Ok(event)) => {
                if !should_rebuild(&event) {
                    continue;
                }
                let now = Instant::now();
                if now.duration_since(last_event) <= debounce {
                    continue;
                }
                last_event = now;
                announce_change(&event);
                trigger_rebuild(cli);
            }
            Ok(Err(e)) => eprintln!("{} Watch error: {}", "⚠️".yellow(), e),
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
}

fn announce_change(event: &Event) {
    if let Some(path) = event.paths.first()
        && let Some(file_name) = path.file_name()
    {
        println!(
            "{} File change detected: {}",
            "🔄".yellow(),
            file_name.to_string_lossy()
        );
        return;
    }
    println!("{} File change detected", "🔄".yellow());
}

fn trigger_rebuild(cli: &Cli) {
    match bundle::run(cli) {
        Ok(()) => println!("{} Rebuild successful!\n", "✅".green()),
        Err(e) => eprintln!("{} Rebuild failed: {}\n", "❌".red(), e),
    }
}

/// Whether a notify event should trigger a rebuild (only Rust source files).
#[must_use]
pub fn should_rebuild(event: &Event) -> bool {
    match &event.kind {
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
            event.paths.iter().any(|path| {
                path.extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext| ext == "rs")
            })
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::make_project;
    use clap::Parser;
    use tempfile::TempDir;

    #[test]
    fn test_handle_watch_command_missing_src_dir_no_output() {
        let tmp = TempDir::new().unwrap();
        make_project(tmp.path(), "fn main() {}");
        let cli = Cli::try_parse_from([
            "cg-bundler",
            tmp.path().to_str().unwrap(),
            "--watch",
            "--src-dir",
            "nonexistent_xyz",
        ])
        .unwrap();
        let result = run(&cli);
        assert!(result.is_err());
        match result.unwrap_err() {
            BundlerError::Io { .. } => {}
            e => panic!("Expected Io error, got: {e}"),
        }
    }

    #[test]
    fn test_handle_watch_command_missing_src_dir_with_output() {
        let tmp = TempDir::new().unwrap();
        make_project(tmp.path(), "fn main() {}");
        let out = tmp.path().join("out.rs");
        let cli = Cli::try_parse_from([
            "cg-bundler",
            tmp.path().to_str().unwrap(),
            "--watch",
            "--src-dir",
            "nonexistent_xyz",
            "-o",
            out.to_str().unwrap(),
        ])
        .unwrap();
        assert!(run(&cli).is_err());
    }

    #[test]
    fn test_should_rebuild_create_rs_file() {
        use notify::event::{CreateKind, EventKind};
        let event = Event {
            kind: EventKind::Create(CreateKind::File),
            paths: vec![PathBuf::from("src/main.rs")],
            attrs: notify::event::EventAttributes::default(),
        };
        assert!(should_rebuild(&event));
    }

    #[test]
    fn test_should_rebuild_modify_rs_file() {
        use notify::event::{EventKind, ModifyKind};
        let event = Event {
            kind: EventKind::Modify(ModifyKind::Data(notify::event::DataChange::Any)),
            paths: vec![PathBuf::from("src/lib.rs")],
            attrs: notify::event::EventAttributes::default(),
        };
        assert!(should_rebuild(&event));
    }

    #[test]
    fn test_should_rebuild_non_rs_file() {
        use notify::event::{EventKind, ModifyKind};
        let event = Event {
            kind: EventKind::Modify(ModifyKind::Data(notify::event::DataChange::Any)),
            paths: vec![PathBuf::from("README.md")],
            attrs: notify::event::EventAttributes::default(),
        };
        assert!(!should_rebuild(&event));
    }

    #[test]
    fn test_should_rebuild_remove_rs_file() {
        use notify::event::{EventKind, RemoveKind};
        let event = Event {
            kind: EventKind::Remove(RemoveKind::File),
            paths: vec![PathBuf::from("src/utils.rs")],
            attrs: notify::event::EventAttributes::default(),
        };
        assert!(should_rebuild(&event));
    }

    #[test]
    fn test_should_rebuild_access_event() {
        use notify::event::{AccessKind, EventKind};
        let event = Event {
            kind: EventKind::Access(AccessKind::Read),
            paths: vec![PathBuf::from("src/main.rs")],
            attrs: notify::event::EventAttributes::default(),
        };
        assert!(!should_rebuild(&event));
    }
}
