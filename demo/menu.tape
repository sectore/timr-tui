Output demo/menu.gif

# https://github.com/charmbracelet/vhs/blob/main/THEMES.md
Set Theme "Apple Classic"

Set FontSize 14
Set Width 800
Set Height 400
Set Padding 0
Set Margin 1

# --- START ---
Set LoopOffset 4
Hide
Type "cargo run -- -r -m p --menu"
Enter
Type@200ms "m"
Show
# --- STYLES ---
Sleep 0.3s
Type@0.3s "m"
Type@0.3s "t"
Type@0.3s "c"
Type@0.3s "p"
Type@0.3s "e"
Escape@0.3s
