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
    workflow_dir="$working_dir/.config/alfred-pinboard-rs"
    mkdir -p "$workflow_dir"
    export alfred_workflow_data="$workflow_dir"
    export alfred_workflow_cache="$workflow_dir"
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
            unset alfred_debug
            $runner config -d | .circleci/json_pretty.sh
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
