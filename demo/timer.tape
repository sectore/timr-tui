Output demo/timer.gif

# https://github.com/charmbracelet/vhs/blob/main/THEMES.md
Set Theme "Belafonte Day"

Set FontSize 14
Set Width 1000
Set Height 500
Set Padding 0
Set Margin 1

# --- START ---
Set LoopOffset 4
Hide
Type "cargo run -- -r -d -m t"
Enter
Sleep 0.2
Type "m" # hide menu
Show
# --- TIMER ---
Type "s"
Sleep 1.4
Type "s"
Sleep 0.3
Type "s"
Sleep 0.3
Type "e"
Sleep 0.2
Up@30ms 57
Sleep 0.7
Type "s"
Sleep 4
