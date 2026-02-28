# SlowHTTPTest GUI

A graphical user interface for [slowhttptest](https://github.com/shekyan/slowhttptest) built with Rust and [egui](https://github.com/emilk/egui).

The slowhttptest binary is **compiled and embedded** into the GUI at build time, so the resulting executable is fully self-contained — no separate installation of slowhttptest is required.

## Features

- **Self-contained** – the slowhttptest executable is embedded; just run the GUI
- **All test modes** – Slow Headers (Slowloris), Slow Body (R-U-Dead-Yet), Range Attack (Apache Killer), Slow Read
- **Full parameter support** – every CLI flag exposed in a clean form
- **Command preview** – see the exact command before running
- **Live output** – streaming terminal output inside the GUI
- **Cross-platform** – runs on Linux and macOS (Windows requires an external slowhttptest binary)

## Download pre-built binaries

Pre-built binaries are available on the [Releases page](../../releases) for:

| Platform | File |
|---|---|
| Linux x86_64 | `slowhttptest-gui-linux-x86_64.tar.gz` |
| macOS x86_64 | `slowhttptest-gui-macos-x86_64.tar.gz` |
| macOS arm64 (Apple Silicon) | `slowhttptest-gui-macos-arm64.tar.gz` |

You can also download **CI artifacts** from any [workflow run](../../actions/workflows/build-gui.yml) without waiting for a tagged release.

## Build from source

```bash
# Prerequisites:
#   - Rust toolchain (https://rustup.rs)
#   - C++ compiler (g++ or clang++)
#   - OpenSSL development headers (libssl-dev / openssl-devel)
cd gui
cargo build --release
# Binary is at: target/release/slowhttptest-gui
```

The build script automatically compiles the C++ slowhttptest code and embeds the resulting binary.

### Linux build dependencies

```bash
sudo apt-get install -y build-essential libssl-dev \
  libgl1-mesa-dev libxcb-render0-dev libxcb-shape0-dev \
  libxcb-xfixes0-dev libxkbcommon-dev libfontconfig1-dev
```

### macOS build dependencies

```bash
brew install openssl
```

## Usage

1. Launch `slowhttptest-gui`
2. Select the **test mode**
3. Enter the **target URL**
4. Adjust connection and timing parameters as needed
5. Expand **HTTP Options**, **Proxy** or **Reporting** for advanced settings
6. Click **▶ Run Test** – the output panel shows live results

> **Tip:** You can override the embedded binary by specifying a custom binary path in the Target section.

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
