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
# Pattern: > [!WARNING]\
#          > warning text
# Replace with: **Warning:** warning text
sed -i '/^> \[!WARNING\]\\$/,/^$/{
    /^> \[!WARNING\]\\$/d
    s/^> //
}' "$TEMP_README"

# Add bold "Warning:" prefix to the warning text
# Find the line that starts with "This is a potentially destructive"
sed -i 's/^This is a potentially destructive/**Warning:** This is a potentially destructive/' "$TEMP_README"

echo "Prepared README for crates.io at $TEMP_README"
