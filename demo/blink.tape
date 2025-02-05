Output demo/blink.gif

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
# countdown 1.0s
Type "cargo run -- -r -m countdown -d -c 1 --blink=on"
Enter
Sleep 0.2
Type "m"
Type ":::"
Show
Type "s"
Sleep 4
