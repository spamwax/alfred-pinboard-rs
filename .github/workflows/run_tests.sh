#!/usr/bin/env bash

# ls -la ./target/aarch64-apple-darwin/debug
# ls -la ./target/x86_64-apple-darwin/debug
set -ex

echo "RELEASE_COMMIT: =$RELEASE_COMMIT="

run_tests() {
    local runner="$1"
    local working_dir="$2"
    # runner="cargo run --target "$TARGET" --"
    export alfred_debug=1
    export alfred_version="4.5.1"
    export alfred_workflow_version=0.16.0
    export alfred_workflow_uid=hamid63
    export alfred_workflow_name="RustyPin"
    export alfred_workflow_bundleid=cc.hamid.alfred-pinboard-rs
    workflow_dir="$working_dir/.config/alfred-pinboard-rs"
    mkdir -p "$workflow_dir"
    export alfred_workflow_data="$workflow_dir"
    export alfred_workflow_cache="$workflow_dir"
    if grep -Eq '[0-9]+/merge' <<< "$GITHUB_REF_NAME"; then
        echo "This is a pull request!"
        PINBOARD_TOKEN="hamid:123456"
        $runner config --authorization "${PINBOARD_TOKEN}"
    else
        echo "This is NOT a pull request"
        $runner config --authorization "${PINBOARD_TOKEN}"
        $runner update
        sleep 2
        $runner search -U rust async
    fi
    $runner config -d
    echo
    ls -la "$alfred_workflow_data"
    sleep 2
    ls -la "$2"
    sleep 2
    ls -la "$2/.config"
    sleep 2
    rm -rf "$2/.config"
    sleep 2

}

echo "$GITHUB_WORKSPACE == $GITHUB_REF_NAME"
executable=$1
chmod u+x "$executable"
strip "$executable" || true
run_tests "$executable" "$GITHUB_WORKSPACE"
