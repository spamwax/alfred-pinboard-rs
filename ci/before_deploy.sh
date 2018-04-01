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
    # cross rustc --bin hello --target $TARGET --release -- -C lto

    # TODO Update this to package the right artifacts
    create_workflow "$stage"

    cd "$stage"
    tar czf "$src/$CRATE_NAME-$TRAVIS_TAG-$TARGET.tar.gz" ./*
    cd "$src"

    rm -rf "$stage"
}

create_workflow() {
    alfred_pinboard_rs=$(pwd)
    workflow_dir="$HOME/Dropbox/Alfred/Alfred.alfredpreferences/workflows/user.workflow.665EAB20-5141-463D-8C5A-90093EEAA756"

    # echo "Building new release..."
    cargo build --release > build.log 2>&1

    # echo "Copying executable to workflow's folder..."
    cp target/release/alfred-pinboard-rs "$workflow_dir"

    # echo "Creating the workflow bundle..."
    cd "$workflow_dir" || exit
    rm -f AlfredPinboardRust.alfredworkflow

    zip -r AlfredPinboardRust.alfredworkflow ./* || exit

    # echo "Moving bundle to executable folder..."
    mv AlfredPinboardRust.alfredworkflow "$1"
    cd "$alfred_pinboard_rs"
}

main
