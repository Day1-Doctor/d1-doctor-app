-- Launch the Day1 Copilot application and wait for it to become responsive.
-- Returns "launched" on success.
--
-- Usage: osascript lib/launch.applescript

on run
	-- Try multiple app name variants
	try
		tell application "d1-copilot" to activate
		delay 3
		return "launched"
	on error
		try
			tell application "d1_copilot" to activate
			delay 3
			return "launched"
		on error
			try
				tell application "Day1 Copilot" to activate
				delay 3
				return "launched"
			on error errMsg
				return "error: " & errMsg
			end try
		end try
	end try
end run
