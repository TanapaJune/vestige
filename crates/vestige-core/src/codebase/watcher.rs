//! File system watching for automatic learning
//!
//! This module watches the codebase for changes and:
//! - Records co-edit patterns (files changed together)
//! - Triggers pattern detection on modified files
//! - Updates relationship strengths based on activity
//!
//! This enables Vestige to learn continuously from developer behavior
//! without requiring explicit user input.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::{broadcast, mpsc, RwLock};

use super::patterns::PatternDetector;
use super::relationships::RelationshipTracker;

// ============================================================================
// ERRORS
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum WatcherError {
    #[error("Watcher error: {0}")]
    Notify(#[from] notify::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Channel error: {0}")]
    Channel(String),
    #[error("Already watching: {0}")]
    AlreadyWatching(PathBuf),
    #[error("Not watching: {0}")]
    NotWatching(PathBuf),
    #[error("Relationship error: {0}")]
    Relationship(#[from] super::relationships::RelationshipError),
}

pub type Result<T> = std::result::Result<T, WatcherError>;

// ============================================================================
// FILE EVENT
// ============================================================================

/// Represents a file change event
#[derive(Debug, Clone)]
pub struct FileEvent {
    /// Type of event
    pub kind: FileEventKind,
    /// Path(s) affected
    pub paths: Vec<PathBuf>,
    /// When the event occurred
    pub timestamp: DateTime<Utc>,
}

/// Types of file events
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileEventKind {
    /// File was created
    Created,
    /// File was modified
    Modified,
    /// File was deleted
    Deleted,
    /// File was renamed
    Renamed,
    /// Access event (read)
    Accessed,
}

impl From<EventKind> for FileEventKind {
    fn from(kind: EventKind) -> Self {
        match kind {
            EventKind::Create(_) => Self::Created,
            EventKind::Modify(_) => Self::Modified,
            EventKind::Remove(_) => Self::Deleted,
            EventKind::Access(_) => Self::Accessed,
            _ => Self::Modified, // Default to modified
        }
    }
}

// ============================================================================
// WATCHER CONFIG
// ============================================================================

/// Configuration for the codebase watcher
#[derive(Debug, Clone)]
pub struct WatcherConfig {
    /// Debounce interval for batching events
    pub debounce_interval: Duration,
    /// Patterns to ignore (gitignore-style)
    pub ignore_patterns: Vec<String>,
    /// File extensions to watch (None = all)
    pub watch_extensions: Option<Vec<String>>,
    /// Maximum depth for recursive watching
    pub max_depth: Option<usize>,
    /// Enable pattern detection on file changes
    pub detect_patterns: bool,
    /// Enable relationship tracking
    pub track_relationships: bool,
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            debounce_interval: Duration::from_millis(500),
            ignore_patterns: vec![
                "**/node_modules/**".to_string(),
                "**/target/**".to_string(),
                "**/.git/**".to_string(),
                "**/dist/**".to_string(),
                "**/build/**".to_string(),
                "**/*.lock".to_string(),
                "**/*.log".to_string(),
            ],
            watch_extensions: Some(vec![
                "rs".to_string(),
                "ts".to_string(),
                "tsx".to_string(),
                "js".to_string(),
                "jsx".to_string(),
                "py".to_string(),
                "go".to_string(),
                "java".to_string(),
                "kt".to_string(),
                "swift".to_string(),
                "cs".to_string(),
                "cpp".to_string(),
                "c".to_string(),
                "h".to_string(),
                "hpp".to_string(),
                "rb".to_string(),
                "php".to_string(),
            ]),
            max_depth: None,
            detect_patterns: true,
            track_relationships: true,
        }
    }
}

// ============================================================================
// EDIT SESSION
// ============================================================================

