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

echo "Building new release..."
cd "$alfred_pinboard_rs" || exit

# fix cargo version
python res/fix_cargo_version.py "$version_tag"
cp Cargo.toml Cargo.toml.back
echo "[profile.release]" >> Cargo.toml
echo "lto = true" >> Cargo.toml
cargo build --release > build.log 2>&1

echo "Copying resoursces from Alfred's workflow dir..."
cp "$workflow_dir"/* "$res_dir"

echo "Copying executable to workflow's folder..."
strip target/release/alfred-pinboard-rs
cp target/release/alfred-pinboard-rs "$res_dir"

echo "Updating version in info.plist"
# version_tag=$(git describe --tags --abbrev=0)
defaults write "$res_dir"/info.plist version "$version_tag"
plutil -convert xml1 "$res_dir"/info.plist
cp "$res_dir"/info.plist "$workflow_dir"

echo "Creating the workflow bundle..."
rm -f AlfredPinboardRust.alfredworkflow
cd "$res_dir" || exit
rm -f AlfredPinboardRust.alfredworkflow

zip -r AlfredPinboardRust.alfredworkflow ./*

echo "Moving bundle to executable folder..."
mv AlfredPinboardRust.alfredworkflow "$alfred_pinboard_rs"
rm alfred-pinboard-rs

git add res/workflow
git add res/images

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
