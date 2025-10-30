#!/usr/bin/env bash
# Script to prepare README.md for publishing to crates.io
# Converts GitHub-specific markdown syntax to standard markdown

set -euo pipefail

README_PATH="${1:-README.md}"
TEMP_README="${2:-README.crates.md}"

if [ ! -f "$README_PATH" ]; then
    echo "Error: README file not found at $README_PATH" >&2
    exit 1
fi

# Copy README to temporary file
cp "$README_PATH" "$TEMP_README"

# Convert GitHub-style warning callout to standard markdown
# This is a more targeted approach that handles the specific pattern in the README
# Pattern: > [!WARNING]\
#          > This is a potentially destructive...
# Replace with: **Warning:** This is a potentially destructive...

# First, remove the GitHub-specific callout marker and preserve the content
# Then add the **Warning:** prefix to the first line of content
awk '
/^> \[!WARNING\]\\$/ {
    # Skip the warning marker line
    getline
    # Remove the "> " prefix and add **Warning:** prefix
    sub(/^> /, "**Warning:** ")
    print
    # Process remaining lines in the warning block
    while (getline && /^> /) {
        sub(/^> /, "")
        print
    }
    # Print the current line (empty line or next content)
    print
    next
}
{ print }
' "$README_PATH" > "$TEMP_README"

echo "Prepared README for crates.io at $TEMP_README"
