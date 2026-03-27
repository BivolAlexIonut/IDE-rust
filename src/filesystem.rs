//! File system operations, shell integration, and terminal command handling.
//!
//! This module implements [`MyIDE`] methods that touch the OS: directory listing,
//! saving buffers, and running external processes. Built-in commands (`clear`, `cd`, `help`)
//! are parsed **before** spawning a shell so the IDE can stay in sync with the UI (e.g. `cd`
//! updates [`MyIDE::current_dir`] and refreshes the tree).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use crate::models::{MyIDE, PendingAction};

static BASH_PATH: OnceLock<PathBuf> = OnceLock::new();

/// Loads a file as **UTF-8**. The editor uses Rust `String` (UTF-8 only); invalid bytes
/// return an error instead of mis-decoding the file.
pub fn read_utf8_file(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|e| format!("Could not read file: {e}"))?;
    String::from_utf8(bytes).map_err(|e| {
        format!(
            "File is not valid UTF-8 (byte offset {}). Save as UTF-8 or convert encoding.",
            e.utf8_error().valid_up_to()
        )
    })
}

const HELP_TEXT: &str = "\
Built-in commands:\n\
  clear / cls     Clear the terminal buffer\n\
  cd [path]       Change working directory (updates the sidebar)\n\
  cd ..           Parent directory\n\
  cd              User home (or project root if home is unavailable)\n\
  help / ?        Show this message\n\
\n\
Shortcuts: Ctrl+S — save the open file\n\
Terminal: ↑ / ↓ — command history\n\
";

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
}

/// Resolves a path for `cd`: absolute, relative to [`MyIDE::current_dir`], or `~` / `~/…`.
fn resolve_path(base: &Path, arg: &str) -> Result<PathBuf, String> {
    let arg = arg.trim();
    if arg.is_empty() {
        return Err("empty path".to_string());
    }
    if arg == "~" {
        return home_dir().ok_or_else(|| "HOME / USERPROFILE unavailable.".to_string());
    }
    if let Some(rest) = arg.strip_prefix("~/") {
        let h = home_dir().ok_or_else(|| "HOME unavailable.".to_string())?;
        return Ok(h.join(rest));
    }
    let p = PathBuf::from(arg);
    let joined = if p.is_absolute() { p } else { base.join(arg) };
    fs::canonicalize(&joined).map_err(|e| e.to_string())
}

enum Builtin {
    Clear,
    Help,
    Cd(String),
}

fn parse_builtin(line: &str) -> Option<Builtin> {
    let line = line.trim();
    let mut it = line.split_whitespace();
    let first = it.next()?.to_lowercase();
    match first.as_str() {
        "clear" | "cls" => {
            if it.next().is_none() {
                Some(Builtin::Clear)
            } else {
                None
            }
        }
        "help" | "?" => {
            if it.next().is_none() {
                Some(Builtin::Help)
            } else {
                None
            }
        }
        "cd" => {
            let rest: String = it.collect::<Vec<_>>().join(" ");
            Some(Builtin::Cd(rest))
        }
        _ => None,
    }
}

/// Locate `bash` once: PATH, then common Git for Windows install paths.
fn bash_executable() -> PathBuf {
    BASH_PATH
        .get_or_init(|| {
            if Command::new("bash")
                .arg("--version")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
            {
                return PathBuf::from("bash");
            }
            #[cfg(windows)]
            {
                for p in [
                    r"C:\Program Files\Git\bin\bash.exe",
                    r"C:\Program Files (x86)\Git\bin\bash.exe",
                ] {
                    let path = Path::new(p);
                    if path.is_file() {
                        return path.to_path_buf();
                    }
                }
            }
            PathBuf::from("bash")
        })
        .clone()
}

impl MyIDE {
    fn append_command_output(&mut self, out: &std::process::Output) {
        let stdout = String::from_utf8_lossy(&out.stdout);
        let stderr = String::from_utf8_lossy(&out.stderr);
        if !stdout.is_empty() {
            self.terminal_output.push_str(&stdout);
            if !stdout.ends_with('\n') {
                self.terminal_output.push('\n');
            }
        }
        if !stderr.is_empty() {
            self.terminal_output.push_str(&stderr);
            if !stderr.ends_with('\n') {
                self.terminal_output.push('\n');
            }
        }
        if !out.status.success() {
            self.terminal_output.push_str(&format!(
                "[exit code {}]\n",
                out.status.code().unwrap_or(-1)
            ));
        }
    }

    fn push_terminal_history(&mut self, cmd: &str) {
        const MAX: usize = 50;
        if self
            .terminal_command_history
            .last()
            .map_or(true, |s| s != cmd)
        {
            self.terminal_command_history.push(cmd.to_string());
            if self.terminal_command_history.len() > MAX {
                self.terminal_command_history.remove(0);
            }
        }
    }

    fn apply_cd(&mut self, path_arg: &str) -> Result<(), String> {
        let arg = path_arg.trim();
        let target = if arg.is_empty() {
            home_dir().unwrap_or_else(|| self.project_root.clone())
        } else {
            resolve_path(&self.current_dir, arg)?
        };
        if !target.is_dir() {
            return Err(format!("not a directory: {}", target.display()));
        }
        self.current_dir = target;
        self.refresh_files();
        Ok(())
    }

