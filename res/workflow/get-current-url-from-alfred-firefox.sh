#!/bin/zsh
#
# Get the current URL from Firefox via the alfred-firefox workflow
#
# If that workflow is not installed, exits with no output;
# then we fallback to grabbing the URL via clipboard

ALFRED_FIREFOX="$(ls ../*/alfred-firefox | head -1)"
if [[ -z "$ALFRED_FIREFOX" || ! -x "$ALFRED_FIREFOX" ]] ; then
    exit
fi

eval `"$ALFRED_FIREFOX" tab-info -shell`

if [[ -z "$FF_URL" ]] ; then
    exit
fi

echo "$FF_URL" fd850fc2e63511e79f720023dfdf24ec "$FF_TITLE"
