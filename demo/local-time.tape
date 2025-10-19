Output demo/local-time.gif

# https://github.com/charmbracelet/vhs/blob/main/THEMES.md
Set Theme "Atom"

Set FontSize 14
Set Width 1000
Set Height 500
Set Padding 0
Set Margin 1

# --- START ---
Set LoopOffset 4
Hide
Type "cargo run -- -m l"
Enter
Sleep .2
Type "m" # hide menu
Show
# --- toggle local time ---
Type@1s ":::"
