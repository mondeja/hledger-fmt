#!/usr/bin/env bash
# Analyze binary size and identify code bloat in hledger-fmt
#
# Usage: ./scripts/bloat-analysis.sh [--install] [--json]

set -euo pipefail

INSTALL=false
JSON_OUTPUT=false

# Parse arguments
for arg in "$@"; do
    case $arg in
        --install)
            INSTALL=true
            shift
            ;;
        --json)
            JSON_OUTPUT=true
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [--install] [--json]"
            echo ""
            echo "Options:"
            echo "  --install    Install cargo-bloat if not present"
            echo "  --json       Output results in JSON format"
            echo "  --help, -h   Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $arg"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

echo "=== Code Size Analysis with cargo-bloat ==="
echo ""

# Check if cargo-bloat is installed
if ! command -v cargo-bloat &> /dev/null; then
    if [ "$INSTALL" = true ]; then
        echo "Installing cargo-bloat..."
        cargo install cargo-bloat
        echo ""
    else
        echo "Error: cargo-bloat not found. Install it with:"
        echo "  cargo install cargo-bloat"
        echo ""
        echo "Or run this script with --install flag:"
        echo "  $0 --install"
        echo ""
        exit 1
    fi
fi

# Build in release mode
echo "1. Building hledger-fmt in release mode..."
cargo build --release --quiet
echo "   ✓ Build complete"
echo ""

# Run cargo-bloat analysis
echo "2. Analyzing binary size..."
echo ""

if [ "$JSON_OUTPUT" = true ]; then
    # JSON output for programmatic analysis
    OUTPUT_FILE="target/bloat-analysis.json"
    cargo bloat --release -n 50 --message-format json > "$OUTPUT_FILE"
    echo "   ✓ Analysis saved to: $OUTPUT_FILE"
else
    # Human-readable output
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "Top 30 functions by size:"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    cargo bloat --release -n 30

    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "Crate-level breakdown:"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    cargo bloat --release --crates
fi

echo ""
echo "=== Analysis complete! ==="
echo ""
echo "Key insights to look for:"
echo "  - Large functions that could be split or optimized"
echo "  - Generic instantiations taking significant space"
echo "  - Unused dependencies contributing to binary size"
echo "  - Opportunities for code deduplication"
echo ""
echo "Optimization strategies:"
echo "  1. Mark large functions with #[inline(never)] if rarely called"
echo "  2. Reduce generic instantiations by using trait objects"
echo "  3. Enable LTO and reduce codegen-units in Cargo.toml"
echo "  4. Strip debug symbols and use opt-level='z' for size"
echo ""
echo "See PROFILING.md for more optimization tips."
