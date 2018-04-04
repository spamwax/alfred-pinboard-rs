# This script takes care of building your crate and packaging it for release

set -ex

main() {
    local src=$(pwd) \
          stage=

    case $TRAVIS_OS_NAME in
        linux)
            stage=$(mktemp -d)
            ;;
        osx)
            stage=$(mktemp -d -t tmp)
            ;;
    esac

    test -f Cargo.lock || cargo generate-lockfile

    # TODO Update this to build the artifacts that matter to you
    cross rustc --target "$TARGET" --release -- -C -lto

    # TODO Update this to package the right artifacts
    create_workflow "$stage"

    cd "$stage"
    tar czf "$src/$CRATE_NAME-$TRAVIS_TAG-$TARGET.tar.gz" ./AlfredPinboardRust.alfredworkflow
    cd "$src"

    rm -rf "$stage"
}

create_workflow() {
    res_dir="res/workflow"

    # echo "Copying executable to workflow's folder..."
    cp target/release/alfred-pinboard-rs "$1"
    cp "$res_dir"/* "$1"

    # echo "Creating the workflow bundle..."
    cd "$1" || exit
    rm -f AlfredPinboardRust.alfredworkflow

    zip -r AlfredPinboardRust.alfredworkflow ./*
}

main
