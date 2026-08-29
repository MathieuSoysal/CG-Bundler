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
    run_event_loop(
        cli,
        resolve_output_path(cli).as_ref(),
        &event_rx,
        &shutdown_rx,
    );

    eprintln!("{} Watch mode stopped.", "🛑".red());
    Ok(())
}

fn print_banner(cli: &Cli) {
    // Status goes to stderr so `--watch` without `-o` keeps stdout free for the bundle.
    eprintln!("{} Starting watch mode...", "🔍".green());
    eprintln!("{} Watching directory: {}", "📁".blue(), cli.src_dir);
    if let Some(output) = &cli.output {
        eprintln!("{} Output file: {}", "📄".blue(), output.display());
    } else {
        eprintln!("{} Output: stdout", "📄".blue());
    }
    eprintln!("{} Debounce delay: {}ms", "⏱️".blue(), cli.debounce);
    eprintln!("{} Press Ctrl+C to stop\n", "ℹ️".yellow());
}

/// The bundle's destination, resolved so it can be compared against event paths.
///
/// Writing the bundle into the watched directory would otherwise be seen as a
/// source change and rebuild forever.
fn resolve_output_path(cli: &Cli) -> Option<PathBuf> {
    let output = cli.output.as_ref()?;
    let absolute = if output.is_absolute() {
        output.clone()
    } else {
        cli.get_project_path().join(output)
    };

    // The file may not exist yet, so canonicalise the directory and re-attach the
    // file name rather than canonicalising the whole path.
    let resolved = absolute
        .parent()
        .zip(absolute.file_name())
        .and_then(|(parent, name)| parent.canonicalize().ok().map(|dir| dir.join(name)));

    Some(resolved.unwrap_or(absolute))
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
        eprintln!("{} Initial build successful!\n", "✅".green());
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

fn run_event_loop(
    cli: &Cli,
    output_path: Option<&PathBuf>,
    events: &Receiver<WatchEvent>,
    shutdown: &Receiver<()>,
) {
    let debounce = Duration::from_millis(cli.debounce);
    let mut pending: Option<(Instant, Option<String>)> = None;

    loop {
        if shutdown.try_recv() == Ok(()) {
            eprintln!("\n{} Received shutdown signal", "🛑".yellow());
            break;
        }

        match events.recv_timeout(Duration::from_millis(50)) {
            Ok(Ok(event)) => {
                if is_rebuild_trigger(&event, output_path) {
                    // Trailing debounce: a burst of writes yields one rebuild of the
                    // final state rather than one rebuild of the first state.
                    pending = Some((Instant::now(), changed_file_name(&event)));
                }
            }
            Ok(Err(e)) => eprintln!("{} Watch error: {}", "⚠️".yellow(), e),
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }

        if pending
            .as_ref()
            .is_some_and(|(since, _)| since.elapsed() >= debounce)
        {
            let (_, name) = pending.take().expect("pending checked above");
            announce_change(name.as_deref());
            trigger_rebuild(cli);
        }
    }
}

fn changed_file_name(event: &Event) -> Option<String> {
    event
        .paths
        .first()
        .and_then(|path| path.file_name())
        .map(|name| name.to_string_lossy().into_owned())
}

fn announce_change(file_name: Option<&str>) {
    match file_name {
        Some(name) => eprintln!("{} File change detected: {}", "🔄".yellow(), name),
        None => eprintln!("{} File change detected", "🔄".yellow()),
    }
}

fn trigger_rebuild(cli: &Cli) {
    match bundle::run(cli) {
        Ok(()) => eprintln!("{} Rebuild successful!\n", "✅".green()),
        Err(e) => eprintln!("{} Rebuild failed: {}\n", "❌".red(), e),
    }
}

/// Whether a notify event should trigger a rebuild (only Rust source files).
#[must_use]
/// Whether `event` should start the debounce window for a rebuild.
///
/// This is the single predicate the event loop consults, so the tests exercise
/// exactly what the loop does.
fn is_rebuild_trigger(event: &Event, output_path: Option<&PathBuf>) -> bool {
    should_rebuild(event) && !targets_output(event, output_path)
}

/// Whether `event` only reports the bundle being written back out.
///
/// `-o` may legitimately point inside the watched directory; without this the
/// rebuild would observe its own output and loop forever.
fn targets_output(event: &Event, output_path: Option<&PathBuf>) -> bool {
    let Some(output) = output_path else {
        return false;
    };

    event.paths.iter().all(|path| {
        let resolved = path
            .parent()
            .and_then(|parent| parent.canonicalize().ok())
            .and_then(|dir| path.file_name().map(|name| dir.join(name)));

        resolved.as_ref() == Some(output) || path == output
    })
}

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

    fn event_for(paths: &[PathBuf]) -> Event {
        Event {
            kind: EventKind::Modify(notify::event::ModifyKind::Any),
            paths: paths.to_vec(),
            attrs: notify::event::EventAttributes::default(),
        }
    }

    /// Writing the bundle into the watched directory used to be seen as a source
    /// change, so every rebuild triggered the next one.
    #[test]
    fn output_file_events_do_not_trigger_a_rebuild() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src");
        std::fs::create_dir_all(&src).unwrap();
        let output = src.join("bundle.rs");
        std::fs::write(&output, "fn main() {}").unwrap();

        let cli = Cli::try_parse_from([
            "cg-bundler",
            tmp.path().to_str().unwrap(),
            "--watch",
            "-o",
            output.to_str().unwrap(),
        ])
        .unwrap();
        let resolved = resolve_output_path(&cli);

        let event = event_for(std::slice::from_ref(&output));
        assert!(should_rebuild(&event), "sanity: a .rs write is interesting");
        assert!(
            !is_rebuild_trigger(&event, resolved.as_ref()),
            "the bundle's own output must not start a rebuild"
        );
    }

    /// A real source edit must still get through the filter.
    #[test]
    fn source_file_events_still_trigger_a_rebuild() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src");
        std::fs::create_dir_all(&src).unwrap();
        let output = src.join("bundle.rs");
        std::fs::write(&output, "fn main() {}").unwrap();
        let main_rs = src.join("main.rs");
        std::fs::write(&main_rs, "fn main() {}").unwrap();

        let cli = Cli::try_parse_from([
            "cg-bundler",
            tmp.path().to_str().unwrap(),
            "--watch",
            "-o",
            output.to_str().unwrap(),
        ])
        .unwrap();
        let resolved = resolve_output_path(&cli);

        let event = event_for(&[main_rs]);
        assert!(is_rebuild_trigger(&event, resolved.as_ref()));
    }

    /// An event naming both the output and a real source file is a real change.
    #[test]
    fn mixed_events_still_trigger_a_rebuild() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src");
        std::fs::create_dir_all(&src).unwrap();
        let output = src.join("bundle.rs");
        std::fs::write(&output, "fn main() {}").unwrap();
        let main_rs = src.join("main.rs");
        std::fs::write(&main_rs, "fn main() {}").unwrap();

        let cli = Cli::try_parse_from([
            "cg-bundler",
            tmp.path().to_str().unwrap(),
            "--watch",
            "-o",
            output.to_str().unwrap(),
        ])
        .unwrap();
        let resolved = resolve_output_path(&cli);

        let event = event_for(&[output, main_rs]);
        assert!(is_rebuild_trigger(&event, resolved.as_ref()));
    }

    /// Without `-o` the bundle goes to stdout and nothing needs filtering.
    #[test]
    fn stdout_output_filters_nothing() {
        let tmp = TempDir::new().unwrap();
        let cli =
            Cli::try_parse_from(["cg-bundler", tmp.path().to_str().unwrap(), "--watch"]).unwrap();

        assert!(resolve_output_path(&cli).is_none());
        assert!(is_rebuild_trigger(
            &event_for(&[tmp.path().join("src/main.rs")]),
            None
        ));
    }

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
