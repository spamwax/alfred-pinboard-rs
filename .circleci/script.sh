# This script takes care of testing your crate

set -ex

run_tests() {
    runner="$1"
    working_dir="$2"
    # runner="cargo run --target "$TARGET" --"
    export alfred_debug=1
    export alfred_version="4.0.1"
    export alfred_workflow_version=0.15.1
    export alfred_workflow_uid=hamid63
    export alfred_workflow_name="RustyPin"
    export alfred_workflow_bundleid=cc.hamid.alfred-pinboard-rs
    mkdir -p "$working_dir"/.config/alfred-pinboard-rs
    export alfred_workflow_data="$working_dir"
    export alfred_workflow_cache="$working_dir"
    case "$TARGET" in
        x86_64-apple-darwin)
            $runner config --authorization hamid:12345
            $runner config -d
            ;;
        x86_64-unknown-linux-gnu)
            ls -ld "$alfred_workflow_data"
            ls -ld "$alfred_workflow_cache"
            chown -R "$USER":"$USER" "$alfred_workflow_data"

            $runner config --authorization hamid:12345
            $runner config -d
            ;;
        i686-apple-darwin)
            $runner config --authorization hamid:12345
            $runner config -d
            ;;
        x86_64-unknown-freebsd)
            # $runner config --authorization hamid:12345
            # $runner config -d
            ;;
        armv7-linux-androideabi)
            $runner config --authorization hamid:12345
            $runner config -d
            ;;
        *)
            return
            ;;
    esac

}

# # TODO This is the "test phase", tweak it as you see fit
# test_phase() {

#     # rustup target add --toolchain stable x86_64-unknown-linux-gnu
#     if [ -n "$DISABLE_TESTS" ] || [ -z "$CIRCLE_TEST" ]; then
#         cargo build --target "$TARGET"
#         return
#     fi

#     cargo test --target "$TARGET" -- --nocapture --test-threads=1 || return
#     run_phase
# }

# we don't run debug builds & tests whena tag is assigned.
# if [ -n "$CIRCLE_TAG" ]; then
#     echo "Tag commit, Not building in debug mode."
#     echo "Refuse to do anything"
#     exit 0
# fi

# if [ -z "$CIRCLE_TEST" ] || [ -n "$DISABLE_TESTS" ]; then
# Build only
if [ -z "$CIRCLE_TEST" ]; then
    arg=
    [[ "$TARGET" == "x86_64-apple-darwin" ]] && [[ "$BUILD_TYPE" == "release" ]] && arg="--release"
    cargo build $arg --target "$TARGET"
elif [[ "$CIRCLE_TEST" == "false" ]]; then # Tests disabled
    echo "Tests Disabled. Finishing the job."
# Test only
elif [[ "$CIRCLE_TEST" == "true" ]]; then
    echo "$1"
    run_tests "$1" "$2"
fi