/// Tracks files being edited in a session
#[derive(Debug)]
struct EditSession {
    /// Files modified in this session
    files: HashSet<PathBuf>,
    /// When the session started (for analytics/debugging)
    #[allow(dead_code)]
    started_at: DateTime<Utc>,
    /// When the last edit occurred
    last_edit_at: DateTime<Utc>,
}

impl EditSession {
    fn new() -> Self {
        let now = Utc::now();
        Self {
            files: HashSet::new(),
            started_at: now,
            last_edit_at: now,
        }
    }

    fn add_file(&mut self, path: PathBuf) {
        self.files.insert(path);
        self.last_edit_at = Utc::now();
    }

    fn is_expired(&self, timeout: Duration) -> bool {
        let elapsed = Utc::now()
            .signed_duration_since(self.last_edit_at)
            .to_std()
            .unwrap_or(Duration::ZERO);
        elapsed > timeout
    }

    fn files_list(&self) -> Vec<PathBuf> {
        self.files.iter().cloned().collect()
    }
}

// ============================================================================
// CODEBASE WATCHER
// ============================================================================

/// Watches a codebase for file changes
pub struct CodebaseWatcher {
    /// Relationship tracker
    tracker: Arc<RwLock<RelationshipTracker>>,
    /// Pattern detector
    detector: Arc<RwLock<PatternDetector>>,
    /// Configuration
    config: WatcherConfig,
    /// Currently watched paths
    watched_paths: Arc<RwLock<HashSet<PathBuf>>>,
    /// Shutdown signal sender
    shutdown_tx: Option<broadcast::Sender<()>>,
    /// Flag to signal watcher thread to stop
    running: Arc<AtomicBool>,
}

impl CodebaseWatcher {
    /// Create a new codebase watcher
    pub fn new(
        tracker: Arc<RwLock<RelationshipTracker>>,
        detector: Arc<RwLock<PatternDetector>>,
    ) -> Self {
        Self::with_config(tracker, detector, WatcherConfig::default())
    }

