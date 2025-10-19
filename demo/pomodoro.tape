# Note: PR "support ctrl + arrow keys" https://github.com/charmbracelet/vhs/pull/673 needs to be merged to run this `tape`.

Output demo/pomodoro.gif

# https://github.com/charmbracelet/vhs/blob/main/THEMES.md
Set Theme "Catppuccin Frappe"

Set FontSize 14
Set Width 1000
Set Height 500
Set Padding 0
Set Margin 1

# --- START ---
Hide
Type "cd cargo run -- -r -d -m p --blink on"
Enter
Sleep .2
Type "m" # hide menu
Show
# --- POMODORO WORK ---
Sleep .5
Type "s" # start
Sleep 2.3
Type "e"
Sleep 0.2
Down@30ms 80
Sleep 100ms
Type "s" # save
Sleep 4
# --- POMODORO PAUSE ---
Ctrl+Right
Sleep 0.5
Type "s"
Sleep 2.3
Type "e"
Sleep 0.2
Down@30ms 60
Sleep 100ms
Type "s" # save
Sleep 4