    fn run_shell_command(&mut self, cmd_text: &str) {
        let bash = bash_executable();
        let output = Command::new(&bash)
            .args(["-c", cmd_text])
            .current_dir(&self.current_dir)
            .output();

        match output {
            Ok(out) => {
                self.append_command_output(&out);
            }
            Err(e) => {
                #[cfg(windows)]
                {
                    match Command::new("powershell.exe")
                        .args(["-NoProfile", "-NonInteractive", "-Command", cmd_text])
                        .current_dir(&self.current_dir)
                        .output()
                    {
                        Ok(out) => {
                            self.append_command_output(&out);
                        }
                        Err(e2) => {
                            self.terminal_output.push_str(&format!(
                                "Could not start Bash ({}). PowerShell failed: {}\n\
                                 Install Git for Windows for Bash, or use PowerShell syntax.\n",
                                e, e2
                            ));
                        }
                    }
                }
                #[cfg(not(windows))]
                {
                    self.terminal_output.push_str(&format!(
                        "Could not start Bash ({}): {}\n",
                        bash.display(),
                        e
                    ));
                }
            }
        }
    }

    /// Executes the text in [`MyIDE::terminal_input`]: built-ins first, then a real shell.
    pub fn run_terminal_command(&mut self) {
        let cmd_text = self.terminal_input.trim().to_string();
        if cmd_text.is_empty() {
            return;
        }

        self.push_terminal_history(&cmd_text);
        self.terminal_history_browse = None;

        if let Some(built) = parse_builtin(&cmd_text) {
            match built {
                Builtin::Clear => {
                    self.terminal_output.clear();
                    self.terminal_output.push_str("Terminal ready.\n");
                }
                Builtin::Help => {
                    self.terminal_output
                        .push_str(&format!("\n$ {}\n", cmd_text));
                    self.terminal_output.push_str(HELP_TEXT);
                }
                Builtin::Cd(path_arg) => {
                    self.terminal_output
                        .push_str(&format!("\n$ {}\n", cmd_text));
                    match self.apply_cd(&path_arg) {
                        Ok(()) => {
                            self.terminal_output
                                .push_str(&format!("{}\n", self.current_dir.display()));
                        }
                        Err(e) => {
                            self.terminal_output.push_str(&format!("cd: {}\n", e));
                        }
                    }
                }
            }
        } else {
            self.terminal_output
                .push_str(&format!("\n$ {}\n", cmd_text));
            self.run_shell_command(&cmd_text);
        }
        self.terminal_input.clear();
    }

    /// Injects a full command line (used by Cargo shortcut buttons).
    pub fn run_terminal_line(&mut self, line: impl Into<String>) {
        self.terminal_input = line.into();
        self.run_terminal_command();
    }

    /// Reloads [`MyIDE::files`] from disk and sorts directories first, then names (case-insensitive).
    pub fn refresh_files(&mut self) {
        if let Ok(entries) = fs::read_dir(&self.current_dir) {
            let mut paths: Vec<PathBuf> = entries
                .filter_map(|res| res.ok())
                .map(|e| e.path())
                .collect();
            paths.sort_by(|a, b| {
                let da = a.is_dir();
                let db = b.is_dir();
                match (da, db) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => {
                        let na = a.file_name().unwrap_or_default();
                        let nb = b.file_name().unwrap_or_default();
                        na.to_string_lossy()
                            .to_lowercase()
                            .cmp(&nb.to_string_lossy().to_lowercase())
                    }
                }
            });
            self.files = paths;
        }
    }

    /// Returns `true` if `path` lies outside the tree rooted at [`MyIDE::project_root`].
    pub fn is_external(&self, path: &Path) -> bool {
        !path.starts_with(&self.project_root)
    }

    pub fn create_file(&mut self) -> std::io::Result<()> {
        if self.new_file_name.is_empty() {
            return Ok(());
        }
        let mut path = self.current_dir.clone();
        path.push(&self.new_file_name);
        fs::File::create(&path)?;
        self.refresh_files();
        self.show_new_file_dialog = false;
        self.new_file_name = "new_file.txt".to_string();
        Ok(())
    }

    pub fn delete_file(&mut self, path: PathBuf) -> std::io::Result<()> {
        if path.is_dir() {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }
        self.refresh_files();
        self.selected_file = None;
        self.code_buffer.clear();
        Ok(())
    }

    /// Writes the buffer as **UTF-8** bytes (standard for Rust source and text).
    pub fn save_current_file(&self) -> std::io::Result<()> {
        if let Some(path) = &self.selected_file {
            fs::write(path, self.code_buffer.as_bytes())?;
        }
        Ok(())
    }

    /// Snaps the working directory back to the project root (startup folder).
    pub fn reset_to_project_root(&mut self) {
        self.current_dir = self.project_root.clone();
        self.refresh_files();
    }

    pub fn execute_pending_action(&mut self) {
        match std::mem::replace(&mut self.pending_action, PendingAction::None) {
            PendingAction::Save => {
                let _ = self.save_current_file();
            }
            PendingAction::Delete(p) => {
                let _ = self.delete_file(p);
            }
            PendingAction::None => {}
        }
    }
}
