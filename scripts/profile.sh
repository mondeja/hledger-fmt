#!/usr/bin/env bash
# Profile hledger-fmt and identify potential improvements
#
# Usage: ./scripts/profile.sh [parse|format|roundtrip]

set -euo pipefail

BENCH_TYPE="${1:-roundtrip}"

echo "=== Profiling hledger-fmt ($BENCH_TYPE) ==="
echo ""

# Check if flamegraph is installed
if ! command -v flamegraph &> /dev/null; then
    echo "Error: flamegraph not found. Install it with:"
    echo "  cargo install flamegraph"
    echo ""
    exit 1
fi

# Determine which benchmark to profile
case "$BENCH_TYPE" in
    parse)
        BENCH_NAME="parse"
        ;;
    format)
        BENCH_NAME="format"
        ;;
    roundtrip)
        BENCH_NAME="roundtrip"
        ;;
    *)
        echo "Error: Unknown benchmark type '$BENCH_TYPE'"
        echo "Usage: $0 [parse|format|roundtrip]"
        exit 1
        ;;
esac

echo "1. Running baseline benchmark..."
cargo bench --bench "$BENCH_NAME" --features bench 2>&1 | grep -A 2 "time:"

echo ""
echo "2. Generating flamegraph for $BENCH_NAME..."
echo "   This will take a few moments..."

# Generate flamegraph
mkdir -p reports
cargo flamegraph --bench "$BENCH_NAME" --features bench -o "reports/flamegraph-$BENCH_NAME.svg" -- --bench

echo ""
echo "=== Profiling complete! ==="
echo ""
echo "Flamegraph saved to: reports/flamegraph-$BENCH_NAME.svg"
echo ""
echo "To view the flamegraph:"
echo "  - Open flamegraph-$BENCH_NAME.svg in your browser"
echo "  - Look for wide bars (functions taking most time)"
echo "  - Click to zoom into specific code paths"
echo ""
echo "Hot paths to examine:"
echo "  - Parser: parse_content, maybe_start_with_directive, parse_transaction_entry"
echo "  - Formatter: format_nodes, extend_entry, spaces::extend"
echo "  - Utilities: utf8_chars_count, split_value_in_before_decimals_after_decimals"
echo ""
echo "See PROFILING.md for more profiling options and analysis tips."
