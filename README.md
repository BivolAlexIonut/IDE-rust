# 🚀 Professional Rust IDE (Prototype)

A lightweight, modular, and high-performance Integrated Development Environment (IDE) built from scratch using **Rust** and the **egui** framework. This project focuses on a clean "Separation of Concerns" architecture, making it easy to scale and maintain.



## 🛠 Features

- **Modular Architecture**: Logical separation between Data Models, File System operations, and UI Rendering.
- **Smart File Explorer**: 
  - Real-time search and filtering.
  - Folder navigation (Enter directories, move to parent).
  - File-type visual cues (Icons and colors for `.rs`, `.cpp`, `.txt`).
- **Integrated Bash Terminal**: 
  - VS Code-inspired terminal at the bottom.
  - Executes commands via **Bash** (optimized for WSL on Windows).
  - Auto-scrolling output history with a dedicated command prompt.
- **Professional Editor**:
  - Multi-line text editing with a dark, developer-focused theme.
  - Integrated Save and Delete functionality.
  - Breadcrumb navigation showing the current path.
- **Safety First**: Modal dialogs for new file creation and external file modification warnings.

## 🏗 Project Structure

The codebase is organized into four main modules:

1.  `main.rs`: Application entry point. Handles window configuration and high-level initialization.
2.  `models.rs`: Defines the state and data structures (the "Source of Truth").
3.  `filesystem.rs`: Contains the business logic for I/O operations and shell command execution.
4.  `app.rs`: The UI engine. Manages layout, panels, styling, and the main event loop.

## 🚀 Getting Started

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (latest stable version).
- [WSL/Bash](https://learn.microsoft.com/en-us/windows/wsl/install) (for the integrated terminal functionality on Windows).

### Installation
1. Clone the repository:
   ```bash
   git clone [https://github.com/your-username/your-repo-name.git](https://github.com/your-username/your-repo-name.git)
