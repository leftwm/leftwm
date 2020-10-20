# How to control LeftWM from an external proccess

LeftWM supports being controlled using EWMH calls and from an external pipe
file.

This folder has example of how to send commands to LeftWM using its external pipe
file.

A full list of supported commands can be found here:

```
LoadTheme PATH_TO_THEME
UnloadTheme
Reload
SendWorkspaceToTag INDEX_OF_WORKSPACE, INDEX_OF_TAG
SendWindowToTag INDEX_OF_TAG
SwapScreens
MoveWindowToLastWorkspace
MoveWindowUp
MoveWindowDown
FocusWindowUp
FocusWindowDown
FocusWorkspaceNext
FocusWorkspacePrevious
CloseWindow
NextLayout
PreviousLayout
```


