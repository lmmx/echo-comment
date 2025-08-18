use echo_comment::{Mode, cli::run_cli};

fn main() {
    run_cli(
        Mode::CommentToEcho,
        "echo-comment",
        "Converts comments to echo statements and runs the script",
    );
}
