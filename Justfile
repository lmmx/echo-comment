default: test

e:
    $EDITOR Justfile

# Justfile recipe using echo-comment (echo the comments)
demo-jf:
    #!/usr/bin/env echo-comment
    set -euo pipefail
    # 🎉 Hello world from comment!
    python -c "print(f'{2+2=}')"
    # ✅ Mission complete

test:
    cargo nextest run

# Run the echo-comment example (comments become echoes)
demo-ec:
    bat examples/example-echo-comment.sh
    ./target/debug/echo-comment examples/example-echo-comment.sh

# Run the comment-echo example (echoes become comments) 
demo-ce:
    bat examples/example-echo-comment.sh
    ./target/debug/comment-echo examples/example-comment-echo.sh

# Convert an echo-heavy script back to comment form
clean-script:
    #!/usr/bin/env bin/echo-comment
    set -euo pipefail
    echo "🧹 This echo becomes a comment"
    echo "foo" | grep "f"
    echo "📋 This one too"
    ls -la | grep "."

ship:
    #!/usr/bin/env -S bash -euo pipefail
    # Refuse to run if not on master branch or not up to date with origin/master
    branch="$(git rev-parse --abbrev-ref HEAD)"
    if [[ "$branch" != "master" ]]; then
    echo -e "\033[1;31m❌ Refusing to run: not on 'master' branch (current: $branch)\033[0m"
    exit 1
    fi
    git fetch origin master
    local_rev="$(git rev-parse HEAD)"
    remote_rev="$(git rev-parse origin/master)"
    if [[ "$local_rev" != "$remote_rev" ]]; then
    echo -e "\033[1;31m❌ Refusing to run: local master branch is not up to date with origin/master\033[0m"
    echo -e "Local HEAD:  $local_rev"
    echo -e "Origin HEAD: $remote_rev"
    echo -e "Please pull/rebase to update."
    exit 1
    fi
    release-plz update
    git add .
    git commit -m "Upgrades"
    git push
    just publish

publish:
    git_token := $(gh auth token 2>/dev/null) || echo $PUBLISH_GITHUB_TOKEN
    release-plz release --backend github --git-token $git_token

# ------------------------------------------------------------

fix-eof-ws mode="":
    #!/usr/bin/env sh
    ARGS=''
    if [ "{{mode}}" = "check" ]; then
        ARGS="--check-only"
    fi
    whitespace-format --add-new-line-marker-at-end-of-file \
          --new-line-marker=linux \
          --normalize-new-line-markers \
          --exclude ".git/|target/|dist/|.json$|.lock$|.parquet$|.venv/|.stubs/|\..*cache/" \
          $ARGS \
          .

code-quality:
    taplo lint
    taplo format --check
    just fix-eof-ws check
    cargo machete
    cargo fmt --check --all

code-quality-fix:
    taplo lint
    taplo format
    just fix-eof-ws
    cargo machete
    cargo fmt --all

