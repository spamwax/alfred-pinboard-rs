# This script takes care of testing your crate

set -ex

run_phase() {
    # if [ ! -z "$DISABLE_TESTS" ]; then
    #     return
    # fi
    case "$TARGET" in
        x86_64-apple-darwin)
            export alfred_debug=1
            export alfred_version="3.6"
            export alfred_workflow_version=0.11.1
            export alfred_workflow_uid=hamid63
            export alfred_workflow_name="RustyPin"
            export alfred_workflow_bundleid=cc.hamid.alfred-pinboard-rs
            mkdir "/Users/travis/.config/alfred-pinboard-rs"

            cross run --target "$TARGET" -- config --authorization hamid:12345
            cross run --target "$TARGET" -- config -d
            ;;
        i686-apple-darwin)
            export alfred_debug=1
            cross run --target "$TARGET" -- config --authorization hamid:12345
            export alfred_versioin=3.6
            cross run --target "$TARGET" -- config -d
            ;;
        armv7-linux-androideabi)
            export alfred_debug=1
            cross run --target "$TARGET" -- config --authorization hamid:12345
            ;;
        *)
            return
            ;;
    esac

}

# TODO This is the "test phase", tweak it as you see fit
test_phase() {

    if [ ! -z "$DISABLE_TESTS" ]; then
        cross build --target "$TARGET"
        return
    fi

    cargo test --target "$TARGET" -- --nocapture --test-threads=1 || return
    run_phase
}

# we don't run the "test phase" when doing deploys
if [ -z "$TRAVIS_TAG" ]; then
    echo "Non tag commit, running only tests."
    test_phase
fi
