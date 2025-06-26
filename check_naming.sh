#!/bin/bash
# Pre-commit hook: kiểm tra vi phạm định danh đơn từ toàn bộ workspace
set -e

CRATES=(architecture knowledge memories naming repository shared task)

VIOLATION=0
for crate in "${CRATES[@]}"; do
    echo "🔍 Kiểm tra naming cho crates/$crate/src ..."
    OUTPUT=$(./naming "crates/$crate/src" 2>&1)
    echo "$OUTPUT"
    if echo "$OUTPUT" | grep -q '\[VIOLATION\]'; then
        VIOLATION=1
    fi
    echo "✅ crates/$crate: Không có vi phạm naming nào."
done

if [ $VIOLATION -eq 1 ]; then
    echo "❌ Commit bị chặn do phát hiện vi phạm định danh đơn từ. Vui lòng refactor lại cho đúng quy tắc một từ tiếng Anh."
    exit 1
fi

echo "🎉 Toàn bộ workspace không có vi phạm naming!" 