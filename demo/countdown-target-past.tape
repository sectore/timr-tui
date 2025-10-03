Output demo/countdown-target-past.gif

# https://github.com/charmbracelet/vhs/blob/main/THEMES.md
Set Theme "seoulbones_light"

Set FontSize 14
Set Width 1000
Set Height 500
Set Padding 0
Set Margin 1

# --- START ---
Set LoopOffset 4
Hide
Type "cargo run -- -r -m c --ct '2024-01-10 18:00'"
Enter
Type "m"
Sleep 0.2
Show
Sleep 0.1
