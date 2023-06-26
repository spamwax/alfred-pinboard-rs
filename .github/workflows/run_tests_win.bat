set executable=%1

echo %executable%

@echo off

set RUST_BACKTRACE=1
set alfred_debug=1
set alfred_version=4.5.1
set alfred_workflow_version=0.16.1
set alfred_workflow_uid=hamid63
set alfred_workflow_name=RustyPin
set alfred_workflow_bundleid="cc.hamid.alfred-pinboard-rs"
set alfred_workflow_data=%GITHUB_WORKSPACE%\data_cache
set alfred_workflow_cache=%GITHUB_WORKSPACE%\data_cache

set working_dir="%GITHUB_WORKSPACE%"

dir %working_dir%
mkdir %alfred_workflow_data%

echo %GITHUB_REF_NAME% > tmp.txt
grep -E '[0-9]+/merge' tmp.txt

if %ERRORLEVEL% EQU 0 (
    echo "This is a pull request"
    PINBOARD_TOKEN="hamid:123456"
    %executable% config --authorization %PINBOARD_TOKEN%
    powershell -nop -c "& {sleep 4}"
 ) else (
   echo "This is not a pull request"
   %executable% config --authorization %PINBOARD_TOKEN%
   powershell -nop -c "& {sleep 4}"

   %executable% update
   powershell -nop -c "& {sleep 4}"

   %executable% search -U rust async
   powershell -nop -c "& {sleep 4}"
 )

 %executable% config --display
 powershell -nop -c "& {sleep 4}"


dir %alfred_workflow_data%
rmdir /s /q %alfred_workflow_data%
