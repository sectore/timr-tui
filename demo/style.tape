Output demo/style.gif

# https://github.com/charmbracelet/vhs/blob/main/THEMES.md
Set Theme "OneDark"

Set FontSize 14
Set Width 1000
Set Height 500
Set Padding 0
Set Margin 1

# --- START ---
Set LoopOffset 4
Hide
Type "cargo run -- -r -d -m c"
Enter
Sleep 0.2
Type "m" # hide menu
Show
# --- STYLES ---
Sleep 0.5
Type "s"
Sleep 0.5
Type@0.7s ",,,,,,"
Sleep 1
