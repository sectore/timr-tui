Output demo/countdown.gif

# https://github.com/charmbracelet/vhs/blob/main/THEMES.md
Set Theme "iceberg-light"

Set FontSize 14
Set Width 1000
Set Height 500
Set Padding 0
Set Margin 1

# --- START ---
Set LoopOffset 4
Hide
Type "cargo run -- -r -d -c 10:00"
Enter
Sleep .2
Type "m" # hide menu
Show
# --- COUNTDOWN ---
Sleep .5
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
Type "s"
Sleep 3
