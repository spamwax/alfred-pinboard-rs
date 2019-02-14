on appIsRunning(appName)
  tell application "System Events" to (name of processes) contains appName
end appIsRunning


on run
  set theApplication to (name of (info for (path to frontmost application)))
  set theText to ""
  set theURL to ""

  if theApplication is "Google Chrome.app" and appIsRunning("Google Chrome") then
    set theResult to run script "tell application id \"com.google.chrome\"
      set theText to title of active tab of first window
      set theURL to get URL of active tab of first window
      return {theURL, theText}
    end tell"
    set theURL to item 1 of theResult
    set theText to item 2 of theResult

  else if theApplication is "Opera.app" and appIsRunning("Opera") then
    set theResult to run script "tell application id \"com.operasoftware.Opera\"
      set theText to title of active tab of first window
      set theURL to get URL of active tab of first window
      return {theURL, theText}
    end tell"
    set theURL to item 1 of theResult
    set theText to item 2 of theResult

  else if theApplication is "Vivaldi.app" and appIsRunning("Vivaldi") then
    set theResult to run script "tell application id \"com.vivaldi.Vivaldi\"
      set theText to title of active tab of first window
      set theURL to get URL of active tab of first window
      return {theURL, theText}
    end tell"
    set theURL to item 1 of theResult
    set theText to item 2 of theResult

  else if theApplication is "Brave Browser.app" and appIsRunning("Brave Browser") then
    set theResult to run script "tell application id \"com.brave.Browser\"
      set theText to title of active tab of first window
      set theURL to get URL of active tab of first window
      return {theURL, theText}
    end tell"
    set theURL to item 1 of theResult
    set theText to item 2 of theResult

  else if theApplication is "Safari.app" and appIsRunning("Safari") then
    set theResult to run script "tell application id \"com.apple.safari\"
      set theTab to front document
      set theText to name of theTab
      set theURL to URL of theTab
      return {theURL, theText}
    end tell"
    set theURL to item 1 of theResult
    set theText to item 2 of theResult

  else if {"Safari Technology Preview.app", "SafariTechnologyPreview.app"} contains theApplication and appIsRunning("Safari Technology Preview") then
    set theResult to run script "tell application id \"com.apple.SafariTechnologyPreview\"
      set theTab to front document
      set theText to name of theTab
      set theURL to URL of theTab
      return {theURL, theText}
    end tell"
    set theURL to item 1 of theResult
    set theText to item 2 of theResult

  else if theApplication is "Chromium.app" and appIsRunning("Chromium") then
    set theResult to run script "tell application \"Chromium\"
      set theURL to URL of active tab of first window
      set theText to title of active tab of first window
      return {theURL, theText}
    end tell"
    set theURL to item 1 of theResult
    set theText to item 2 of theResult

  else if theApplication is "Firefox.app" and appIsRunning("Firefox") then
    set theResult to run script "tell application id \"org.mozilla.firefox\"
      activate
      set w to item 1 of window 1
      set theText to name of w
    end tell
    tell application \"System Events\"
      set myApp to name of first application process whose frontmost is true
      if myApp is \"Firefox\" then
        tell application \"System Events\"
          key code 97
          delay 0.5
          keystroke \"c\" using command down
        end tell
        delay 0.5
      end if
      delay 0.5
    end tell
    set theURL to get the clipboard
    return {theURL, theText}"
    set theURL to item 1 of theResult
    set theText to item 2 of theResult

  else if {"Firefox Developer Edition.app", "FirefoxDeveloperEdition.app"} contains theApplication and appIsRunning("Firefox") then
    set theResult to run script "tell application id \"org.mozilla.firefoxdeveloperedition\"
      activate
      set w to item 1 of window 1
      set theText to name of w
    end tell
    tell application \"System Events\"
      set myApp to name of first application process whose frontmost is true
      if myApp is \"Firefox\" then
        tell application \"System Events\"
          keystroke \"l\" using command down
          delay 0.5
          keystroke \"c\" using command down
        end tell
        delay 0.5
      end if
      delay 0.5
    end tell
    set theURL to get the clipboard
    return {theURL, theText}"
    set theURL to item 1 of theResult
    set theText to item 2 of theResult

  end if

  return {theURL & " fd850fc2e63511e79f720023dfdf24ec " & theText}

end run
