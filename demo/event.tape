Output demo/event.gif

# https://github.com/charmbracelet/vhs/blob/main/THEMES.md
Set Theme "Builtin Solarized Dark"

Set FontSize 14
Set Width 800
Set Height 400
Set Padding 0
Set Margin 1

# --- START ---
Set LoopOffset 4
Hide
Type "cargo run -- -r -e 'time=2010-01-10 10:00:00,title=hello world'"
Enter
Type "m"
Sleep 0.2
Show
# --- EVENT ---
Sleep 1
Type "e"
Backspace@1ms 17
Type@20ms "50-01-01 01:00:01"
Enter
Type "e"
Tab
Backspace@10ms 11
Type@20ms "hello future"
Enter
Sleep 1
