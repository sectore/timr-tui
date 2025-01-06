Output demo/local-time.gif

# https://github.com/charmbracelet/vhs/blob/main/THEMES.md
Set Theme "AtomOneLight"

Set FontSize 14
Set Width 800
Set Height 400
Set Padding 0
Set Margin 1

# --- START ---
Set LoopOffset 4
Hide
Type "cargo run -- -m c"
Enter
Sleep 0.2
Show
Sleep 1
# --- toggle local time ---
Type@1.5s ":::"
Sleep 1.5
