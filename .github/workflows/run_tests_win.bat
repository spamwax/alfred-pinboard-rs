set executable=%1

set RUST_BACKTRACE=1
set alfred_debug=1
set alfred_version=4.0.1
set alfred_workflow_version=0.11.1
set alfred_workflow_uid=hamid63
set alfred_workflow_name=RustyPin
set alfred_workflow_bundleid="cc.hamid.alfred-pinboard-rs"
set alfred_workflow_data=%GITHUB_WORKSPACE%\data_cache
set alfred_workflow_cache=%GITHUB_WORKSPACE%\data_cache

REM set TARGET=x86_64-pc-windows-msvc
set working_dir="%GITHUB_WORKSPACE%"

dir %working_dir%
mkdir %alfred_workflow_data%

cargo build && ^
cargo run -- config --authorization hamid:12345 && ^
cargo run -- config -d && ^
cargo test -- --nocapture --test-threads=1
