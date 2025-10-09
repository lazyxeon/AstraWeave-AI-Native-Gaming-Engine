//! File Watcher Module - Phase PBR-G Task 3
//!
//! Provides automatic hot-reload capabilities for material files and textures.
//!
//! # Features
//! - Watches `assets/materials/**/*.toml` for material definition changes
//! - Watches texture files (`*.png`, `*.ktx2`, `*.dds`) referenced by materials
//! - Debouncing (500ms) to avoid duplicate events from editor saves
//! - Thread-safe communication via channels (mpsc)
//! - Graceful error handling (continues watching even if some events fail)
//!
//! # Architecture
//! ```text
//! FileWatcher (notify thread) -> Channel -> MaterialInspector (main thread)
//!     │                                          │
//!     ├─ Watches assets/materials/               ├─ Receives reload events
//!     ├─ Debounces events (500ms)                ├─ Re-parses TOML
//!     └─ Sends ReloadEvent                       └─ Updates GPU buffers
//! ```
//!
//! # Usage
//! ```rust
//! // In MaterialInspector::new()
//! let watcher = FileWatcher::new("assets/materials")?;
//!
//! // In MaterialInspector::show() or update()
//! while let Ok(event) = self.file_watcher.try_recv() {
//!     match event {
//!         ReloadEvent::Material(path) => self.reload_material(&path),
//!         ReloadEvent::Texture(path) => self.reload_texture(&path),
//!     }
//! }
//! ```

use anyhow::{Context, Result};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Types of reload events
#[derive(Debug, Clone)]
pub enum ReloadEvent {
    /// A material TOML file changed
    Material(PathBuf),
    /// A texture file changed (albedo/normal/ORM)
    Texture(PathBuf),
}

/// File watcher state with debouncing
pub struct FileWatcher {
    /// notify watcher (keeps thread alive)
    _watcher: RecommendedWatcher,

    /// Receiver for reload events (debounced)
    pub receiver: Receiver<ReloadEvent>,
}

impl FileWatcher {
    /// Create a new file watcher for the given directory
    ///
    /// # Arguments
    /// - `watch_path`: Root directory to watch (e.g., "assets/materials")
    ///
    /// # Returns
    /// - `Ok(FileWatcher)` on success
    /// - `Err` if the directory doesn't exist or watcher creation fails
    pub fn new<P: AsRef<Path>>(watch_path: P) -> Result<Self> {
        let watch_path = watch_path.as_ref().to_path_buf();

        // Validate path exists
        if !watch_path.exists() {
            anyhow::bail!("Watch path does not exist: {}", watch_path.display());
        }

        // Create channels for debounced events
        let (tx, rx) = channel();

        // Debouncer state (shared between notify callback and debounce thread)
        let debounce_state = Arc::new(Mutex::new(DebounceState::new()));
        let debounce_state_clone = Arc::clone(&debounce_state);
        let tx_clone = tx.clone();

        // Create notify watcher
        let mut watcher = RecommendedWatcher::new(
            move |res: notify::Result<Event>| {
                if let Ok(event) = res {
                    Self::handle_notify_event(event, &debounce_state, &tx);
                }
            },
            Config::default(),
        )
        .context("Failed to create file watcher")?;

        // Start watching the directory recursively
        watcher
            .watch(&watch_path, RecursiveMode::Recursive)
            .context("Failed to watch directory")?;

        // Spawn debounce thread (processes buffered events after 500ms)
        std::thread::spawn(move || {
            Self::debounce_loop(debounce_state_clone, tx_clone);
        });

        Ok(FileWatcher {
            _watcher: watcher,
            receiver: rx,
        })
    }

    /// Try to receive a reload event (non-blocking)
    pub fn try_recv(&self) -> Result<ReloadEvent, std::sync::mpsc::TryRecvError> {
        self.receiver.try_recv()
    }

