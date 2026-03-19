-- Click at screen coordinates using System Events.
--
-- Usage: osascript lib/click.applescript <x> <y>
-- Example: osascript lib/click.applescript 500 300

on run argv
	if (count of argv) < 2 then
		error "Usage: click.applescript <x> <y>"
	end if

	set x to item 1 of argv as integer
	set y to item 2 of argv as integer

	tell application "System Events"
		-- Bring the frontmost app to focus
		set frontApp to name of first application process whose frontmost is true
		tell application process frontApp
			-- Use AppleScript coordinate click
			click at {x, y}
		end tell
	end tell

	return "clicked at {" & x & ", " & y & "}"
end run
