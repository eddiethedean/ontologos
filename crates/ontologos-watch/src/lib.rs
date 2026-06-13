//! File-watch API for reloading OWL ontologies (library hook for Ontocode).
//!
//! CLI `--watch` is planned for v1.2; this crate exposes the reload primitive only.

#![warn(missing_docs)]

mod error;

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::time::Duration;

use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use ontologos_core::Ontology;
use ontologos_parser::load_ontology;

pub use error::{Error, Result};

/// A filesystem change relevant to an ontology file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchEvent {
    /// Path that changed.
    pub path: PathBuf,
    /// Reloaded ontology (parse errors surface as [`Error::Parse`]).
    pub ontology: Ontology,
}

/// Watch `path` and invoke `on_change` when the file is modified (debounced).
///
/// Blocks until the watcher channel closes or `on_change` returns an error.
pub fn watch_ontology_path<F>(
    path: impl AsRef<Path>,
    debounce_ms: u64,
    mut on_change: F,
) -> Result<()>
where
    F: FnMut(WatchEvent) -> Result<()>,
{
    let path = path.as_ref().to_path_buf();
    let (tx, rx) = mpsc::channel();
    let mut watcher = RecommendedWatcher::new(
        move |res: notify::Result<Event>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        Config::default(),
    )
    .map_err(|e| Error::Watch(e.to_string()))?;

    watcher
        .watch(
            path.parent().unwrap_or(path.as_path()),
            RecursiveMode::NonRecursive,
        )
        .map_err(|e| Error::Watch(e.to_string()))?;

    let debounce = Duration::from_millis(debounce_ms);
    let mut last_fire = std::time::Instant::now()
        .checked_sub(debounce)
        .unwrap_or_else(std::time::Instant::now);

    loop {
        match rx.recv_timeout(debounce) {
            Ok(event) => {
                if !event.paths.iter().any(|p| p == &path) {
                    continue;
                }
                if last_fire.elapsed() < debounce {
                    continue;
                }
                last_fire = std::time::Instant::now();
                let ontology = load_ontology(&path).map_err(Error::Parse)?;
                on_change(WatchEvent {
                    path: path.clone(),
                    ontology,
                })?;
            }
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
    Ok(())
}

/// Non-blocking receiver for tests: watch and return first reload event or parse error.
#[doc(hidden)]
pub fn watch_once(
    path: impl AsRef<Path>,
    debounce_ms: u64,
) -> Result<Receiver<std::result::Result<WatchEvent, Error>>> {
    let path = path.as_ref().to_path_buf();
    let (out_tx, out_rx) = mpsc::channel();
    let (notify_tx, notify_rx) = mpsc::channel();
    let mut watcher = RecommendedWatcher::new(
        move |res: notify::Result<Event>| {
            if let Ok(event) = res {
                let _ = notify_tx.send(event);
            }
        },
        Config::default().with_poll_interval(Duration::from_millis(100)),
    )
    .map_err(|e| Error::Watch(e.to_string()))?;
    watcher
        .watch(path.as_path(), RecursiveMode::NonRecursive)
        .map_err(|e| Error::Watch(e.to_string()))?;

    std::thread::spawn(move || {
        let _watcher = watcher;
        let debounce = Duration::from_millis(debounce_ms);
        while let Ok(event) = notify_rx.recv() {
            if event
                .paths
                .iter()
                .any(|p| p == &path || p.file_name().is_some() && p.file_name() == path.file_name())
            {
                let result =
                    load_ontology(&path)
                        .map_err(Error::Parse)
                        .map(|ontology| WatchEvent {
                            path: path.clone(),
                            ontology,
                        });
                let _ = out_tx.send(result);
                break;
            }
            std::thread::sleep(debounce);
        }
    });
    Ok(out_rx)
}