    /// Handle notify event (called from watcher thread)
    fn handle_notify_event(
        event: Event,
        debounce_state: &Arc<Mutex<DebounceState>>,
        _tx: &Sender<ReloadEvent>,
    ) {
        // Only care about modify/create/remove events
        let is_relevant = matches!(
            event.kind,
            EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
        );

        if !is_relevant {
            return;
        }

        // Add each path to debounce buffer
        for path in event.paths {
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();

                // Material TOML files
                if ext_str == "toml" {
                    let mut state = debounce_state.lock().unwrap();
                    state
                        .buffer
                        .insert(path.clone(), ReloadEvent::Material(path));
                }
                // Texture files
                else if matches!(
                    ext_str.as_str(),
                    "png" | "jpg" | "jpeg" | "ktx2" | "dds" | "basis"
                ) {
                    let mut state = debounce_state.lock().unwrap();
                    state
                        .buffer
                        .insert(path.clone(), ReloadEvent::Texture(path));
                }
            }
        }
    }

    /// Debounce loop (runs in separate thread)
    fn debounce_loop(debounce_state: Arc<Mutex<DebounceState>>, tx: Sender<ReloadEvent>) {
        const DEBOUNCE_DURATION: Duration = Duration::from_millis(500);

        loop {
            std::thread::sleep(Duration::from_millis(100));

            let mut state = debounce_state.lock().unwrap();

            // Check if any buffered events are ready to send
            let now = Instant::now();
            let ready_paths: Vec<PathBuf> = state
                .buffer
                .keys()
                .filter(|path| {
                    state
                        .last_event_time
                        .get(*path)
                        .map(|&time| now.duration_since(time) >= DEBOUNCE_DURATION)
                        .unwrap_or(true)
                })
                .cloned()
                .collect();

            // Send ready events and remove from buffer
            for path in ready_paths {
                if let Some(event) = state.buffer.remove(&path) {
                    // Send event (ignore errors if receiver dropped)
                    let _ = tx.send(event);
                    state.last_event_time.remove(&path);
                }
            }

            // Update last event times for remaining buffered events
            let now = Instant::now();
            let remaining_paths: Vec<PathBuf> = state.buffer.keys().cloned().collect();
            for path in remaining_paths {
                state.last_event_time.entry(path).or_insert(now);
            }
        }
    }
}

/// Debounce state (shared between threads)
struct DebounceState {
    /// Buffered events (path -> event)
    buffer: HashMap<PathBuf, ReloadEvent>,

    /// Last event time for each path (for debouncing)
    last_event_time: HashMap<PathBuf, Instant>,
}

impl DebounceState {
    fn new() -> Self {
        DebounceState {
            buffer: HashMap::new(),
            last_event_time: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    #[ignore] // Requires assets/materials directory
    fn test_watcher_creation() {
        let watcher = FileWatcher::new("assets/materials");
        assert!(watcher.is_ok(), "Failed to create watcher");
    }

    #[test]
    #[ignore] // Integration test (requires file system changes)
    fn test_material_reload() {
        let temp_dir = std::env::temp_dir().join("astraweave_test_materials");
        fs::create_dir_all(&temp_dir).unwrap();

        let watcher = FileWatcher::new(&temp_dir).unwrap();

        // Create a test material file
        let test_material = temp_dir.join("test.toml");
        let mut file = fs::File::create(&test_material).unwrap();
        writeln!(file, "[material]").unwrap();
        writeln!(file, "name = \"test\"").unwrap();
        drop(file);

        // Wait for debounce + processing
        std::thread::sleep(Duration::from_millis(700));

        // Check for reload event
        let event = watcher.try_recv();
        assert!(event.is_ok(), "Expected reload event");

        if let Ok(ReloadEvent::Material(path)) = event {
            assert_eq!(path, test_material);
        } else {
            panic!("Expected Material reload event");
        }

        // Cleanup
        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    #[ignore] // Integration test (requires file system changes)
    fn test_texture_reload() {
        let temp_dir = std::env::temp_dir().join("astraweave_test_textures");
        fs::create_dir_all(&temp_dir).unwrap();

        let watcher = FileWatcher::new(&temp_dir).unwrap();

        // Create a test texture file (empty PNG)
        let test_texture = temp_dir.join("test.png");
        fs::File::create(&test_texture).unwrap();

        // Wait for debounce + processing
        std::thread::sleep(Duration::from_millis(700));

        // Check for reload event
        let event = watcher.try_recv();
        assert!(event.is_ok(), "Expected reload event");

        if let Ok(ReloadEvent::Texture(path)) = event {
            assert_eq!(path, test_texture);
        } else {
            panic!("Expected Texture reload event");
        }

        // Cleanup
        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_debounce() {
        // Create watcher with temporary directory
        let temp_dir = std::env::temp_dir().join("astraweave_test_debounce");
        fs::create_dir_all(&temp_dir).unwrap();

        let watcher = FileWatcher::new(&temp_dir).unwrap();

        // Create and modify file multiple times rapidly
        let test_file = temp_dir.join("test.toml");
        for i in 0..5 {
            let mut file = fs::File::create(&test_file).unwrap();
            writeln!(file, "[material]").unwrap();
            writeln!(file, "version = {}", i).unwrap();
            drop(file);
            std::thread::sleep(Duration::from_millis(50));
        }

        // Wait for debounce
        std::thread::sleep(Duration::from_millis(700));

        // Should receive only ONE event (debounced)
        let mut event_count = 0;
        while watcher.try_recv().is_ok() {
            event_count += 1;
        }

        assert_eq!(event_count, 1, "Expected exactly 1 debounced event");

        // Cleanup
        fs::remove_dir_all(&temp_dir).ok();
    }
}
