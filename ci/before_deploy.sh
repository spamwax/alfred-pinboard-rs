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

    # only build for macOS
    case $TARGET in
        x86_64-apple-darwin)
            build_release "$src" "$stage"
            ;;
        i686-apple-darwin)
            build_release "$src" "$stage"
            ;;
        *)
            return
            ;;
    esac

}

build_release() {
    src=$1
    stage=$2
    test -f Cargo.lock || cargo generate-lockfile

    # TODO Update this to build the artifacts that matter to you
    cross rustc --bin alfred-pinboard-rs --target "$TARGET" --release -- -C lto

    # TODO Update this to package the right artifacts
    res_dir="$src/res/workflow"

    # echo "Copying executable to workflow's folder..."
    cp "$src/target/$TARGET/release/alfred-pinboard-rs" "$stage"
    cp "$res_dir"/* "$stage"

    # echo "Creating the workflow bundle..."
    cd "$stage" || exit
    strip ./alfred-pinboard-rs || true
    rm -f AlfredPinboardRust.alfredworkflow

    zip -r AlfredPinboardRust.alfredworkflow ./*

    cd "$stage"
    case $TARGET in
        x86_64-apple-darwin)
            cp ./AlfredPinboardRust.alfredworkflow "$src/AlfredPinboardRust-$TRAVIS_TAG.alfredworkflow"
            ;;
        i686-apple-darwin)
            tar czf "$src/$CRATE_NAME-$TRAVIS_TAG-$TARGET.tar.gz" ./AlfredPinboardRust.alfredworkflow
            ;;
        *)
            return
            ;;
    esac
    cd "$src"

    rm -rf "$stage"

}

main
