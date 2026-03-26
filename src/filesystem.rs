use std::fs;
use std::process::Command;
use std::path::{Path, PathBuf};
use crate::models::{MyIDE, PendingAction};

impl MyIDE {
    // Refresh the file explorer list by reading the current directory
    pub fn refresh_files(&mut self) {
        if let Ok(entries) = fs::read_dir(&self.current_dir) {
            self.files = entries
                .filter_map(|res| res.ok())
                .map(|e| e.path())
                .collect();
        }
    }

    // Check if a file is located outside the initial project root for safety warnings
    pub fn is_external(&self, path: &Path) -> bool {
        !path.starts_with(&self.project_root)
    }

    // Create a new file on disk using the name provided in the UI dialog
    pub fn create_file(&mut self) -> std::io::Result<()> {
        if self.new_file_name.is_empty() { return Ok(()); }
        let mut path = self.current_dir.clone();
        path.push(&self.new_file_name);
        fs::File::create(&path)?;
        self.refresh_files();
        self.show_new_file_dialog = false;
        self.new_file_name = "new_file.txt".to_string();
        Ok(())
    }

    // Delete a file or directory recursively and clear the editor if it was open
    pub fn delete_file(&mut self, path: PathBuf) -> std::io::Result<()> {
        if path.is_dir() { fs::remove_dir_all(path)?; }
        else { fs::remove_file(path)?; }
        self.refresh_files();
        self.selected_file = None;
        self.code_buffer.clear();
        Ok(())
    }

    // Write the current editor content back to the physical file on disk
    pub fn save_current_file(&self) -> std::io::Result<()> {
        if let Some(path) = &self.selected_file {
            fs::write(path, &self.code_buffer)?;
        }
        Ok(())
    }

    // Execute shell commands using Bash.
    // We use "cmd /C bash" to ensure Windows correctly handles the environment relay.
    pub fn run_terminal_command(&mut self) {
        let cmd_text = self.terminal_input.clone();
        if cmd_text.is_empty() { return; }

        self.terminal_output.push_str(&format!("\n$ {}\n", cmd_text));

        // Using "cmd /C bash" is the most stable way to invoke the default Linux shell on Windows
        // without triggering the "execvpe" relay errors.
        let output = Command::new("cmd")
            .args(["/C", &format!("bash -c \"{}\"", cmd_text.replace("\"", "\\\""))])
            .current_dir(&self.current_dir)
            .output();

        match output {
            Ok(out) => {
                // Combine standard output and error output into the terminal history
                self.terminal_output.push_str(&String::from_utf8_lossy(&out.stdout));
                self.terminal_output.push_str(&String::from_utf8_lossy(&out.stderr));
            }
            Err(e) => self.terminal_output.push_str(&format!("System Error: Failed to invoke shell. ({})", e)),
        }
        self.terminal_input.clear();
    }

    // Handle deferred actions that required user confirmation via modal dialogs
    pub fn execute_pending_action(&mut self) {
        match std::mem::replace(&mut self.pending_action, PendingAction::None) {
            PendingAction::Save => { let _ = self.save_current_file(); }
            PendingAction::Delete(p) => { let _ = self.delete_file(p); }
            PendingAction::None => {}
        }
    }
}