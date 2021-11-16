#!/usr/bin/env bash

# ls -la ./target/aarch64-apple-darwin/debug
# ls -la ./target/x86_64-apple-darwin/debug
set -ex

echo "RELEASE_COMMIT: =$RELEASE_COMMIT="

build_alfred_bundle() {
    src=$1
    stage=$2
    test -f Cargo.lock || cargo generate-lockfile

    # TODO Update this to build the artifacts that matter to you
    # cross rustc --bin alfred-pinboard-rs --target "$TARGET" --release -- -C lto

    # TODO Update this to package the right artifacts
    # res_dir="$src/res/workflow"
    res_dir="$src/res/workflow/"

    # echo "Copying executable to workflow's folder..."
    strip target/aarch64-apple-darwin/release/alfred-pinboard-rs || true
    strip target/x86_64-apple-darwin/release/alfred-pinboard-rs || true
    lipo -create -output alfred-pinboard-rs target/aarch64-apple-darwin/release/alfred-pinboard-rs target/x86_64-apple-darwin/release/alfred-pinboard-rs
    strip ./alfred-pinboard-rs || true
    chmod u+x ./alfred-pinboard-rs
    cp "alfred-pinboard-rs" "$stage"
    cp "$res_dir"/* "$stage"

    # echo "Creating the workflow bundle..."
    cd "$stage" || exit
    rm -f AlfredPinboardRust.alfredworkflow

    zip -r AlfredPinboardRust.alfredworkflow ./*

    mv ./AlfredPinboardRust.alfredworkflow "$src"/AlfredPinboardRust-"$GITHUB_REF_NAME".alfredworkflow
    cd "$src"

}

run_tests() {
    runner="$1"
    working_dir="$2"
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
    $runner config --authorization "${PINBOARD_TOKEN}"
    $runner update
    $runner config -d
    $runner search -U rust async
    ls -la "$alfred_workflow_data"
    sleep 2
    ls -la "$2"
    sleep 2
    ls -la "$2/.config"
    sleep 2
    rm -rf "$2/.config"
    sleep 2

}

src="$GITHUB_WORKSPACE"
stage=$(mktemp -d -t tmp)

echo "$GITHUB_WORKSPACE == $GITHUB_REF_NAME"
if [[ "$RELEASE_COMMIT" = "true" ]]; then
    ls -lh ./target/aarch64-apple-darwin/release/alfred-pinboard-rs
    ls -lh ./target/x86_64-apple-darwin/release/alfred-pinboard-rs
    build_alfred_bundle "$src" "$stage"
else
    lipo -create -output alfred-pinboard-rs target/aarch64-apple-darwin/debug/alfred-pinboard-rs target/x86_64-apple-darwin/debug/alfred-pinboard-rs
    strip ./alfred-pinboard-rs || true
    chmod u+x ./alfred-pinboard-rs
    run_tests ./alfred-pinboard-rs "$GITHUB_WORKSPACE"
fi

