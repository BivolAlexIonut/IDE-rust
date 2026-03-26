use std::path::PathBuf;

// Actions that require user confirmation before execution
pub enum PendingAction {
    None,
    Save,
    Delete(PathBuf),
}

// The main state of our Integrated Development Environment
pub struct MyIDE {
    pub code_buffer: String,
    pub files: Vec<PathBuf>,
    pub current_dir: PathBuf,
    pub project_root: PathBuf,
    pub selected_file: Option<PathBuf>,
    pub show_confirmation: bool,
    pub pending_action: PendingAction,
    pub show_new_file_dialog: bool,
    pub new_file_name: String,
    pub search_query: String,

    // Terminal state fields
    pub terminal_input: String,  // Current command being typed
    pub terminal_output: String, // History of terminal responses
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
        }
    }
}