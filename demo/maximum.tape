Output demo/maximum.gif

# https://github.com/charmbracelet/vhs/blob/main/THEMES.md
Set Theme "C64"

Set FontSize 14
Set Width 1000
Set Height 500
Set Padding 0
Set Margin 1

# --- START ---
Set LoopOffset 4
Hide
Type "cargo run -- -r -m timer"
Enter
Sleep 0.2
Type "m"
Type "e"
# secs
Up@1ms 60
Left
# mins
Up@1ms 59
Left
# hours
Up@1ms 23
Left
# days
Up@1ms 364
Right@1ms 3
Down@1ms 1
Left@1ms 4
# years
Up@1ms 998
Right
# days
Up@1ms 365
Type@1ms "s"
Show
Sleep 0.1
