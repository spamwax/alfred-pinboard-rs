# This script takes care of testing your crate

set -ex

run_phase() {
    # if [ ! -z "$DISABLE_TESTS" ]; then
    #     return
    # fi
    export alfred_debug=1
    export alfred_version="4.0.1"
    export alfred_workflow_version=0.15.1
    export alfred_workflow_uid=hamid63
    export alfred_workflow_name="RustyPin"
    export alfred_workflow_bundleid=cc.hamid.alfred-pinboard-rs
    mkdir -p /home/circleci/.config/alfred-pinboard-rs
    case "$TARGET" in
        x86_64-apple-darwin)
            export alfred_workflow_data=/home/circleci/.config/alfred-pinboard-rs
            export alfred_workflow_cache=/home/circleci/.config/alfred-pinboard-rs
            cross run --target "$TARGET" -- config --authorization hamid:12345
            cross run --target "$TARGET" -- config -d
            ;;
        x86_64-unknown-linux-gnu)
            mkdir -p "$HOME/.config/alfred-pinboard-rs"
            export alfred_workflow_data=$HOME/.config/alfred-pinboard-rs
            export alfred_workflow_cache=$HOME/.config/alfred-pinboard-rs
            export RUST_BACKTRACE=1
            ls -ld "$alfred_workflow_data"
            ls -ld "$alfred_workflow_cache"
            chown -R "$USER":"$USER" "$alfred_workflow_data"

            cross run --target "$TARGET" -- config --authorization hamid:12345
            cross run --target "$TARGET" -- config -d
            ;;
        i686-apple-darwin)
            mkdir "$HOME/.config/alfred-pinboard-rs"
            export alfred_workflow_data=$HOME/.config/alfred-pinboard-rs
            export alfred_workflow_cache=$HOME/.config/alfred-pinboard-rs
            cross run --target "$TARGET" -- config --authorization hamid:12345
            cross run --target "$TARGET" -- config -d
            ;;
        x86_64-unknown-freebsd)
            # mkdir "$HOME/.config/alfred-pinboard-rs"
            # export alfred_workflow_data=$HOME/.config/alfred-pinboard-rs
            # export alfred_workflow_cache=$HOME/.config/alfred-pinboard-rs
            # cross run --target "$TARGET" -- config --authorization hamid:12345
            # cross run --target "$TARGET" -- config -d
            ;;
        armv7-linux-androideabi)
            mkdir "$HOME/.config/alfred-pinboard-rs"
            export alfred_workflow_data=$HOME/.config/alfred-pinboard-rs
            export alfred_workflow_cache=$HOME/.config/alfred-pinboard-rs
            cross run --target "$TARGET" -- config --authorization hamid:12345
            ;;
        *)
            return
            ;;
    esac

}

# TODO This is the "test phase", tweak it as you see fit
test_phase() {

    rustup target add --toolchain stable x86_64-unknown-freebsd
    if [ -n "$DISABLE_TESTS" ] || [ -z "$CIRCLECI_TEST" ]; then
        cross build --target "$TARGET"
        return
    fi

    cargo test --target "$TARGET" -- --nocapture --test-threads=1 || return
    run_phase
}

# we don't run the "test phase" when doing deploys
if [ -z "$CIRCLECI_TAG" ]; then
    echo "Non tag commit, running only tests."
    test_phase
fi
