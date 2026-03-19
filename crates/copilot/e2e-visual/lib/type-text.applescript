-- Type text into the frontmost application using System Events.
--
-- Usage: osascript lib/type-text.applescript "Hello, world!"

on run argv
	if (count of argv) < 1 then
		error "Usage: type-text.applescript <text>"
	end if

	set textToType to item 1 of argv

	tell application "System Events"
		keystroke textToType
		delay 0.3
	end tell

	return "typed: " & textToType
end run
