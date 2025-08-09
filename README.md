# timeout-cli

`timeout-cli` runs a command with a specified time limit. If the command completes within the timeout period, it returns the command's exit code. If the timeout is exceeded, it terminates the command and returns exit code 124 (following standard timeout behavior).

## Installation

### From Source

```bash
git clone <repository-url>
cd timeout-cli
cargo build --release
```

The binary will be available at `target/release/timeout`.

### Using Cargo

```bash
cargo install timeout-cli
```

## Usage

```
timeout <SECONDS> <COMMAND> [ARGS]...
```

### Arguments

- `<SECONDS>` - Number of seconds to wait before timing out
- `<COMMAND>` - Command to execute  
- `[ARGS]...` - Arguments to pass to the command

### Exit Codes

- **0-255**: The exit code returned by the executed command (when it completes successfully within the timeout)
- **124**: Command timed out and was terminated
- **1**: Error executing the command (e.g., command not found)

## Examples

### Basic Usage

```bash
# Run echo with a 5-second timeout
timeout 5 echo "Hello World"
# Output: Hello World
# Exit code: 0
```

### Command with Arguments

```bash
# List directory contents with timeout
timeout 10 ls -la /home
```

### Command with Flags

```bash
# Run a command that might hang
timeout 30 curl -s https://httpbin.org/delay/5
```

### Timeout Scenarios

```bash
# This will timeout after 2 seconds
timeout 2 sleep 10
# Exit code: 124

# This will complete normally
timeout 10 sleep 2
# Exit code: 0
```

### Exit Code Forwarding

```bash
# Command that exits with specific code
timeout 5 sh -c "exit 42"
# Exit code: 42

# Non-existent command
timeout 5 nonexistent-command
# Exit code: 1 (with error message)
```

### Complex Commands

```bash
# Commands with multiple arguments and spaces
timeout 15 sh -c "echo 'Processing...'; sleep 3; echo 'Done!'"

# Pipeline commands (wrap in shell)
timeout 10 sh -c "ps aux | grep timeout"
```

## Features

- ✅ **Reliable timeout handling** - Commands are properly terminated when timeout is reached
- ✅ **Exit code forwarding** - Returns the actual exit code of successful commands
- ✅ **Standard behavior** - Follows Unix timeout command conventions (exit code 124 for timeouts)
- ✅ **Argument handling** - Supports commands with flags, multiple arguments, and spaces
- ✅ **Cross-platform** - Works on Linux, macOS, and Windows
- ✅ **Zero dependencies** - Minimal runtime footprint
- ✅ **Fast execution** - Commands that complete quickly don't wait for timeout duration

## Testing

Run the comprehensive test suite:

```bash
cargo test
```

The test suite includes 17+ integration tests covering:
- Basic command execution
- Timeout scenarios
- Exit code forwarding
- Error handling
- Edge cases (zero timeout, very long timeouts)
- Commands with various argument patterns

## Contributing

### Reporting Issues

Found a bug or have a feature request? Please [open an issue on GitHub](https://github.com/yaleman/timeout-cli/issues).

When reporting issues, please include:

- Your operating system and version
- The command you were trying to run
- Expected vs actual behavior
- Any error messages

### Development

```bash
# Clone the repository
git clone <repository-url>
cd timeout-cli

# Run tests
cargo test

# Build for development
cargo build

# Build optimized release
cargo build --release
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Comparison with System timeout

This tool aims to be compatible with the standard Unix `timeout` command:

| Feature | timeout-cli | GNU timeout |
|---------|-------------|-------------|
| Basic timeout functionality | ✅ | ✅ |
| Exit code 124 for timeouts | ✅ | ✅ |
| Exit code forwarding | ✅ | ✅ |
| Signal handling options | ⏳ | ✅ |
| Preserve exit status | ✅ | ✅ |
| Kill after additional time | ⏳ | ✅ |

## Acknowledgments

Built with [Rust](https://rust-lang.org/) and [clap](https://github.com/clap-rs/clap) for argument parsing.
