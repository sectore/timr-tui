# Note: PR "support ctrl + arrow keys" https://github.com/charmbracelet/vhs/pull/673 needs to be merged to run this `tape`.

Output demo/tabata.gif

# https://github.com/charmbracelet/vhs/blob/main/THEMES.md
# Set Theme "AdventureTime"
Set Theme "AlienBlood"

Set FontSize 14
Set Width 1000
Set Height 500
Set Padding 0
Set Margin 1

# --- START ---
Hide
Type "cargo run -- -r -d -m p --blink on --tabata"
Enter
Sleep .2
Type "m" # hide menu
Show
# --- SCREENSHOT ---
Screenshot demo/tabata.png
Sleep 1s
