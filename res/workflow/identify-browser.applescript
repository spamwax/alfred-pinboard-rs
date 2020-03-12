on appIsRunning(appName)
	tell application "System Events" to (name of processes) contains appName
end appIsRunning


on run
    delay 2
	set theApplication to (name of (info for (path to frontmost application)))
	set supportedBrowser to true

	if theApplication is "qutebrowser.app" and appIsRunning("qutebrowser") then
        set supportedBrowser to false

	else if theApplication is "Firefox.app" and appIsRunning("Firefox") then
        set supportedBrowser to false

	else if {"Firefox Developer Edition.app", "FirefoxDeveloperEdition.app"} contains theApplication and appIsRunning("Firefox") then
        set supportedBrowser to false

	end if

	return supportedBrowser

end run
