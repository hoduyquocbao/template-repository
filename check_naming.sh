#!/bin/bash
# Script kiểm tra vi phạm định danh đơn từ cho toàn bộ workspace Rust
# Yêu cầu: đã build xong binary naming

set -e

CRATES=(architecture knowledge memories naming repository shared task)

for crate in "${CRATES[@]}"; do
    echo "🔍 Kiểm tra naming cho crates/$crate/src ..."
    cargo run --bin naming "crates/$crate/src"
    echo "✅ crates/$crate: Không có vi phạm naming nào."
done

echo "🎉 Toàn bộ workspace không có vi phạm naming!" 