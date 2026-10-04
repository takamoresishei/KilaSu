#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
set -euo pipefail
tree=${1:-$PWD}
script_path=${BASH_SOURCE[0]:-}
if [[ -n "$script_path" && -f "$script_path" && -f "$(dirname "$script_path")/integrate.py" ]]; then
 source_dir=$(cd "$(dirname "$script_path")/.." && pwd)
else
 source_dir="$PWD/.kilasu-source"
 if [[ ! -d "$source_dir/.git" ]]; then
  git clone --depth 1 --branch "${KILASU_REF:-main}" https://github.com/takamoresishei/KilaSu.git "$source_dir"
 fi
fi
python3 "$source_dir/scripts/integrate.py" "$tree"
