set executable=%1

echo %executable%

Rem Turns the echo off so that each command will not be shown when executed
@echo off
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

echo on

dir %working_dir%
mkdir %alfred_workflow_data%


%executable% config --display
powershell -nop -c "& {sleep 4}"

%executable% config --authorization %PINBOARD_TOKEN%
powershell -nop -c "& {sleep 4}"

%executable% update
powershell -nop -c "& {sleep 4}"

%executable% search -U rust async
powershell -nop -c "& {sleep 4}"

dir %alfred_workflow_data%
rmdir /s /q %alfred_workflow_data%


echo %GITHUB_REF_NAME% > tmp.txt
grep -E '[0-9]+/merge' tmp.txt

echo "23/merge" > tmp.txt
grep -E '[0-9]+/merge' tmp.txt
