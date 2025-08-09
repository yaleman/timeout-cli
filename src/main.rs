use clap::Parser;
use std::process::{Command, ExitCode};
use std::time::Duration;
use std::thread;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Parser)]
#[command(name = "timeout")]
#[command(about = "Run a command with a timeout")]
struct Args {
    #[arg(help = "Number of seconds to wait before timing out")]
    seconds: u64,
    
    #[arg(help = "Command to execute", required = true)]
    command: String,
    
    #[arg(help = "Arguments for the command", trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

fn main() -> ExitCode {
    let args = Args::parse();
    
    let timeout_duration = Duration::from_secs(args.seconds);
    let should_terminate = Arc::new(AtomicBool::new(false));
    let should_terminate_clone = should_terminate.clone();
    
    let (tx, rx) = mpsc::channel();
    
    let command_thread = thread::spawn(move || {
        let mut cmd = Command::new(&args.command);
        cmd.args(&args.args);
        
        let mut child = match cmd.spawn() {
            Ok(child) => child,
            Err(e) => {
                eprintln!("Failed to execute command: {}", e);
                tx.send(Err(1)).unwrap();
                return;
            }
        };
        
        loop {
            if should_terminate_clone.load(Ordering::Relaxed) {
                let _ = child.kill();
                let _ = child.wait();
                tx.send(Err(124)).unwrap(); // timeout exit code
                return;
            }
            
            match child.try_wait() {
                Ok(Some(status)) => {
                    let exit_code = status.code().unwrap_or(1);
                    tx.send(Ok(exit_code)).unwrap();
                    return;
                }
                Ok(None) => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(e) => {
                    eprintln!("Error waiting for child process: {}", e);
                    tx.send(Err(1)).unwrap();
                    return;
                }
            }
        }
    });
    
    let _timeout_thread = thread::spawn(move || {
        thread::sleep(timeout_duration);
        should_terminate.store(true, Ordering::Relaxed);
    });
    
    let result = rx.recv().unwrap();
    
    let _ = command_thread.join();
    // Don't wait for timeout thread - let it finish on its own
    
    match result {
        Ok(exit_code) => ExitCode::from(exit_code as u8),
        Err(exit_code) => ExitCode::from(exit_code as u8),
    }
}
