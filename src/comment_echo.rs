// comment-echo entrypoint
use echo_comment::{Mode, cli::run_cli};

/// Converts echo statements to comments and runs the script
fn main() {
    run_cli(Mode::EchoToComment);
}
