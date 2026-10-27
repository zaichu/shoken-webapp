#!/usr/bin/env bash
set +x
set -euo pipefail
exec python3 -B "${BASH_SOURCE[0]%/*}/db_migration.py" "$@"
