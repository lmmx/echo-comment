# Build and ship both binaries to bin/ directory
ship-binaries:
    cargo build --release
    cp target/release/comment-echo bin/comment-echo
    cp target/release/echo-comment bin/echo-comment
    chmod +x bin/comment-echo bin/echo-comment

# Your clean recipe using comment-echo (comments → echoes)
foo:
    #!/usr/bin/env bin/comment-echo
    set -euo pipefail
    # 🎉 Hello world from comment!
    # 🔧 This is being processed by comment-echo
    echo "This is a regular command"
    # ✅ Another comment that becomes an echo

# Convert an echo-heavy script back to comment form
clean-script:
    #!/usr/bin/env bin/echo-comment
    set -euo pipefail
    echo "🧹 This echo becomes a comment"
    echo "foo" | grep "f"
    echo "📋 This one too"
    ls -la | grep "."
