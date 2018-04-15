# This script takes care of testing your crate

set -ex

run_phase() {
    # if [ ! -z "$DISABLE_TESTS" ]; then
    #     return
    # fi
    case "$TARGET" in
        x86_64-unknown-linux-gnu)
            cargo test --target "$TARGET" -- --nocapture --test-threads=1
            ;;
        *)
            export alfred_debug=1
            cross run --target "$TARGET" -- config --authorization hamid:12345
            export alfred_versioin=3.6
            cross run --target "$TARGET" -- config -d
    esac

}

# TODO This is the "test phase", tweak it as you see fit
test_phase() {

    if [ ! -z "$DISABLE_TESTS" ]; then
        cross build --target "$TARGET"
        return
    fi

    # cross test --target $TARGET
    export alfred_debug=1
    cross test --target "$TARGET" --release -- --test-threads=1 || return

    # only run for macOS
    case $TARGET in
        x86_64-apple-darwin)
            run_phase
            ;;
        i686-apple-darwin)
            run_phase
            ;;
        armv7-linux-androideabi)
            run_phase
            ;;
        *)
            return
            ;;
    esac
}

# we don't run the "test phase" when doing deploys
if [ -z "$TRAVIS_TAG" ]; then
    test_phase
fi
