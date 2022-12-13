on appIsRunning(appName)
	set osver to system version of (system info)
	considering numeric strings
		set catalina to (osver >= "10.15" and osver < "10.16")
	end considering
	if catalina then
		return true
	end if
	tell application "System Events" to (name of processes) contains appName
	-- set processName to run script "tell application \"System Events\" to (name of processes)"
	-- set ret to processName contains appName
	-- return ret
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
		
	else if theApplication is "Arc.app" and appIsRunning("Arc") then
		set theResult to run script "tell application id \"company.thebrowser.Browser\"
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
		
	else if theApplication is "Opera Developer.app" and appIsRunning("Opera") then
		set theResult to run script "tell application id \"com.operasoftware.OperaDeveloper\"
        set theText to title of active tab of first window
        set theURL to get URL of active tab of first window
        return {theURL, theText}
        end tell"
		set theURL to item 1 of theResult
		set theText to item 2 of theResult
		
	else if theApplication is "Opera Beta.pp" and appIsRunning("Opera") then
		set theResult to run script "tell application id \"com.operasoftware.OperaNext\"
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
		
	else if theApplication is "Brave Browser Beta.app" and appIsRunning("Brave Browser Beta") then
		set theResult to run script "tell application id \"com.brave.Browser.beta\"
        set theText to title of active tab of first window
        set theURL to get URL of active tab of first window
        return {theURL, theText}
        end tell"
		set theURL to item 1 of theResult
		set theText to item 2 of theResult
		
	else if theApplication is "Brave Browser Nightly.app" and appIsRunning("Brave Browser Nightly") then
		set theResult to run script "tell application id \"com.brave.Browser.nightly\"
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
		
	else if theApplication is "Microsoft Edge.app" and appIsRunning("Microsoft Edge") then
		set theResult to run script "tell application id \"com.microsoft.edgemac\"
        set theText to title of active tab of first window
        set theURL to get URL of active tab of first window
        return {theURL, theText}
        end tell"
		set theURL to item 1 of theResult
		set theText to item 2 of theResult
		
	else if theApplication is "Orion.app" and appIsRunning("Orion") then
		set theResult to run script "tell application id \"com.kagi.kagimacOS\"
        set theTab to current tab of first window
	 set theUrl to URL of theTab
	 set theText to name of theTab
	 return {theUrl, theText}
	 end tell"
		set theURL to item 1 of theResult
		set theText to item 2 of theResult
		
	else if theApplication is "qutebrowser.app" and appIsRunning("qutebrowser") then
		set theResult to run script "tell application id \"org.qt-project.Qt.QtWebEngineCore\"
          activate
        end tell
        tell application \"System Events\"
          set myApp to name of first application process whose frontmost is true
        end tell
        if myApp is \"qutebrowser\" then
          tell application \"System Events\"
            key code 53 -- ESC
            delay 0.5
          end tell
          tell application \"System Events\" -- yank url
            keystroke \"y\"
            delay 0.4
            keystroke \"y\"
          end tell
          delay 0.5
          set theURL to (get the clipboard as Unicode text)

          tell application \"System Events\" -- yank title
            keystroke \"y\"
            delay 0.4
            keystroke \"t\"
          end tell
          delay 0.5
          set theTitle to (the clipboard as Unicode text)
        end if
        do shell script \"pbcopy < /dev/null\"
        return {theURL, theTitle}"
		set theURL to item 1 of theResult
		set theText to item 2 of theResult
		
	else if theApplication is "Firefox.app" and appIsRunning("Firefox") then
		set externalResult to do shell script "./get-current-url-from-alfred-firefox.sh"
		if externalResult contains "fd850fc2e63511e79f720023dfdf24ec" then
			return externalResult
		end if
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
