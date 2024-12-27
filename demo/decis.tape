Output demo/decis.gif

# https://github.com/charmbracelet/vhs/blob/main/THEMES.md
Set Theme "nord-light"

Set FontSize 14
Set Width 800
Set Height 400
Set Padding 0
Set Margin 1

# --- START ---
Set LoopOffset 4
Hide
Type "cargo run -- -r -m t"
Enter
Sleep 0.2
Show
# --- STYLES ---
Type "s"
Sleep 0.2
Type@0.4s "......."
Sleep 0.2
