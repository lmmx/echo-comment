#!/usr/bin/env -S echo-comment --shell-flags="-euo pipefail"
# Test comment: this grep call will fail and we won't get the Python
echo "hello" | grep "world"
# hiss
python -c "print('Goodbye')
