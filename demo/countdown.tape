Output demo/countdown.gif

# https://github.com/charmbracelet/vhs/blob/main/THEMES.md
Set Theme "Builtin Solarized Light"

Set FontSize 14
Set Width 800
Set Height 400
Set Padding 0
Set Margin 1

# --- START ---
Set LoopOffset 4
Hide
Type "cargo run -- -r -d -m c"
Enter
Sleep 0.2
Show
# --- COUNTDOWN ---
Sleep 1
Type "s"
Sleep 1.4
Type "s"
Sleep 0.3
Type "s"
Sleep 0.3
Type "e"
Sleep 0.1
Down@10ms 65
Sleep 0.1
Type "e"
Sleep 0.1
Sleep 3
