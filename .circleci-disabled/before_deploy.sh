#!/bin/bash
# This script takes care of building your crate and packaging it for release

set -ex

main() {
    local src=$(pwd) \
          stage=

    case $CIRCLE_OS_NAME in
        linux)
            stage=$(mktemp -d)
            ;;
        macos)
            stage=$(mktemp -d -t tmp)
            ;;
        *)
            echo "$CIRCLE_OS_NAME not a supported OS"
            exit 1
    esac

    # only build for macOS
    case $TARGET in
        x86_64-apple-darwin)
            build_alfred_bundle "$src" "$stage"
            ;;
        # i686-apple-darwin)
            # build_alfred_bundle "$src" "$stage"
            # ;;
        *)
            return
            ;;
    esac

}

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
    cp "$src/target/$TARGET/release/alfred-pinboard-rs" "$stage"
    cp "$res_dir"/* "$stage"

    # echo "Creating the workflow bundle..."
    cd "$stage" || exit
    strip ./alfred-pinboard-rs || true
    rm -f AlfredPinboardRust.alfredworkflow

    zip -r AlfredPinboardRust.alfredworkflow ./*

    case $TARGET in
        x86_64-apple-darwin)
            mv ./AlfredPinboardRust.alfredworkflow "$src/target/alfred-pinboard-rust-$CIRCLE_TAG.alfredworkflow"
            ;;
        i686-apple-darwin)
            tar czf "$src/$TARGET-$CRATE_NAME-$CIRCLE_TAG.tar.gz" ./AlfredPinboardRust.alfredworkflow
            ;;
        *)
            return
            ;;
    esac
    cd "$src"

}

if [ -z "$CIRCLE_TAG" ]; then
    echo "Not a tagged commit. Exitting"
    exit 1
else
    echo "This is a tagged commit, running before_deploy"
fi

main
