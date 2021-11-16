#!/bin/bash

# set -x
version_tag=$1
msg=$2
push_it=$3

if [ -z "$version_tag" ]; then
    echo "You need to provide a semver tag: v0.9.10"
    exit
fi

alfred_pinboard_rs="/Volumes/Home/hamid/src/learn/rust/alfred-pinboard-rs"
workflow_dir="$HOME/Dropbox/Alfred/Alfred.alfredpreferences/workflows/user.workflow.BE0BAA6E-408E-40A0-AEF2-922BFAA8BBF0"
res_dir="$alfred_pinboard_rs/res/workflow"

git checkout master || exit
cargo update || exit
cargo generate-lockfile

# Run clippy
if ! cargo +nightly clippy; then
    exit
fi

echo "Building new release..."
cd "$alfred_pinboard_rs" || exit

# Bump Cargo.toml version
echo "Bumping Cargo.toml version to $version_tag"
python res/fix_cargo_version.py "$version_tag"
cargo build --release > build.log 2>&1

echo "Copying resoursces from Alfred's workflow dir..."
cp "$workflow_dir"/* "$res_dir" || exit

echo "Copying executable to workflow's folder..."
strip target/release/alfred-pinboard-rs
cp target/release/alfred-pinboard-rs "$res_dir"

echo "Updating version in info.plist"
# version_tag=$(git describe --tags --abbrev=0)
defaults write "$res_dir"/info.plist version "$version_tag"
plutil -convert xml1 "$res_dir"/info.plist
cp "$res_dir"/info.plist "$workflow_dir" || exit

echo "Creating the workflow bundle..."
rm -f AlfredPinboardRust.alfredworkflow
cd "$res_dir" || exit
rm -f AlfredPinboardRust.alfredworkflow

zip -r AlfredPinboardRust.alfredworkflow ./*

echo "Moving bundle to executable folder..."
mv AlfredPinboardRust.alfredworkflow "$alfred_pinboard_rs"
rm alfred-pinboard-rs

cd "$alfred_pinboard_rs" || exit
git add .github
git add res/workflow
git add res/images
git add Cargo.toml
git add Cargo.lock
git add CHANGELOG.md
git add README.md
git add create_alfred_workflow.sh
git add src

commit_msg="Release version $version_tag"
[ -n "$msg" ] && commit_msg="$commit_msg

$msg"
git pull origin master
git commit -a -m "$commit_msg"
git tag "$version_tag"

if [ -n "$push_it" ]; then
    git push
    sleep 5
    git push --tags
fi
