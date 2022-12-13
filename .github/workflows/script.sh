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
    res_dir="$src/res/workflow"

    # echo "Copying executable to workflow's folder..."
    cp "alfred-pinboard-rs" "$stage"
    cp "$res_dir"/* "$stage"

    # echo "Creating the workflow bundle..."
    cd "$stage" || exit
    rm -f AlfredPinboardRust.alfredworkflow

    zip -r AlfredPinboardRust.alfredworkflow ./*

    mv ./AlfredPinboardRust.alfredworkflow "$src"/AlfredPinboardRust-"$GITHUB_REF_NAME".alfredworkflow
    cd "$src"

}

src="$GITHUB_WORKSPACE"
stage=$(mktemp -d -t tmp)

echo "$GITHUB_WORKSPACE == $GITHUB_REF_NAME"
if [[ "$RELEASE_COMMIT" == "true" ]]; then
  build_type=release
else
  build_type=debug
fi
ls -lh ./target/aarch64-apple-darwin/"$build_type"/alfred-pinboard-rs
ls -lh ./target/x86_64-apple-darwin/"$build_type"/alfred-pinboard-rs

strip target/aarch64-apple-darwin/"$build_type"/alfred-pinboard-rs || true
strip target/x86_64-apple-darwin/"$build_type"/alfred-pinboard-rs || true
lipo -create -output alfred-pinboard-rs target/aarch64-apple-darwin/"$build_type"/alfred-pinboard-rs target/x86_64-apple-darwin/"$build_type"/alfred-pinboard-rs
strip ./alfred-pinboard-rs || true
chmod u+x ./alfred-pinboard-rs
if [[ "$RELEASE_COMMIT" == "true" ]]; then
  build_alfred_bundle "$src" "$stage"
else
  .github/workflows/run_tests.sh ./alfred-pinboard-rs "$PINBOARD_TOKEN"
fi
