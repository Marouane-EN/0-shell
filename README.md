# 0-shell

![Build Status](https://github.com/Marouane-EN/0-shell/actions/workflows/ci.yml/badge.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust Version](https://img.shields.io/badge/rust-1.75%2B-orange.svg)

**0-shell** is a lightweight, custom Unix-like shell implementation written in Rust.

The goal of this project is to explore low-level system programming concepts, process management, and the internal architecture of command-line interpreters. It aims to be compliant with basic POSIX standards while leveraging Rust's memory safety guarantees.

> **⚠️ Current Status:** *Active Development (Pre-Alpha)*.
> This shell is currently in the initial stages of construction.

## 🚀 Features Roadmap

This project is built iteratively. Below is the current implementation status:

- [ ] **The REPL** (Read-Eval-Print Loop) basic cycle
- [ ] **Command Parsing** (Tokenizing input)
- [ ] **Process Execution** (`fork` and `exec` equivalent in Rust)
- [ ] **Built-in Commands**
    - [ ] `cd` (Change Directory)
    - [ ] `exit` (Graceful shutdown)
    - [ ] `echo`
- [ ] **Signal Handling** (Ctrl+C, Ctrl+D)

## 🛠️ Installation & Usage

Ensure you have [Rust and Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) installed.

```bash
# Clone the repository
git clone [https://github.com/YOUR_USERNAME/0-shell.git](https://github.com/YOUR_USERNAME/0-shell.git)

# Navigate to the project directory
cd 0-shell

# Build and run the shell
cargo run --release
