# comment-echo

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
#!/usr/bin/env comment-echo
set -euo pipefail

# 🧹 Cleaning up any existing .so files...
find python/ -name "*.so" -delete || true

# 📋 Running pre-commit checks...
pre-commit run --all-files

# 🔨 Building fresh release version...
maturin develop --release

# 📦 Downloading wheel artifacts...
gh run download "${latest_run_id}" -p wheel*
```

At runtime, `comment-echo` automatically converts comments to echo statements, so you get the verbose output without cluttering your source code.

## Example

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

To run this normally you get a _Hiss_ and a _Goodbye_:

```bash
louis 🌟 ~/dev/comment-echo $ bash hello.sh 
```
⇣
```
🐍 Hiss
Goodbye
```

To run this via `echo-comment` you get a _Hiss_ and no more _Goodbye_ (the `echo` became a comment):

```bash
louis 🌟 ~/dev/comment-echo $ ./target/debug/echo-comment hello.sh 
```
⇣
```
🐍 Hiss
```

To run this via `comment-echo` you get a running commentary on all 3 steps (the comments now `echo`):

```bash
louis 🌟 ~/dev/comment-echo $ ./target/debug/comment-echo hello.sh 
```
⇣
```
1) Run some Python
🐍 Hiss
2) Pause for effect...
3) Complete
Goodbye
```

## Features

- **Bidirectional conversion**: `comment-echo` (comments → echoes) and `echo-comment` (echoes → comments)
- **Perfect for Justfiles**: Clean recipes that become verbose at runtime
- **Preserves formatting**: Maintains indentation and structure
- **No dependencies**: Ships as a single binary
- **Cross-platform**: Works on Linux, macOS, and Windows

## Installation

### Option 1: Install from crates.io
```bash
cargo install comment-echo
```

### Option 2: Install with cargo-binstall
```bash
cargo binstall comment-echo
```

### Option 3: Build from source
```bash
git clone https://github.com/yourusername/comment-echo
cd comment-echo
cargo build --release
```

Both `comment-echo` and `echo-comment` binaries will be installed.

## Usage

### With Justfiles (Recommended)

Create clean, readable recipes:

```just
# justfile
build-and-publish:
    #!/usr/bin/env comment-echo
    set -euo pipefail
    
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

### Standalone Script Usage

```bash
# Convert comments to echoes and run
comment-echo script.sh

# Convert echoes to comments and run  
echo-comment verbose-script.sh

# Pass arguments to the script
comment-echo deploy.sh --env=production
```

## Mode Detection

The tool automatically detects which mode to use based on the binary name:

- **`comment-echo`**: Converts `# comments` → `echo "comments"`
- **`echo-comment`**: Converts `echo "text"` → `# text`

Both binaries are the same executable - the behavior changes based on how it's invoked.

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

**Running with `comment-echo script.sh`:**
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

**Running with `echo-comment verbose.sh`:**
```bash
#!/usr/bin/env bash
# Backing up database
pg_dump mydb > backup.sql  
# Database backed up successfully
```

## Integration with Pre-commit

Add to your `.pre-commit-config.yaml`:

```yaml
repos:
  - repo: local
    hooks:
      - id: build-check
        name: Build Check
        entry: bin/comment-echo
        args: [scripts/build-check.sh]
        language: system
        pass_filenames: false
```

## Why This Approach?

1. **Separation of Concerns**: Comments describe intent, code does work
2. **Better Diffs**: Code changes are separate from message changes
3. **IDE Support**: Proper syntax highlighting for your actual logic
4. **Maintainability**: Easy to update documentation without touching echo statements
5. **Round-trip**: Convert echo-heavy legacy scripts to clean format

## Contributing

Contributions welcome! Please feel free to submit a Pull Request.

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
