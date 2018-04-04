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
    cross rustc --bin alfred-pinboard-rs --target "$TARGET" --release -- -C lto

    # TODO Update this to package the right artifacts
    res_dir="$src/res/workflow"

    # echo "Copying executable to workflow's folder..."
    ls -l "$src" || echo
    ls -l "$src/target/$TARGET"  || echo
    ls -l "$src/$TARGET"  || echo
    ls -l "$src/target/$TARGET"/relaese  || echo
    ls -l "$src/$TARGET"/relaese  || echo
    cp -l "$src/$TARGET/release/alfred-pinboard-rs" "$stage"  || echo
    cp -l "$res_dir"/* "$stage"  || echo

    # echo "Creating the workflow bundle..."
    cd "$stage" || exit
    rm -f AlfredPinboardRust.alfredworkflow  || echo

    zip -r AlfredPinboardRust.alfredworkflow ./*  || echo

    cd "$stage"
    tar czf "$src/$CRATE_NAME-$TRAVIS_TAG-$TARGET.tar.gz" ./AlfredPinboardRust.alfredworkflow  || echo
    cd "$src"

    rm -rf "$stage"
}

main
