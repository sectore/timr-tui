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
Type "cargo run -- -r -m p"
Enter
Sleep 0.2
Show
# --- STYLES ---
Sleep 0.5
Type "m"
Sleep 0.5
Type@0.5s "t"
Type@0.5s "c"
Type@0.5s "p"
Type@0.5s "e"
Right@0.5s
Left@0.5s
Type@0.5s "e"
Sleep 0.5
