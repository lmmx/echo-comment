#!/usr/bin/env bash

# Check if we have enough arguments
if [ $# -lt 2 ]; then
    echo "Usage: $0 <name> <handle> [city]"
    exit 1
fi

# Assign arguments to variables
name="$1"
handle="$2"
city="${3:-Unknown}"  # Default to "Unknown" if not provided

# Use the arguments
echo "Hello, $name!"
echo "You are @$handle on GitHub."
echo "You live in: $city"

# Show all arguments
echo "All arguments: $@"
echo "Number of arguments: $#"
