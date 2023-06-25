set executable=%1

echo %executable%

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

%executable% config --authorization hamid:12345
%executable% config --display

cargo test -- --nocapture --test-threads=1

%executable% config --authorization %PINBOARD_TOKEN%
%executable% update
powershell -nop -c "& {sleep 2}"
%executable% search -U rust async

dir %alfred_workflow_data%
dir %alfred_workflow_cache%
rmdir /s /q %alfred_workflow_data%


echo %GITHUB_REF_NAME% > tmp.txt

Rem Turns the echo off so that each command will not be shown when executed
@echo off
echo "Hello World"

Rem Displays the contents of the PATH variable
echo %PATH%

Rem Turns the echo on so that each command will be shown as executed
echo on
echo "ON Hello World"

