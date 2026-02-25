# SlowHTTPTest GUI

A graphical user interface for [slowhttptest](https://github.com/shekyan/slowhttptest) built with Rust and [egui](https://github.com/emilk/egui).

## Features

- **All test modes** – Slow Headers (Slowloris), Slow Body (R-U-Dead-Yet), Range Attack (Apache Killer), Slow Read
- **Full parameter support** – every CLI flag exposed in a clean form
- **Command preview** – see the exact command before running
- **Live output** – streaming terminal output inside the GUI
- **Cross-platform** – runs on Linux, Windows and macOS

## Requirements

- [`slowhttptest`](https://github.com/shekyan/slowhttptest) must be installed and available in your `PATH`  
  (or specify a custom binary path in the GUI)

## Download pre-built binaries

Pre-built binaries are available on the [Releases page](../../releases) for:

| Platform | File |
|---|---|
| Linux x86_64 | `slowhttptest-gui-linux-x86_64.tar.gz` |
| Windows x86_64 | `slowhttptest-gui-windows-x86_64.zip` |
| macOS x86_64 | `slowhttptest-gui-macos-x86_64.tar.gz` |
| macOS arm64 (Apple Silicon) | `slowhttptest-gui-macos-arm64.tar.gz` |

You can also download **CI artifacts** from any [workflow run](../../actions/workflows/build-gui.yml) without waiting for a tagged release.

## Build from source

```bash
# Prerequisites: Rust toolchain (https://rustup.rs)
cd gui
cargo build --release
# Binary is at: target/release/slowhttptest-gui
```

### Linux build dependencies

```bash
sudo apt-get install -y libgl1-mesa-dev libxcb-render0-dev libxcb-shape0-dev \
  libxcb-xfixes0-dev libxkbcommon-dev libfontconfig1-dev
```

## Usage

1. Launch `slowhttptest-gui`
2. Select the **test mode**
3. Enter the **target URL**
4. Adjust connection and timing parameters as needed
5. Expand **HTTP Options**, **Proxy** or **Reporting** for advanced settings
6. Click **▶ Run Test** – the output panel shows live results

## Parameters reference

| Section | Parameters |
|---|---|
| Test Mode | `-H` `-B` `-R` `-X` |
| Connection | `-c` `-r` `-l` `-i` `-s` `-x` |
| HTTP | `-t` `-f` `-m` `-j` `-1` |
| Proxy | `-d` `-e` `-p` |
| Reporting | `-g` `-o` `-v` |
| Range Attack | `-a` `-b` |
| Slow Read | `-k` `-n` `-w` `-y` `-z` |
