use clap::Parser;
use std::process::{Command, ExitCode};
use std::time::Duration;
use std::thread;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::io;

#[derive(Parser)]
#[command(name = "timeout")]
#[command(about = "Run a command with a timeout")]
#[command(version)]
struct Args {
    #[arg(help = "Number of seconds to wait before timing out")]
    seconds: u64,
    
    #[arg(short = 'k', long = "kill-after", help = "Also send KILL signal after this many seconds")]
    kill_after: Option<u64>,
    
    #[arg(short = 'v', long = "verbose", help = "Print debug information")]
    verbose: bool,
    
    #[arg(help = "Command to execute", required = true)]
    command: String,
    
    #[arg(help = "Arguments for the command", trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

// Exit codes following GNU timeout convention
const EXIT_TIMEOUT: u8 = 124;      // Command timed out
const EXIT_TIMEOUT_FAIL: u8 = 125; // timeout command itself failed
const EXIT_CANNOT_INVOKE: u8 = 126; // Command found but cannot be invoked
const EXIT_NOT_FOUND: u8 = 127;     // Command not found
const EXIT_KILLED: u8 = 137;        // Command killed with KILL signal (128+9)

#[derive(Debug)]
enum TimeoutResult {
    Completed(i32),
    TimedOut,
    Killed,
    NotFound,
    CannotInvoke,
    InternalError,
}

macro_rules! debug_print {
    ($verbose:expr, $($arg:tt)*) => {
        if $verbose {
            eprintln!("DEBUG: {}", format!($($arg)*));
        }
    };
}

fn main() -> ExitCode {
    let args = Args::parse();
    
    let timeout_duration = Duration::from_secs(args.seconds);
    let kill_after_duration = args.kill_after.map(Duration::from_secs);
    let verbose = args.verbose;
    
    debug_print!(verbose, "Starting timeout: {}s, kill-after: {:?}s, command: {}", 
                 args.seconds, args.kill_after, args.command);
    
    let should_terminate = Arc::new(AtomicBool::new(false));
    let should_kill = Arc::new(AtomicBool::new(false));
    
    let (tx, rx) = mpsc::channel();
    
    // Spawn the command
    let should_terminate_clone = should_terminate.clone();
    let should_kill_clone = should_kill.clone();
    let command_name = args.command.clone();
    
    let command_thread = thread::spawn(move || {
        let mut cmd = Command::new(&args.command);
        cmd.args(&args.args);
        
        debug_print!(verbose, "Spawning command: {} {:?}", args.command, args.args);
        
        let mut child = match cmd.spawn() {
            Ok(child) => {
                debug_print!(verbose, "Command spawned successfully with PID: {}", child.id());
                child
            }
            Err(e) => {
                debug_print!(verbose, "Failed to spawn command: {}", e);
                let exit_code = match e.kind() {
                    io::ErrorKind::NotFound => {
                        eprintln!("timeout: failed to run command '{}': No such file or directory", command_name);
                        TimeoutResult::NotFound
                    }
                    io::ErrorKind::PermissionDenied => {
                        eprintln!("timeout: failed to run command '{}': Permission denied", command_name);
                        TimeoutResult::CannotInvoke
                    }
                    _ => {
                        eprintln!("timeout: failed to run command '{}': {}", command_name, e);
                        TimeoutResult::InternalError
                    }
                };
                let _ = tx.send(exit_code);
                return;
            }
        };
        
        let mut term_sent = false;
        
        loop {
            // Check if we should send KILL signal
            if should_kill_clone.load(Ordering::Relaxed) {
                debug_print!(verbose, "Sending KILL signal to PID {}", child.id());
                let _ = child.kill();
                let _ = child.wait();
                let _ = tx.send(TimeoutResult::Killed);
                debug_print!(verbose, "Command killed with KILL signal");
                return;
            }
            
            // Check if we should send TERM signal
            if should_terminate_clone.load(Ordering::Relaxed) && !term_sent {
                debug_print!(verbose, "Timeout reached, sending TERM signal to PID {}", child.id());
                
                #[cfg(unix)]
                {
                    // Send TERM signal first
                    unsafe {
                        let result = libc::kill(child.id() as i32, libc::SIGTERM);
                        debug_print!(verbose, "SIGTERM sent, result: {}", result);
                    }
                }
                #[cfg(not(unix))]
                {
                    debug_print!(verbose, "Non-Unix system, using kill()");
                    let _ = child.kill();
                }
                
                term_sent = true;
                
                // If no kill-after, wait briefly then kill and exit
                if kill_after_duration.is_none() {
                    debug_print!(verbose, "No kill-after specified, waiting 100ms then killing");
                    thread::sleep(Duration::from_millis(100));
                    let _ = child.kill();
                    let _ = child.wait();
                    let _ = tx.send(TimeoutResult::TimedOut);
                    debug_print!(verbose, "Command terminated after timeout");
                    return;
                }
                debug_print!(verbose, "Kill-after specified, waiting for KILL signal or process completion");
                // If kill-after is set, continue loop and wait for KILL signal
            }
            
            match child.try_wait() {
                Ok(Some(status)) => {
                    let exit_code = status.code().unwrap_or(-1);
                    debug_print!(verbose, "Command exited with code: {}, term_sent: {}", exit_code, term_sent);
                    
                    // If we sent TERM and process exited
                    if term_sent {
                        // If kill-after was specified, process responded to TERM - this is still a timeout
                        if kill_after_duration.is_some() {
                            debug_print!(verbose, "Process responded to TERM signal (kill-after was available) - treating as timeout");
                            let _ = tx.send(TimeoutResult::TimedOut);
                        } else {
                            debug_print!(verbose, "Process exited after TERM signal - treating as timeout");
                            let _ = tx.send(TimeoutResult::TimedOut);
                        }
                    } else {
                        debug_print!(verbose, "Process completed normally");
                        let _ = tx.send(TimeoutResult::Completed(exit_code));
                    }
                    return;
                }
                Ok(None) => {
                    // Command still running
                    thread::sleep(Duration::from_millis(10));
                }
                Err(e) => {
                    debug_print!(verbose, "Error waiting for child: {}", e);
                    eprintln!("timeout: error waiting for child process: {}", e);
                    let _ = tx.send(TimeoutResult::InternalError);
                    return;
                }
            }
        }
    });
    
    // Timeout thread for TERM signal
    let should_terminate_timer = should_terminate.clone();
    let _timeout_thread = thread::spawn(move || {
        debug_print!(verbose, "Timeout thread started, sleeping for {}s", timeout_duration.as_secs());
        thread::sleep(timeout_duration);
        debug_print!(verbose, "Timeout reached, setting terminate flag");
        should_terminate_timer.store(true, Ordering::Relaxed);
    });
    
    // Kill-after thread for KILL signal
    if let Some(kill_duration) = kill_after_duration {
        let should_kill_timer = should_kill.clone();
        let total_duration = timeout_duration + kill_duration;
        let _kill_thread = thread::spawn(move || {
            debug_print!(verbose, "Kill-after thread started, sleeping for {}s total", total_duration.as_secs());
            thread::sleep(total_duration);
            debug_print!(verbose, "Kill-after timeout reached, setting kill flag");
            should_kill_timer.store(true, Ordering::Relaxed);
        });
    }
    
    debug_print!(verbose, "Waiting for command result...");
    let result = rx.recv().unwrap_or(TimeoutResult::InternalError);
    debug_print!(verbose, "Command result received: {:?}", result);
    
    let _ = command_thread.join();
    
    let exit_code = match result {
        TimeoutResult::Completed(exit_code) => {
            debug_print!(verbose, "Command completed normally with exit code {}", exit_code);
            if (0..=255).contains(&exit_code) {
                ExitCode::from(exit_code as u8)
            } else {
                ExitCode::from(1)
            }
        }
        TimeoutResult::TimedOut => {
            debug_print!(verbose, "Command timed out");
            ExitCode::from(EXIT_TIMEOUT)
        }
        TimeoutResult::Killed => {
            debug_print!(verbose, "Command killed with KILL signal");
            ExitCode::from(EXIT_KILLED)
        }
        TimeoutResult::NotFound => {
            debug_print!(verbose, "Command not found");
            ExitCode::from(EXIT_NOT_FOUND)
        }
        TimeoutResult::CannotInvoke => {
            debug_print!(verbose, "Command cannot be invoked");
            ExitCode::from(EXIT_CANNOT_INVOKE)
        }
        TimeoutResult::InternalError => {
            debug_print!(verbose, "Internal error occurred");
            ExitCode::from(EXIT_TIMEOUT_FAIL)
        }
    };
    
    debug_print!(verbose, "Exiting with code: {:?}", exit_code);
    exit_code
}
