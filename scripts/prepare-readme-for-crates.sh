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

# Convert GitHub-style warning callout to standard markdown
# This is a more targeted approach that handles the specific pattern in the README
# Pattern: > [!WARNING]\
#          > This is a potentially destructive...
# Replace with: **Warning:** This is a potentially destructive...

# Remove the GitHub-specific callout marker and add **Warning:** prefix
awk '
/^> \[!WARNING\]\\$/ {
    # Skip the warning marker line and read the first warning content line
    if (getline > 0) {
        # Remove the "> " prefix and add **Warning:** prefix
        sub(/^> /, "**Warning:** ")
        print
        # Process remaining lines in the warning block
        while (getline > 0 && /^> /) {
            sub(/^> /, "")
            print
        }
        # Print the current line if it was successfully read
        if (NF > 0 || length($0) > 0) {
            print
        }
    }
    next
}
{ print }
' "$README_PATH" > "$TEMP_README"

echo "Prepared README for crates.io at $TEMP_README"
