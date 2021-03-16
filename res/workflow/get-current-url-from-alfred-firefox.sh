#!/bin/zsh

ALFRED_FIREFOX="$(ls ../*/alfred-firefox | head -1)"
if [[ -z "$ALFRED_FIREFOX" || ! -x "$ALFRED_FIREFOX" ]] ; then
    exit
fi

eval `"$ALFRED_FIREFOX" tab-info -shell`
echo "$FF_URL" fd850fc2e63511e79f720023dfdf24ec "$FF_TITLE"
