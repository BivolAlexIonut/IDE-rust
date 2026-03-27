//! Domain model: application state and deferred UI actions.
//!
//! [`MyIDE`] is the single source of truth for the GUI. It intentionally stays **UI-agnostic**
//! in spirit: no egui types here, only data the views (`app.rs`) read and mutate. This
//! separation is a common pattern in immediate-mode GUIs: state struct + imperative draw loop.

use std::path::PathBuf;

/// Actions that may need confirmation or batching (reserved for future dialogs).
#[derive(Debug)]
pub enum PendingAction {
    None,
    Save,
    Delete(PathBuf),
}

/// Primary application state for the Rust mini-IDE.
///
/// Fields are `pub` for straightforward access from `impl eframe::App` and `impl MyIDE`
/// blocks; a larger codebase might use accessor methods or split into sub-structs
/// (`EditorState`, `TerminalState`).
pub struct MyIDE {
    pub code_buffer: String,
    pub files: Vec<PathBuf>,
    /// Working directory for the file tree and for spawned shell commands.
    pub current_dir: PathBuf,
    /// Directory where the app was started; used as a safe “home base” for `cd` and reset.
    pub project_root: PathBuf,
    pub selected_file: Option<PathBuf>,
    pub show_confirmation: bool,
    pub pending_action: PendingAction,
    pub show_new_file_dialog: bool,
    pub new_file_name: String,
    pub search_query: String,

    /// Current line typed in the integrated terminal (not yet executed).
    pub terminal_input: String,
    /// Scrollback + output of executed commands.
    pub terminal_output: String,
    /// Ring buffer of recent commands (used for ↑ / ↓ history in the terminal field).
    pub terminal_command_history: Vec<String>,
    /// When `Some(i)`, the user is browsing [`terminal_command_history`][i].
    pub terminal_history_browse: Option<usize>,

    /// Last user-facing status line (save result, hints). Cleared when overwritten.
    pub status_message: String,
    /// Modal “About this project” window (presentation / coursework).
    pub show_about: bool,
}

impl Default for MyIDE {
    fn default() -> Self {
        let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self {
            code_buffer: String::new(),
            files: Vec::new(),
            current_dir: root.clone(),
            project_root: root,
            selected_file: None,
            show_confirmation: false,
            pending_action: PendingAction::None,
            show_new_file_dialog: false,
            new_file_name: "new_file.txt".to_string(),
            search_query: String::new(),
            terminal_input: String::new(),
            terminal_output: "Terminal ready...\n".to_string(),
            terminal_command_history: Vec::new(),
            terminal_history_browse: None,
            status_message: String::new(),
            show_about: false,
        }
    }
}
