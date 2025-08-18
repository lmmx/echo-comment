# echo-comment

A bidirectional bash interpreter that converts between comments and echo statements, making your shell scripts cleaner and more maintainable.

## The Problem

Ever written a shell script like this?

```bash
#!/usr/bin/env bash
set -euo pipefail

echo "🧹 Cleaning up any existing .so files..."
find python/ -name "*.so" -delete || true

echo "📋 Running pre-commit checks..."
pre-commit run --all-files

echo "🔨 Building fresh release version..."
maturin develop --release

echo "📦 Downloading wheel artifacts..."
gh run download "${latest_run_id}" -p wheel*
```

The actual business logic is completely obscured by echo statements! 

## The Solution

Write your scripts with clean comments instead:

```bash
#!/usr/bin/env -S echo-comment --shell-flags="-euo pipefail"

# 🧹 Cleaning up any existing .so files...
find python/ -name "*.so" -delete || true

# 📋 Running pre-commit checks...
pre-commit run --all-files

# 🔨 Building fresh release version...
maturin develop --release

# 📦 Downloading wheel artifacts...
gh run download "${latest_run_id}" -p wheel*
```

At runtime, `echo-comment` automatically converts comments to echo statements, so you get the verbose output without cluttering your source code.

To write a comment that _doesn't_ get echoed, simply start the line with 2: `## ...`

The tool runs in either direction, so exposes two binary names:

- **`echo-comment`**: Converts `# comments` → `echo "comments"`
- **`comment-echo`**: Converts `echo "text"` → `# text`

## Demo

Here is our script `hello.sh` with 3 steps

- **Python**: We use Python to print "🐍 Hiss" and "🐱 Meow" and grep the snake emoji
- **Pause**: Then we do a no-op using the bash `:` operator
- **Echo** Lastly we `echo "Goodbye"`

```bash
# 1) Run some Python
python -c 'print("\N{SNAKE} Hiss\n\N{CAT FACE} Meow")' | grep 🐍

# 2) Pause for effect...
:

# 3) Complete
echo "Goodbye"
```

### bash

Run normally, you get _Hiss_ and _Goodbye_:

```bash
bash hello.sh 
```
```
🐍 Hiss
Goodbye
```

### echo-comment

_'Echo the comments'_: a running commentary on all 3 steps:

```bash
echo-comment hello.sh 
```
```
1) Run some Python
🐍 Hiss
2) Pause for effect...
3) Complete
Goodbye
```

### comment-echo

_'Comment out the echoes'_: _Hiss_, no _Goodbye_:

```bash
comment-echo hello.sh 
```
```
🐍 Hiss
```

## Features

- **Bidirectional**: `echo-comment` (comments → echoes) and `comment-echo` (echoes → comments)
- **Perfect for Justfiles**: Clean recipes that become verbose at runtime
- **Preserves formatting**: Maintains indentation and structure, does not need quotation marks
- **Variable expansion**: Comments support `$variables`, <code>`backtick subshells`</code>, and any other shell expansions
- **Opt out**: Use `##` for silent comments that won't be echoed, `#\#` to echo text starting with `#`
- **No dependencies**: Ships as a single binary
- **Cross-platform**: Works on Linux, macOS, and Windows

## Usage

### Shebang lines

As well as pointing the binary at a bash script, you can invoke it on a program from a shebang line.

The shebang line can be in one of two forms: **simple** (no configuration)

```sh
#!/usr/bin/env echo-comment 
```

or **with arguments** (setting fields in the [cli::Args](https://github.com/lmmx/echo-comment/blob/master/src/cli.rs) struct):

```sh
#!/usr/bin/env -S echo-comment --color green --shell bash --shell-flags="-euo pipefail"
```

The `Args` parser takes 1 positional argument as the bash script it's invoking and passes the rest on ("trailing" set) positional arguments.

Besides this, it takes some arguments which can be configured for your use:

- `--shell` / `-s` (default: "bash")
- `--shell-args` (no default). Note that they must **always** be passed with the `=` syntax.
- `--color` / `-c` (no default). One of the names in the
  [color](https://github.com/lmmx/echo-comment/blob/master/src/color.rs) module or an ANSI code of
  your choice.
  - Supports: black/red/green/yellow/blue/magenta/cyan/white plus "bright-" and "bold-" variants.
    Plus aliases: orange (= yellow) and pink (= magenta).

**Beware** of accidentally invoking it twice: if you use a script with an `echo-comment` shebang line with flags set,
but then point the `echo-comment` binary at it without flags set on the command line, the latter
takes precedence and your script will run without the flags in the shebang line. This may be unexpected!

### With Justfiles (Recommended)

Create clean, readable recipes:

```just
# justfile
build-and-publish:
    #!/usr/bin/env -S echo-comment --shell-flags="-euo pipefail"
    
    # 🧹 Cleaning up build artifacts...
    find . -name "*.so" -delete || true
    
    # 📋 Running tests...
    cargo test
    
    # 🚀 Publishing...
    cargo publish
```

When you run `just build-and-publish`, you'll see:

```
🧹 Cleaning up build artifacts...
📋 Running tests...
🚀 Publishing...
```

### Variable Expansion in Comments

Comments support full shell expansion - variables, and command substitution (subshells).

```just
demo-expand:
    #!/usr/bin/env echo-comment
    x=123
    # Value: $x
```

- The comment is equivalent to `echo "Value: $x"`

Running `just demo-expand` outputs:

```
Value: 123
```

This makes comments context-aware yet easy to silence, perfect for logging current state or computed values.

## Examples

### Converting Comments to Echoes

**Input (`script.sh`):**

```bash
#!/usr/bin/env bash
set -euo pipefail

# Starting deployment process
kubectl apply -f deployment.yaml

# Waiting for rollout
kubectl rollout status deployment/app

# Deployment complete!
echo "App is now live"
```

**Running with `echo-comment script.sh`:**

```bash
#!/usr/bin/env bash
set -euo pipefail

echo "Starting deployment process"
kubectl apply -f deployment.yaml

echo "Waiting for rollout" 
kubectl rollout status deployment/app

echo "Deployment complete!"
echo "App is now live"
```

### Converting Echoes to Comments

**Input (`verbose.sh`):**

```bash
#!/usr/bin/env bash
echo "Backing up database"
pg_dump mydb > backup.sql
echo "Database backed up successfully"
```

**Running with `comment-echo verbose.sh`:**

```bash
#!/usr/bin/env bash
# Backing up database
pg_dump mydb > backup.sql  
# Database backed up successfully
```

## Why This Approach?

1. **Low line noise**: Unlike `bash -v` or `set -x`, you just see the comment text
2. **Readability**: Foreground your actual logic with syntax highlighting
3. **Better Diffs**: Code changes are separate from message changes
4. **Maintainability**: Makes it trivial to document code

### Standalone Script Usage

```bash
# Convert comments to echoes and run
echo-comment script.sh

# Convert echoes to comments and run  
comment-echo verbose-script.sh

# Pass arguments to the script
echo-comment deploy.sh --env=production
```

Note that it does take flags and will consume any not separated with `--`.
These are used for custom colours etc. It will not consume `--help`.

### Ship Binary with Your Project

For maximum portability, ship the binary in your repo:

```bash
# Build and copy to your project
cargo build --release
cp target/release/comment-echo bin/comment-echo
cp target/release/echo-comment bin/echo-comment
chmod +x bin/comment-echo bin/echo-comment
```

Then use relative paths in your justfile:
```just
build:
    #!/usr/bin/env bin/comment-echo
    # Your clean script here...
```

## Contributing

Contributions welcome! Please feel free to submit a Pull Request.

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
