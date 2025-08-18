use crate::{Mode, run_script};

/// Run the CLI with the specified mode and binary name for usage messages
pub fn run_cli(mode: Mode, binary_name: &str, description: &str) {
    let env_args: Vec<String> = std::env::args().collect();

    if env_args.len() < 2 {
        eprintln!("Usage: {} <script.sh> [args...]", binary_name);
        eprintln!("{}", description);
        std::process::exit(1);
    }

    let script = &env_args[1];
    let script_args: Vec<String> = env_args[2..].to_vec();

    if let Err(e) = run_script(script, &script_args, mode) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
