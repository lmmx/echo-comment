use echo_comment::{run_script, Mode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env_args: Vec<String> = std::env::args().collect();

    if env_args.len() < 2 {
        eprintln!("Usage: comment-echo <script.sh> [args...]");
        eprintln!("Converts echo statements to comments and runs the script");
        std::process::exit(1);
    }

    let script = &env_args[1];
    let script_args: Vec<String> = env_args[2..].to_vec();

    run_script(script, &script_args, Mode::EchoToComment)
}
