use echo_comment::{Mode, cli::run_cli};

fn main() {
    run_cli(
        Mode::EchoToComment,
        "comment-echo",
        "Converts echo statements to comments and runs the script",
    );
}
