#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

default_branch() {
  local branch
  branch="$(git symbolic-ref --quiet --short refs/remotes/origin/HEAD 2>/dev/null || true)"
  if [[ -n "$branch" ]]; then
    echo "${branch#origin/}"
    return
  fi

  if git show-ref --verify --quiet refs/heads/main; then
    echo "main"
    return
  fi

  echo "master"
}

MAIN_BRANCH="$(default_branch)"

usage() {
  cat <<EOF
Usage:
  scripts/git_feature_flow.sh start <branch-name>
  scripts/git_feature_flow.sh push
  scripts/git_feature_flow.sh status

Commands:
  start <branch-name>  Create feature branch from ${MAIN_BRANCH} (latest)
  push                 Push current feature branch (rejects ${MAIN_BRANCH})
  status               Show branch/remote status and PR command hint
EOF
}

current_branch() {
  git rev-parse --abbrev-ref HEAD
}

require_feature_branch() {
  local current
  current="$(current_branch)"
  if [[ "$current" == "$MAIN_BRANCH" ]]; then
    echo "Refusing to continue on ${MAIN_BRANCH}. Create a feature branch first." >&2
    exit 1
  fi
}

cmd="${1:-}"
case "$cmd" in
  start)
    branch_name="${2:-}"
    if [[ -z "$branch_name" ]]; then
      echo "Missing branch name." >&2
      usage
      exit 1
    fi
    git fetch origin "$MAIN_BRANCH"
    git switch "$MAIN_BRANCH"
    git pull --ff-only origin "$MAIN_BRANCH"
    git switch -c "$branch_name"
    echo "Created and switched to branch: $branch_name"
    ;;
  push)
    require_feature_branch
    current="$(current_branch)"
    git push -u origin "$current"
    echo "Suggested PR command:"
    echo "  gh pr create --base $MAIN_BRANCH --head $current --fill"
    ;;
  status)
    current="$(current_branch)"
    echo "Default branch: $MAIN_BRANCH"
    echo "Current branch: $current"
    git status -sb
    if [[ "$current" != "$MAIN_BRANCH" ]]; then
      echo "Suggested PR command:"
      echo "  gh pr create --base $MAIN_BRANCH --head $current --fill"
    fi
    ;;
  *)
    usage
    exit 1
    ;;
esac
