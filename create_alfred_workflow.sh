#!/bin/bash

alfred_pinboard_rs=`pwd`
workflow_dir="$HOME/Dropbox/Alfred/Alfred.alfredpreferences/workflows/user.workflow.665EAB20-5141-463D-8C5A-90093EEAA756"

echo "Building new release..."
cd "$alfred_pinboard_rs" || exit
cargo build --release > build.log 2>&1

echo "Copying executable to workflow's folder..."
cp target/release/alfred-pinboard-rs "$workflow_dir"

echo "Creating the workflow bundle..."
cd "$workflow_dir" || exit
rm -f AlfredPinboardRust.alfredworkflow

zip -r AlfredPinboardRust.alfredworkflow ./* || exit

echo "Moving bundle to executable folder..."
mv AlfredPinboardRust.alfredworkflow "$alfred_pinboard_rs"
