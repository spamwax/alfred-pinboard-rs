#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RUN_TESTS=1
BUILD_ARGS=()

usage() {
  cat <<'EOF'
Usage:
  scripts/bootstrap_workflow.sh [options] [-- build_workflow_args...]

Options:
  --skip-tests     Skip cargo test
  -h, --help       Show this message

Examples:
  scripts/bootstrap_workflow.sh
  scripts/bootstrap_workflow.sh -- --version 0.2.0
  scripts/bootstrap_workflow.sh --skip-tests -- --profile debug
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --skip-tests)
      RUN_TESTS=0
      shift
      ;;
    --)
      shift
      BUILD_ARGS=("$@")
      break
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage
      exit 1
      ;;
  esac
done

pushd "$ROOT_DIR" >/dev/null

if [[ "$RUN_TESTS" -eq 1 ]]; then
  cargo test
fi

"$ROOT_DIR/scripts/build_workflow.sh" "${BUILD_ARGS[@]}"

echo "Next:"
echo "  1) Double-click the generated .alfredworkflow file"
echo "  2) In Alfred workflow variables, optionally set BINARY_PATH"
echo "  3) Run keyword 'cb' in Alfred to verify results"

popd >/dev/null
