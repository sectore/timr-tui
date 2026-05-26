# https://github.com/charmbracelet/vhs/blob/main/THEMES.md
Set Theme "Ayu Mirage"

Set FontSize 14
Set Width 1000
Set Height 500
Set Padding 0
Set Margin 1

# --- START ---
Set LoopOffset 4
Hide
Type "cargo run -- -r -c 25:38 --blink=on"
Enter
Sleep .2
Type "m" # hide menu
Type ":::" # show local time
Show
# --- COUNTDOWN ---
Type " "
Sleep 0.5s
Screenshot demo/countdown-finish.png
Sleep 1s
