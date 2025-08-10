# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2024-08-09

### Added
- Initial release of timeout-cli
- Basic timeout functionality with configurable duration
- Command execution with proper exit code forwarding
- Advanced exit codes matching GNU timeout standards:
  - 124: Command timed out
  - 125: timeout command itself failed  
  - 126: Command found but cannot be invoked (permission denied)
  - 127: Command not found
  - 137: Command killed with KILL signal
- Kill-after functionality (`--kill-after`) for SIGTERM → SIGKILL escalation
- Verbose debugging mode (`--verbose`) for troubleshooting
- Comprehensive signal handling with proper SIGTERM/SIGKILL timing
- Cross-platform support (Linux, macOS, Windows)
- Comprehensive test suite with 25 integration tests
- Full GNU timeout compatibility for core features

### Features
- Command-line interface using clap with proper argument parsing
- Process group handling and signal propagation
- Thread-safe timeout and kill-after coordination
- Detailed debug output for troubleshooting process behavior
- Proper handling of responsive vs unresponsive processes
- Edge case support (zero timeouts, quick commands, error conditions)

### Documentation
- Comprehensive README with usage examples
- Detailed exit code documentation
- Feature comparison with GNU timeout
- Installation instructions for multiple methods
- Contributing guidelines and issue reporting information

### Testing
- 25 comprehensive integration tests covering:
  - Normal command execution and completion
  - Timeout scenarios with SIGTERM handling
  - Kill-after scenarios with SIGKILL escalation  
  - Exit code forwarding and error conditions
  - Cross-platform compatibility verification
  - Performance and timing validation

## [Unreleased]

### Planned
- Duration suffixes support (m, h, d) for time specifications
- Preserve exit status option (`--preserve-status`)
- Custom signal specification (`--signal`)
- Process group control (`--foreground`)
- Verbose output on timeout (`--verbose` timeout messages)

---

## Release Notes

### v0.1.0 - Initial Release

This is the first stable release of timeout-cli, providing a reliable alternative to GNU timeout with the following key capabilities:

- **GNU Compatibility**: Implements core GNU timeout features including proper exit codes and kill-after functionality
- **Robust Process Management**: Reliable SIGTERM/SIGKILL escalation for stubborn processes
- **Cross-Platform**: Works on Linux, macOS, and Windows with appropriate signal handling
- **Well-Tested**: Comprehensive test suite ensures reliability across different scenarios
- **Easy Installation**: Available via cargo-binstall for fast binary installation

The implementation focuses on reliability and compatibility, making it suitable for shell scripts, CI/CD pipelines, and system administration tasks where process timeout management is critical.