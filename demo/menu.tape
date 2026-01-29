Output demo/menu.gif

# https://github.com/charmbracelet/vhs/blob/main/THEMES.md
Set Theme "Purple Rain"

Set FontSize 14
Set Width 1000
Set Height 500
Set Padding 0
Set Margin 1

# --- START ---
Set LoopOffset 4
Hide
Type "cargo run -- -r -m c"
Enter
Type@200ms "m" # hide menu
Show
# --- STYLES ---
Sleep 0.3s
Type@0.3s "m" # show menu
Type@0.3s "2"
Type@0.3s "3"
Type@0.3s "e"
Escape@0.3s
Type@0.3s "4"
Type@0.3s "0"
