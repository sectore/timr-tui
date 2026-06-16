# Note: PR "support ctrl + arrow keys" https://github.com/charmbracelet/vhs/pull/673 needs to be merged to run this `tape`.

Output demo/pomodoro_auto_special.gif

# https://github.com/charmbracelet/vhs/blob/main/THEMES.md
Set Theme "Catppuccin Frappe"

Set FontSize 14
Set Width 1000
Set Height 500
Set Padding 0
Set Margin 1

# --- START ---
Hide
Type "cargo run -- -r -m p -d -w 20:00 -p '3:00,30:00,4' --blink on"
Enter
Sleep .2
Type "m" # hide menu
Type "a" # enable auto-switch
Up 3 # count to round 4
Show
Type "e"
Down@10ms 19
Right
Down@10ms 59
Type "s" # save
Type " " # start
Sleep 4 # wait for auto-switch work -> pause
Type "e"
Down@10ms 86
Type "s" # save
Sleep 6
