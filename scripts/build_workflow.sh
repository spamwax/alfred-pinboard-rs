#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORKFLOW_TEMPLATE="$ROOT_DIR/res/chrome-workflow"
PROFILE="release"
OUTPUT_PATH="$ROOT_DIR/dist/AlfredChromeBookmarks.alfredworkflow"
BINARY_PATH=""
VERSION=""
SKIP_BUILD=0

usage() {
  cat <<'EOF'
Usage:
  scripts/build_workflow.sh [options]

Options:
  --profile <release|debug>     Build profile (default: release)
  --binary <path>               Use a prebuilt binary instead of target path
  --output <path>               Output .alfredworkflow path
  --template <path>             Workflow template directory (default: res/chrome-workflow)
  --version <semver>            Write version into info.plist (e.g. 0.2.0)
  --skip-build                  Skip cargo build and only package files
  -h, --help                    Show this message
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --profile)
      PROFILE="${2:-}"
      shift 2
      ;;
    --binary)
      BINARY_PATH="${2:-}"
      shift 2
      ;;
    --output)
      OUTPUT_PATH="${2:-}"
      shift 2
      ;;
    --template)
      WORKFLOW_TEMPLATE="${2:-}"
      shift 2
      ;;
    --version)
      VERSION="${2:-}"
      shift 2
      ;;
    --skip-build)
      SKIP_BUILD=1
      shift
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

if [[ "$PROFILE" != "release" && "$PROFILE" != "debug" ]]; then
  echo "Invalid --profile: $PROFILE (must be release or debug)" >&2
  exit 1
fi

if [[ ! -d "$WORKFLOW_TEMPLATE" ]]; then
  echo "Workflow template not found: $WORKFLOW_TEMPLATE" >&2
  exit 1
fi

if [[ -z "$BINARY_PATH" ]]; then
  BINARY_PATH="$ROOT_DIR/target/$PROFILE/alfred-chrome-bookmarks"
fi

if [[ "$SKIP_BUILD" -eq 0 ]]; then
  pushd "$ROOT_DIR" >/dev/null
  if [[ "$PROFILE" == "release" ]]; then
    cargo build --release
  else
    cargo build
  fi
  popd >/dev/null
fi

if [[ ! -x "$BINARY_PATH" ]]; then
  echo "Executable binary not found or not executable: $BINARY_PATH" >&2
  exit 1
fi

mkdir -p "$(dirname "$OUTPUT_PATH")"
TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/alfred-workflow.XXXXXX")"
cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

cp -R "$WORKFLOW_TEMPLATE/." "$TMP_DIR/"
cp "$BINARY_PATH" "$TMP_DIR/alfred-chrome-bookmarks"
chmod +x "$TMP_DIR/alfred-chrome-bookmarks"
chmod +x "$TMP_DIR/run.sh"

if [[ -n "$VERSION" ]]; then
  if /usr/libexec/PlistBuddy -c "Print :version" "$TMP_DIR/info.plist" >/dev/null 2>&1; then
    /usr/libexec/PlistBuddy -c "Set :version $VERSION" "$TMP_DIR/info.plist"
  else
    /usr/libexec/PlistBuddy -c "Add :version string $VERSION" "$TMP_DIR/info.plist"
  fi
fi

rm -f "$OUTPUT_PATH"
(
  cd "$TMP_DIR"
  zip -qr "$OUTPUT_PATH" .
)

echo "Workflow package created:"
echo "  $OUTPUT_PATH"