    /// Create a new codebase watcher with custom config
    pub fn with_config(
        tracker: Arc<RwLock<RelationshipTracker>>,
        detector: Arc<RwLock<PatternDetector>>,
        config: WatcherConfig,
    ) -> Self {
        Self {
            tracker,
            detector,
            config,
            watched_paths: Arc::new(RwLock::new(HashSet::new())),
            shutdown_tx: None,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Start watching a directory
    pub async fn watch(&mut self, path: &Path) -> Result<()> {
        let path = path.canonicalize()?;

        // Check if already watching
        {
            let watched = self.watched_paths.read().await;
            if watched.contains(&path) {
                return Err(WatcherError::AlreadyWatching(path));
            }
        }

        // Add to watched paths
        self.watched_paths.write().await.insert(path.clone());

        // Create shutdown channel
        let (shutdown_tx, mut shutdown_rx) = broadcast::channel::<()>(1);
        self.shutdown_tx = Some(shutdown_tx);

        // Create event channel
        let (event_tx, mut event_rx) = mpsc::channel::<FileEvent>(100);

        // Clone for move into watcher thread
        let config = self.config.clone();
        let watch_path = path.clone();

        // Set running flag to true and clone for thread
        self.running.store(true, Ordering::SeqCst);
        let running = Arc::clone(&self.running);

        // Spawn watcher thread
        let event_tx_clone = event_tx.clone();
        std::thread::spawn(move || {
            let config_notify = Config::default().with_poll_interval(config.debounce_interval);

            let tx = event_tx_clone.clone();
            let mut watcher = match RecommendedWatcher::new(
                move |res: std::result::Result<Event, notify::Error>| {
                    if let Ok(event) = res {
                        let file_event = FileEvent {
                            kind: event.kind.into(),
                            paths: event.paths,
                            timestamp: Utc::now(),
                        };
                        let _ = tx.blocking_send(file_event);
                    }
                },
                config_notify,
            ) {
                Ok(w) => w,
                Err(e) => {
                    eprintln!("Failed to create watcher: {}", e);
                    return;
                }
            };

            if let Err(e) = watcher.watch(&watch_path, RecursiveMode::Recursive) {
                eprintln!("Failed to watch path: {}", e);
                return;
            }

            // Keep thread alive until shutdown signal
            while running.load(Ordering::SeqCst) {
                std::thread::sleep(Duration::from_millis(100));
            }
        });

        // Clone for move into handler task
        let tracker = Arc::clone(&self.tracker);
        let detector = Arc::clone(&self.detector);
        let config = self.config.clone();

        // Spawn event handler task
        tokio::spawn(async move {
            let mut session = EditSession::new();
            let session_timeout = Duration::from_secs(60 * 30); // 30 minutes

            loop {
                tokio::select! {
                    Some(event) = event_rx.recv() => {
                        // Check session expiry
                        if session.is_expired(session_timeout) {
                            // Record co-edits from expired session
                            if session.files.len() >= 2 {
                                let files = session.files_list();
                                if let Ok(mut tracker) = tracker.try_write() {
                                    let _ = tracker.record_coedit(&files);
                                }
                            }
                            session = EditSession::new();
                        }

                        // Process event
                        for path in &event.paths {
                            if Self::should_process(path, &config) {
                                match event.kind {
                                    FileEventKind::Modified | FileEventKind::Created => {
                                        // Track in session
                                        if config.track_relationships {
                                            session.add_file(path.clone());
                                        }

                                        // Detect patterns if enabled
                                        if config.detect_patterns {
                                            if let Ok(content) = std::fs::read_to_string(path) {
                                                let language = Self::detect_language(path);
                                                if let Ok(detector) = detector.try_read() {
                                                    let _ = detector.detect_patterns(&content, &language);
                                                }
                                            }
                                        }
                                    }
                                    FileEventKind::Deleted => {
                                        // File was deleted, remove from session
                                        session.files.remove(path);
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        // Finalize session before shutdown
                        if session.files.len() >= 2 {
                            let files = session.files_list();
                            if let Ok(mut tracker) = tracker.try_write() {
                                let _ = tracker.record_coedit(&files);
                            }
                        }
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    /// Stop watching a directory
    pub async fn unwatch(&mut self, path: &Path) -> Result<()> {
        let path = path.canonicalize()?;

        let mut watched = self.watched_paths.write().await;
        if !watched.remove(&path) {
            return Err(WatcherError::NotWatching(path));
        }

        // If no more paths being watched, send shutdown signals
        if watched.is_empty() {
            // Signal watcher thread to exit
            self.running.store(false, Ordering::SeqCst);

            // Signal async task to exit
            if let Some(tx) = &self.shutdown_tx {
                let _ = tx.send(());
            }
        }

        Ok(())
    }

    /// Stop watching all directories
    pub async fn stop(&mut self) -> Result<()> {
        self.watched_paths.write().await.clear();

        // Signal watcher thread to exit
        self.running.store(false, Ordering::SeqCst);

        // Signal async task to exit
        if let Some(tx) = &self.shutdown_tx {
            let _ = tx.send(());
        }

        Ok(())
    }

    /// Check if a path should be processed based on config
    fn should_process(path: &Path, config: &WatcherConfig) -> bool {
        let path_str = path.to_string_lossy();

        // Check ignore patterns
        for pattern in &config.ignore_patterns {
            // Simple glob matching (basic implementation)
            if Self::glob_match(&path_str, pattern) {
                return false;
            }
        }

        // Check extensions
        if let Some(ref extensions) = config.watch_extensions {
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if !extensions.iter().any(|e| e.to_lowercase() == ext_str) {
                    return false;
                }
            } else {
                return false; // No extension and we're filtering by extension
            }
        }

        true
    }

    /// Simple glob pattern matching
    fn glob_match(path: &str, pattern: &str) -> bool {
        // Handle ** (match any path)
        if pattern.contains("**") {
            let parts: Vec<_> = pattern.split("**").collect();
            if parts.len() == 2 {
                let prefix = parts[0].trim_end_matches('/');
                let suffix = parts[1].trim_start_matches('/');

                let prefix_match = prefix.is_empty() || path.starts_with(prefix);

                // Handle suffix with wildcards like *.lock
                let suffix_match = if suffix.is_empty() {
                    true
                } else if suffix.starts_with('*') {
                    // Pattern like *.lock - match the extension
                    let ext_pattern = suffix.trim_start_matches('*');
                    path.ends_with(ext_pattern)
                } else {
                    // Exact suffix match
                    path.ends_with(suffix) || path.contains(&format!("/{}", suffix))
                };

                return prefix_match && suffix_match;
            }
        }

        // Handle * (match single component)
        if pattern.contains('*') {
            let pattern = pattern.replace('*', "");
            return path.contains(&pattern);
        }

        // Direct match
        path.contains(pattern)
    }

    /// Detect language from file extension
    fn detect_language(path: &Path) -> String {
        path.extension()
            .map(|e| {
                let ext = e.to_string_lossy().to_lowercase();
                match ext.as_str() {
                    "rs" => "rust",
                    "ts" | "tsx" => "typescript",
                    "js" | "jsx" => "javascript",
                    "py" => "python",
                    "go" => "go",
                    "java" => "java",
                    "kt" | "kts" => "kotlin",
                    "swift" => "swift",
                    "cs" => "csharp",
                    "cpp" | "cc" | "cxx" | "c" | "h" | "hpp" => "cpp",
                    "rb" => "ruby",
                    "php" => "php",
                    _ => "unknown",
                }
                .to_string()
            })
            .unwrap_or_else(|| "unknown".to_string())
    }

    /// Get currently watched paths
    pub async fn get_watched_paths(&self) -> Vec<PathBuf> {
        self.watched_paths.read().await.iter().cloned().collect()
    }

    /// Check if a path is being watched
    pub async fn is_watching(&self, path: &Path) -> bool {
        let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        self.watched_paths.read().await.contains(&path)
    }

    /// Get the current configuration
    pub fn config(&self) -> &WatcherConfig {
        &self.config
    }

    /// Update the configuration
    pub fn set_config(&mut self, config: WatcherConfig) {
        self.config = config;
    }
}

impl Drop for CodebaseWatcher {
    fn drop(&mut self) {
        // Signal watcher thread to exit
        self.running.store(false, Ordering::SeqCst);

        // Signal async task to exit
        if let Some(tx) = &self.shutdown_tx {
            let _ = tx.send(());
        }
    }
}

// ============================================================================
// MANUAL EVENT HANDLER (for non-async contexts)
// ============================================================================

/// Handles file events manually (for use without the async watcher)
pub struct ManualEventHandler {
    tracker: Arc<RwLock<RelationshipTracker>>,
    detector: Arc<RwLock<PatternDetector>>,
    session_files: HashSet<PathBuf>,
    config: WatcherConfig,
}

impl ManualEventHandler {
    /// Create a new manual event handler
    pub fn new(
        tracker: Arc<RwLock<RelationshipTracker>>,
        detector: Arc<RwLock<PatternDetector>>,
    ) -> Self {
        Self {
            tracker,
            detector,
            session_files: HashSet::new(),
            config: WatcherConfig::default(),
        }
    }

    /// Handle a file modification event
    pub async fn on_file_modified(&mut self, path: &Path) -> Result<()> {
        if !CodebaseWatcher::should_process(path, &self.config) {
            return Ok(());
        }

        // Add to session
        self.session_files.insert(path.to_path_buf());

        // Record co-edit if we have multiple files
        if self.session_files.len() >= 2 {
            let files: Vec<_> = self.session_files.iter().cloned().collect();
            let mut tracker = self.tracker.write().await;
            tracker.record_coedit(&files)?;
        }

        // Detect patterns
        if self.config.detect_patterns {
            if let Ok(content) = std::fs::read_to_string(path) {
                let language = CodebaseWatcher::detect_language(path);
                let detector = self.detector.read().await;
                let _ = detector.detect_patterns(&content, &language);
            }
        }

        Ok(())
    }

    /// Handle a file creation event
    pub async fn on_file_created(&mut self, path: &Path) -> Result<()> {
        self.on_file_modified(path).await
    }

    /// Handle a file deletion event
    pub async fn on_file_deleted(&mut self, path: &Path) -> Result<()> {
        self.session_files.remove(path);
        Ok(())
    }

    /// Clear the current session
    pub fn clear_session(&mut self) {
        self.session_files.clear();
    }

    /// Finalize the current session
    pub async fn finalize_session(&mut self) -> Result<()> {
        if self.session_files.len() >= 2 {
            let files: Vec<_> = self.session_files.iter().cloned().collect();
            let mut tracker = self.tracker.write().await;
            tracker.record_coedit(&files)?;
        }
        self.session_files.clear();
        Ok(())
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_match() {
        // Match any path with pattern
        assert!(CodebaseWatcher::glob_match(
            "/project/node_modules/foo/bar.js",
            "**/node_modules/**"
        ));
        assert!(CodebaseWatcher::glob_match(
            "/project/target/debug/main",
            "**/target/**"
        ));
        assert!(CodebaseWatcher::glob_match(
            "/project/.git/config",
            "**/.git/**"
        ));

        // Extension matching
        assert!(CodebaseWatcher::glob_match(
            "/project/Cargo.lock",
            "**/*.lock"
        ));

        // Non-matches
        assert!(!CodebaseWatcher::glob_match(
            "/project/src/main.rs",
            "**/node_modules/**"
        ));
    }

    #[test]
    fn test_should_process() {
        let config = WatcherConfig::default();

        // Should process source files
        assert!(CodebaseWatcher::should_process(
            Path::new("/project/src/main.rs"),
            &config
        ));
        assert!(CodebaseWatcher::should_process(
            Path::new("/project/src/app.tsx"),
            &config
        ));

        // Should not process node_modules
        assert!(!CodebaseWatcher::should_process(
            Path::new("/project/node_modules/foo/index.js"),
            &config
        ));

        // Should not process target
        assert!(!CodebaseWatcher::should_process(
            Path::new("/project/target/debug/main"),
            &config
        ));

        // Should not process lock files
        assert!(!CodebaseWatcher::should_process(
            Path::new("/project/Cargo.lock"),
            &config
        ));
    }

    #[test]
    fn test_detect_language() {
        assert_eq!(
            CodebaseWatcher::detect_language(Path::new("main.rs")),
            "rust"
        );
        assert_eq!(
            CodebaseWatcher::detect_language(Path::new("app.tsx")),
            "typescript"
        );
        assert_eq!(
            CodebaseWatcher::detect_language(Path::new("script.js")),
            "javascript"
        );
        assert_eq!(
            CodebaseWatcher::detect_language(Path::new("main.py")),
            "python"
        );
        assert_eq!(CodebaseWatcher::detect_language(Path::new("main.go")), "go");
    }

    #[test]
    fn test_edit_session() {
        let mut session = EditSession::new();

        session.add_file(PathBuf::from("a.rs"));
        session.add_file(PathBuf::from("b.rs"));

        assert_eq!(session.files.len(), 2);
        assert!(!session.is_expired(Duration::from_secs(60)));
    }

    #[test]
    fn test_watcher_config_default() {
        let config = WatcherConfig::default();

        assert!(!config.ignore_patterns.is_empty());
        assert!(config.watch_extensions.is_some());
        assert!(config.detect_patterns);
        assert!(config.track_relationships);
    }
}
