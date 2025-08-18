use facet::Facet;
use echo_comment::{run_script, Mode};

#[derive(Facet)]
struct Args {
    /// Script file to process and execute
    #[facet(positional)]
    script: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env_args: Vec<String> = std::env::args().collect();
    
    if env_args.len() < 2 {
        eprintln!("Usage: echo-comment <script.sh> [args...]");
        eprintln!("Converts comments to echo statements and runs the script");
        std::process::exit(1);
    }
    
    // Parse just the script name with facet-args
    let str_args: Vec<&str> = env_args[1..2].iter().map(|s| s.as_str()).collect();
    let args: Args = facet_args::from_slice(&str_args).map_err(|e| {
        eprintln!("Usage: echo-comment <script.sh> [args...]");
        eprintln!("Error: {}", e);
        std::process::exit(1);
    })?;

    // Everything after the script name goes to the script
    let script_args: Vec<String> = env_args[2..].to_vec();

    run_script(&args.script, &script_args, Mode::CommentToEcho)
}
