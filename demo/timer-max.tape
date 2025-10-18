# Output demo/timer-max.gif

# https://github.com/charmbracelet/vhs/blob/main/THEMES.md
Set Theme "SeaShells"

Set FontSize 14
Set Width 1000
Set Height 500
Set Padding 0
Set Margin 1

# --- START ---
Type 'cargo run -- -r -m t'
Enter
Type "m"
Type "e"
Up@1ms 60 # ss
Left
Up@1ms 60 # mm
Left
Up@1ms 23 # hh
Left
Up@1ms 363 # ddd
Left
Up@1ms 9999 # yyyy
Right 4
Down # ss
Left
Down ## mm
Left 2
Down ## ddd
Up 2
Type "."
Type "s" # save
Type "s" # start to reach DONE
Sleep 2s
# --- SCREENSHOT ---
Screenshot demo/timer-max.png
Sleep 1s
