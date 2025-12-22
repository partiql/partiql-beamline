# Installation and Setup

This chapter will guide you through installing PartiQL Beamline and setting up your development environment. 
PartiQL Beamline is written in Rust, so we'll cover both building from source and using pre-built binaries when available.

## Prerequisites

Before installing PartiQL Beamline, ensure you have the following prerequisites:

### Required

- **Rust Toolchain**: PartiQL Beamline requires Rust 1.70 or later
- **Git**: For cloning the repository
- **Command Line Access**: Terminal or command prompt

### Optional but Recommended

- **Text Editor**: For editing Ion scripts (VS Code, vim, emacs, etc.)
- **JSON/Ion Viewer**: Use [jq](https://jqlang.org/) and/or [ion-cli](https://github.com/amazon-ion/ion-cli) tools for 
examining generated data

## Installing Rust

If you don't have Rust installed, follow these steps:

### On macOS, Linux, or WSL

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### On Windows

1. Download and run [rustup-init.exe](https://rustup.rs/)
2. Follow the installation prompts
3. Restart your command prompt

### Verify Rust Installation

```bash
rustc --version
cargo --version
```

You should see version information for both `rustc` and `cargo`.

## Installing PartiQL Beamline

### Method 1: Building from Source (Recommended)

This is currently the primary method for installing PartiQL Beamline:

1. **Clone the Repository**
   ```bash
   git clone https://github.com/partiql/partiql-beamline.git
   cd partiql-beamline
   ```

2. **Build the Project**
   ```bash
   cargo build --release
   ```

   This will compile PartiQL Beamline in release mode, which provides better performance for data generation.

3. **Verify the Installation**
   ```bash
   ./target/release/beamline --version
   ```

   You should see version information for PartiQL Beamline.

4. **Optional: Add to PATH**
   
   For easier access, you can add the binary to your PATH or create a symlink:
   
   **On macOS/Linux:**
   ```bash
   # Option 1: Copy to a directory in your PATH
   sudo cp target/release/beamline /usr/local/bin/
   
   # Option 2: Create a symlink
   ln -s $(pwd)/target/release/beamline ~/.local/bin/beamline
   
   # Option 3: Add to your shell profile
   echo 'export PATH="'$(pwd)'/target/release:$PATH"' >> ~/.bashrc
   source ~/.bashrc
   ```
   
   **On Windows:**
   ```cmd
   # Add the target/release directory to your PATH environment variable
   # Or copy the .exe file to a directory already in your PATH
   ```

### Method 2: Using Cargo Install (Not available yet)

Once PartiQL Beamline is published to crates.io, you'll be able to install it directly:

```bash
# This will be available in the future
cargo install beamline
```

## Verifying Your Installation

Let's verify that PartiQL Beamline is installed correctly by running a few basic commands:

### 1. Check Version

```bash
beamline --version
```

### 2. View Help

```bash
beamline --help
```

You should see output similar to:

```
PartiQL Beamline CLI

Usage: beamline <COMMAND>

Commands:
  gen          Run the generator
  infer-shape  Run the script shape inference
  help         Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### 3. Test Data Generation

Let's run a simple test to ensure data generation works:

```bash
beamline gen data --help
```

This should display the help for the data generation command, confirming that the core functionality is available.

## Development Environment Setup

### Setting Up Your Workspace

Create a directory for your PartiQL Beamline projects:

```bash
mkdir ~/partiql-beamline-workspace
cd ~/partiql-beamline-workspace
```

### Editor Configuration

#### VS Code

If you're using VS Code, consider installing these extensions for better Ion support:

1. **Rust Analyzer**: For Rust syntax highlighting if you plan to contribute
2. **JSON**: For viewing generated JSON output
3. **Better TOML**: For configuration files

#### Vim/Neovim

Add Ion syntax highlighting by creating `~/.vim/syntax/ion.vim` or using existing Ion plugins.

### Shell Aliases (Optional)

For convenience, you might want to create shell aliases:

```bash
# Add to your ~/.bashrc, ~/.zshrc, or equivalent
alias pql-gen='beamline gen data'
alias pql-query='beamline query'
alias pql-shape='beamline infer-shape'
```

## Troubleshooting Installation

### Common Issues

#### Rust Version Too Old

**Error**: `error: package requires rustc 1.70 or newer`

**Solution**: Update Rust:
```bash
rustup update
```

#### Build Failures

**Error**: Compilation errors during `cargo build`

**Solutions**:
1. Ensure you have the latest Rust version
2. Clean and rebuild:
   ```bash
   cargo clean
   cargo build --release
   ```
3. Check for system-specific dependencies

#### Permission Issues

**Error**: Permission denied when copying to `/usr/local/bin`

**Solution**: Use `sudo` or choose a different installation location:
```bash
# Install to user directory instead
mkdir -p ~/.local/bin
cp target/release/beamline ~/.local/bin/
```

#### PATH Issues

**Error**: `command not found: beamline`

**Solution**: Verify the binary is in your PATH:
```bash
which beamline
echo $PATH
```

### Getting Help

If you encounter issues not covered here:

1. Check the [Troubleshooting](../reference/troubleshooting.md) section
2. Review the [GitHub Issues](https://github.com/partiql/partiql-beamline/issues)
3. Create a new issue with:
   - Your operating system
   - Rust version (`rustc --version`)
   - Complete error messages
   - Steps to reproduce

## Performance Considerations

### Release vs Debug Builds

Always use release builds for actual data generation:

```bash
# Debug build (slower, for development)
cargo build

# Release build (faster, for production use)
cargo build --release
```

Release builds can be 10-100x faster than debug builds for data generation tasks.

### System Resources

PartiQL Beamline is designed to be memory-efficient, but consider your system resources:

- **RAM**: 4GB minimum, 8GB+ recommended for large datasets
- **Storage**: Ensure adequate disk space for generated data
- **CPU**: Multi-core processors will benefit from parallel processing features

## Next Steps

Now that you have PartiQL Beamline installed and verified, you're ready to generate your first dataset!
In the next section, we'll walk through creating your first data generation script and producing some sample data.