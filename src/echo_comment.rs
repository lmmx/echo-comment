// echo-comment entrypoint
use echo_comment::{Mode, cli::run_cli};

/// Converts comments to echo statements and runs the script
fn main() {
    run_cli(Mode::CommentToEcho);
}
