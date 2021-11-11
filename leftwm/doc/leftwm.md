% leftwm(1) version git | leftwm manual page


# leftwm(1) Manual Page

## NAME

LeftWM - A tiling window manager for Adventurers.



For comprehensive documentation of leftwm please see: https://github.com/leftwm/leftwm/wiki

## SYNOPSIS

**leftwm** [SUBCOMMAND ...]

## Description

Starts the left window manager on $DISPLAY. This also boots `leftwm-worker` and logs for errors to the console. The list of available options for `leftwm` is listed below:

SUBCOMMAND can be:

**-h, --help**: Prints help information.

**-v, --version**: Prints version information.

**check**: Checks the syntax of the configuration file

**command**: Sends external commands to `leftwm`

**state**: Prints the current state of leftwm (in JSON format)

**theme**: Manage leftwm themes



# Configuring

The settings file to change key bindings and the default mod key can be found at

```
$HOME/.config/leftwm/config.toml
```

the file is automatically generated when leftwm or leftwm-check is run for the first time.



## Default keybinds

**up, down, left or right refer to the arrow keys in your keyboard**

| Keybinding                    | Description                                                  |
| ----------------------------- | ------------------------------------------------------------ |
| Mod + (1-9)                   | Switch to a desktop/tag                                      |
| Mod + Shift + (1-9)           | Move the focused window to desktop/tag                       |
| Mod + W                       | Switch the desktops for each screen. Desktops [1][2] changes to [2][1] |
| Mod + Shift + W               | Move window to the other desktop                             |
| Mod + (up or down)            | Focus on the different windows in the current workspace      |
| Mod + Shift + (up or down)    | Move the different windows in the current workspace          |
| Mod + Enter                   | Move selected window to the top of the stack in the current workspace |
| Mod + Ctrl + (up or down)     | Switch between different layouts                             |
| Mod + Shift + (left or right) | Switch between different workspaces                          |
| Mod + Shift + Enter           | Open a terminal                                              |
| Mod + Ctrl + L                | Lock the screen                                              |
| Mod + Shift + X               | Exit LeftWM                                                  |
| Mod + Shift + Q               | Close the current window                                     |
| Mod + Shift + R               | Reload LeftWM and its config                                 |
| Mod + p                       | Use dmenu to start application                               |



## Floating Windows

You can optionally switch between tiling or floating mode for any window.

| Keybinding              | Description                             |
| ----------------------- | --------------------------------------- |
| Mod + MouseDrag         | Switch a tiled window to floating mode  |
| Mod + RightMouseDrag    | Resize a window                         |
| Drag window onto a tile | Switch a floating window to tiling mode |
| Mod + Shift + (1-9)     | Switch a floating window to tiling mode |

## 

## External Commands

This is a brief overview of the available external commands and their possible arguments.

Generally you pass the string of the external command to `$XDG_RUNTIME_DIR/leftwm/command.pipe`. For example from a shell you could use:

```
echo "SetLayout CenterMain" > $XDG_RUNTIME_DIR/leftwm/command.pipe
```

If you are on the leftwm 0.2.8 or above, external commands can be passed in using leftwm-command. Commands that contain arguments require quotes. For example:

```
leftwm-command "SetLayout CenterMain"
```

Commands can also be chained together, for example:

```
leftwm-command "SendWindowToTag 2" "SendWorkspaceToTag 0 2"
```

| Command                       | Arguments (if needed)         | Notes                                                        |
| ----------------------------- | ----------------------------- | ------------------------------------------------------------ |
| Reload                        |                               | Reloads leftwm                                               |
| LoadTheme                     | `Path-to/theme.toml`          | usually used in themes `up` script to load a theme           |
| UnloadTheme                   |                               | usually used in themes `down` script to unload the theme     |
| SetLayout                     | `LayoutName`                  |                                                              |
| NextLayout                    |                               |                                                              |
| PreviousLayout                |                               |                                                              |
| RotateTag                     |                               |                                                              |
| SetMarginMultiplier           | `multiplier as float`         | set a factor by which the margin gets multiplied, use "1.0" to reset, negative values will be abs-converted |
| SwapScreen                    |                               | swaps two screens/workspaces                                 |
| SendWorkspaceToTag            | `workspace index` `tag_index` | both indices as integer, focuses `Tag` on `Workspace`        |
| SendWindowToTag               | `tag_index`                   | index as integer, sends currently focused window to `Tag`    |
| MoveWindowToLastWorkspace     |                               | moves currently focused window to last used workspace        |
| MoveWindowToNextWorkspace     |                               | moves currently focused window to next workspace             |
| MoveWindowToPreviousWorkspace |                               | moves currently focused window to previous workspace         |
| MoveWindowDown                |                               | moves currently focused window down once                     |
| MoveWindowUp                  |                               | moves currently focused window up once                       |
| MoveWindowTop                 |                               | moves currently focused window to the top                    |
| FloatingToTile                |                               | pushes currently focused floating window back to tiling mode |
| TileToFloating                |                               | Switch currently focused tiled window to floating mode       |
| ToggleFloating                |                               | Switch currently focused window between tiled and floating mode |
| CloseWindow                   |                               | closes currently focused window                              |
| FocusWindowDown               |                               |                                                              |
| FocusWindowUp                 |                               |                                                              |
| FocusNextTag                  |                               |                                                              |
| FocusPreviousTag              |                               |                                                              |
| FocusWorkspaceNext            |                               |                                                              |
| FocusWorkspacePrevious        |                               |                                                              |
| ToggleFullScreen              |                               | Makes currently focused window fullscreen/non-fullscreen     |



## Configuration

All entries require a modifier, even if blank: `modifier = []`

***Important: You will need to reload (recommended SoftReload, as it tries to preserve the WM state as far as possible) in order to apply changes to `config.toml`.***

### Terms

- *Default* refers to the original `config.toml` specified when LeftWM first runs.
- *Partial Default* refers to a command that is in the original `config.toml` but is not the only instance of that command.
- *Example* refers to a snippet that is not in the original `config.toml` but can be added or modified for additional features.

### Modkey

The modkey is the most important setting. It is used by many other settings and controls how key bindings work. For more info please read [this](https://stackoverflow.com/questions/19376338/xcb-keyboard-button-masks-meaning) post on x11 Mod keys.

Default: `modkey = "Mod4"` (windows key)

Example: `modkey = "Mod1"`

### Mousekey

The mousekey is similarly quite important. This value can be used to determine which key, when held, can assist a mouse drag in resizing or moving a floating window or making a window float or tile. For more info please read [this](https://stackoverflow.com/questions/19376338/xcb-keyboard-button-masks-meaning) post on x11 Mod keys.

Default: `mousekey = "Mod4"` (windows key)

Example: `mousekey = "Mod1"`

### Tag Behaviour

Starting with LeftWM 0.2.7, the behaviour of [SwapTags](https://github.com/leftwm/leftwm/wiki/Config#swaptags) was changed such that if you are on a tag, such as tag 1, and then SwapTags to tag 1, LeftWM will go to the previous tag instead. This behaviour can be disabled with `disable_current_tag_swap`:

Default: `disable_current_tag_swap = false`

Example: `disable_current_tag_swap = true` (returns to old behaviour)

### Focus Behaviour

LeftWM now has 3 focusing behaviours (Sloppy, ClickTo, and Driven) and one option (focus_new_windows), which alter the way focus is handled. These encompass 4 different patterns:

1. Sloppy Focus. Focus follows the mouse, hovering over a window brings it to focus.
2. Click-to-Focus. Focus follows the mouse, but only clicks change focus.
3. Driven Focus. Focus disregards the mouse, only keyboard actions drive the focus.
4. Event Focus. Focuses when requested by the window/new windows.

Default:

```
focus_behaviour = "Sloppy" # Can be Sloppy, ClickTo, or Driven
focus_new_windows = true
```

### Layouts

Leftwm supports an ever-growing amount layouts, which define the way that windows are tiled in the workspace.

Default (all layouts, check [this enum](https://github.com/leftwm/leftwm/blob/master/leftwm-core/src/layouts/mod.rs#L21) for the latest list):

```toml
layouts = [
    "MainAndDeck",
    "MainAndVertStack",
    "MainAndHorizontalStack",
    "GridHorizontal",
    "EvenHorizontal",
    "EvenVertical",
    "Fibonacci",
    "CenterMain",
    "CenterMainBalanced",
    "Monocle",
    "RightWiderLeftStack",
    "LeftWiderRightStack",
]
```

Example:

```toml
layouts = [
    "MainAndVertStack",
    "Monocle",
]
```

### Layout Mode

Leftwm now has 2 layout modes, Workspace and Tag. These determine how layouts are remembered. When in Workspace mode, layouts will be remembered per workspace. When in Tag mode, layouts are remembered per tag.

Default:

```toml
layout_mode = "Workspace" # Can be Workspace or Tag
```

### Tags

Tags are the names of the virtual desktops were windows live. In other window managers these are sometimes just called desktops. You can rename them to any unicode string including symbols/icons from popular icon libraries such as font-awesome.

Default: `tags = ["1", "2", "3", "4", "5", "6", "7", "8", "9"]`

Example: `tags = ["Browser ♖", "Term ♗", "Shell ♔", "Code ♕"]`

### Max Window Width

You can configure a `max_window_width` to limit the width of the tiled windows (or rather, the width of columns in a layout). This feature comes in handy when working on ultra-wide monitors where you don't want a single window to take the complete workspace width.

**Demonstration**

Without `max_window_width`

```
+-----------------------------------------------+
|+---------------------------------------------+|
||                                             ||
||                     1                       ||  [49' monitor]
||                                             ||
|+---------------------------------------------+|
+-----------------------------------------------+
+-----------------------------------------------+
|+----------------------+----------------------+|
||                      |                      ||
||          1           |          2           ||  [49' monitor]
||                      |                      ||
|+----------------------+----------------------+|
+-----------------------------------------------+
```

With `max_window_width`

```
+-----------------------------------------------+
|               +---------------+               |
|               |               |               |
|               |       1       |               |  [49' monitor]
|               |               |               |
|               +---------------+               |
+-----------------------------------------------+

                ^^^^^^^^^^^^^^^^^
                MAX_WINDOW_WIDTH
+-----------------------------------------------+
|        +--------------+--------------+        |
|        |              |              |        |
|        |       1      |       2      |        |  [49' monitor]
|        |              |              |        |
|        +--------------+--------------+        |
+-----------------------------------------------+

         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
              MAX_WINDOW_WIDTH * 2
```

This setting can be configured either globally, per workspace, or both. The workspace specific configuration always takes precedence over the global setting.

Values: An `int` value for absolute pixels (`2200` means `2200px`), or a decimal value for fractions (`0.4` means `40%`).
Default: Has no default value. No value means no width limit.

Example:

```toml
# global configuration: 40%
max_window_width = 0.4

[[workspaces]]
y = 0
x = 0
height = 1440
width = 2560
# workspace specific configuration: 1200px
max_window_width = 1200
```



### Workspaces

Workspaces are how you view tags (desktops). A workspace is a area on a screen or most likely the whole screen. in this areas you can view a given tag.

Default: `workspaces = []` (one workspace per screen)

Example (two workspaces on a single ultrawide):

```toml
[[workspaces]]
y = 0
x = 0
height = 1440
width = 1720

[[workspaces]]
y = 0
x = 1720
height = 1440
width = 1720
```

Or with short syntax:

```toml
workspaces = [
    { y = 0, x = 0, height = 1440, width = 1720 },
    { y = 0, x = 1720, height = 1440, width = 1720 },
]
```

### Scratchpads

A scratchpad is a window which you can call to any tag and hide it when not needed. These windows can be any application which can be run from a terminal. To call a scratchpad you will require a keybind for [ToggleScratchPad](https://github.com/leftwm/leftwm/wiki/Config#togglescratchpad).

Example:

```toml
# Create a scratchpad for alacritty
[[scratchpad]]
name = "Alacritty" # This is the name which is referenced when calling (case-sensitive)
value = "alacritty" # The command to load the application if it isn't started
# x, y, width, height are in pixels when an integer is inputted or a percentage when a float is inputted.
# These values are relative to the size of the workspace, and will be restricted depending on the workspace size.
x = 860
y = 390
height = 300
width = 200
```

Or with short syntax:

```toml
scratchpad = [
    { name = "Alacritty", value = "alacritty", x = 860, y = 390, height = 300, width = 200 },
]
```



### Keybind

All other commands are keybindings. you can think of key bindings as a way of telling LeftWM to do something when a key combination is pressed. There are several types of key bindings. In order for the keybind event to fire, the keys listed in the modifier section should be held down, and the key in the key section should then be pressed. [Here is a list of all keys LeftWM can use as a modifier or a key](https://github.com/leftwm/leftwm/blob/master/leftwm-core/src/utils/xkeysym_lookup.rs#L46).

Example:

```toml
[[keybind]]
command = "Execute"
value = "vlc https://www.youtube.com/watch?v=oHg5SJYRHA0"
modifier = []
key = "XF86XK_AudioPlay"
```

You can use the short syntax here as well:

```toml
keybind = [
    { command = "Execute", value = "vlc https://www.youtube.com/watch?v=oHg5SJYRHA0", modifier = [], key = "XF86XK_AudioPlay" },
    { command = "HardReload", modifier = ["modkey", "Shift"], key = "b"},
    { command = "CloseWindow", modifier = ["modkey", "Shift"], key = "q" },
]
```

**Note: even if blank, a modifier must be present! Use `modifier = []` for no modifier**

### Keybind Commands

#### Execute

Execute a shell command when a key combination is pressed.

Partial default:

```
[[keybind]]
command = "Execute"
value = "rofi -show run"
modifier = ["modkey"]
key = "p"
```

**Note: This command requires a value field to be specified**.

#### HardReload

Completely restarts LeftWM.

Example:

```
[[keybind]]
command = "HardReload"
modifier = ["modkey", "Shift"]
key = "b"
```

#### SoftReload

Restarts LeftWM but remembers the state of all windows. This is useful when playing with the config file or themes.

Default:

```
[[keybind]]
command = "SoftReload"
modifier = ["modkey", "Shift"]
key = "r"
```

#### CloseWindow

Closes the window that is currently focused. This is not a forceful quit. It is equivalent to clicking the (x) in the top right of a window normally.

Default:

```
[[keybind]]
command = "CloseWindow"
modifier = ["modkey", "Shift"]
key = "q"
```

#### MoveToLastWorkspace

Takes the window that is currently focused and moves it to the workspace that was active before the current workspace.

Default:

```
[[keybind]]
command = "MoveToLastWorkspace"
modifier = ["modkey", "Shift"]
key = "w"
```

#### MoveWindowToNextWorkspace

Takes the window that is currently focused and moves it to the next workspace.

Example:

```
[[keybind]]
command = "MoveWindowToNextWorkspace"
modifier = ["modkey", "Shift"]
key = "Right"
```

#### MoveWindowToPreviousWorkspace

Takes the window that is currently focused and moves it to the previous workspace.

Example:

```
[[keybind]]
command = "MoveWindowToPreviousWorkspace"
modifier = ["modkey", "Shift"]
key = "Left"
```

#### FloatingToTile

Snaps the focused floating window into the workspace below.

Example:

```
[[keybind]]
command = "FloatingToTile"
modifier = ["modkey", "Shift"]
key = "t"
```

#### TileToFloating

Switch the focused window to floating mode when it is tiled

Example:

```
[[keybind]]
command = "TileToFloating"
modifier = ["modkey", "Shift"]
key = "f"
```

#### ToggleFloating

Switch the focused window between floating and tiled mode.

Example:

```
[[keybind]]
command = "TileToFloating"
modifier = ["modkey", "Ctrl"]
key = "f"
```

#### MoveWindowUp

Re-orders the focused window within the current workspace (moves up in order).

Default:

```
[[keybind]]
command = "MoveWindowUp"
modifier = ["modkey", "Shift"]
key = "Up"
```

#### MoveWindowDown

Re-orders the focused window within the current workspace (moves down in order).

Default:

```
[[keybind]]
command = "MoveWindowDown"
modifier = ["modkey", "Shift"]
key = "Down"
```

#### MoveWindowTop

Re-orders the focused window within the current workspace (moves to the top of the stack).

Default:

```
[[keybind]]
command = "MoveWindowTop"
modifier = ["modkey"]
key = "Return"
```

#### MoveToTag

Moves a window to a given tag.

Partial default:

```
[[keybind]]
command = "MoveToTag"
value = "1"
modifier = ["modkey", "Shift"]
key = "1"
```

**Note: This command requires a value field to be specified**.

#### FocusWindowUp

Focuses the window that is one higher in order on the current workspace.

Default:

```
[[keybind]]
command = "FocusWindowUp"
modifier = ["modkey"]
key = "Up"
```

#### FocusWindowDown

Focuses the window that is one lower in order on the current workspace.

Default:

```
[[keybind]]
command = "FocusWindowDown"
modifier = ["modkey"]
key = "Down"
```

#### NextLayout

Changes the workspace to a new layout.

Default:

```
[[keybind]]
command = "NextLayout"
modifier = ["modkey", "Control"]
key = "Up"
```

#### PreviousLayout

Changes the workspace to the previous layout.

Default:

```
[[keybind]]
command = "PreviousLayout"
modifier = ["modkey", "Control"]
key = "Down"
```

#### SetLayout

Changes the workspace to the specified layout.

Example:

```
[[keybind]]
command = "SetLayout"
value = "Monocle"
modifier = ["modkey"]
key = "m"
```

**Note: This command requires a value field to be specified**.

#### RotateTag

Rotates the tag/layout. If the layout supports it, the tag will flip horizontally, vertically, or both. For example the fibonacci layout rotates in the four different directions.

Example:

```
[[keybind]]
command = "RotateTag"
modifier = ["modkey"]
key = "z"
```

#### FocusWorkspaceNext

Moves the focus from the current workspace to the next workspace (next screen).

Default:

```
[[keybind]]
command = "FocusWorkspaceNext"
modifier = ["modkey"]
key = "Right"
```

#### FocusWorkspacePrevious

Moves the focus from the current workspace to the previous workspace (previous screen).

Default:

```
[[keybind]]
command = "FocusWorkspacePrevious"
modifier = ["modkey"]
key = "Left"
```

#### GotoTag

Changes the tag that is being displayed in a given workspace.

Partial default:

```
[[keybind]]
command = "GotoTag"
value = "9"
modifier = ["modkey"]
key = "9"
```

**Note: This command requires a value field to be specified**.

#### FocusNextTag

Moves the focus from the current tag to the next tag in a given workspace.

Example:

```
[[keybind]]
command = "FocusNextTag"
modifier = ["modkey"]
key = "Right"
```

#### FocusPreviousTag

Moves the focus from the current tag to the previous tag in a given workspace.

Example:

```
[[keybind]]
command = "FocusPreviousTag"
modifier = ["modkey"]
key = "Left"
```

#### SwapTags

Swaps the tags in the current workspace with the tags in the previous workspace.

Default:

```
[[keybind]]
command = "SwapTags"
modifier = ["modkey"]
key = "w"
```

#### IncreaseMainWidth

Increases the width of the currently focused window.

Example:

```
[[keybind]]
command = "IncreaseMainWidth"
value = "5"
modifier = ["modkey"]
key = "a"
```

**Note: This command requires a value field to be specified**. **Note: This command does not apply to all layouts**.

#### DecreaseMainWidth

Decreases the width of the currently focused window.

Example:

```
[[keybind]]
command = "DecreaseMainWidth"
value = "5"
modifier = ["modkey"]
key = "x"
```

**Note: This command requires a value field to be specified**. **Note: This command does not apply to all layouts**.

#### SetMarginMultiplier

Set the multiplier applied to the configured margin value.

Example:

```
[[keybind]]
command = "SetMarginMultiplier"
value = "2.5"
modifier = ["modkey"]
key = "m"
```

**Note: This command requires a value field to be specified**. *Note: The value needs to be a positive float, use "0.0" for no margins at all, use "1.0" to reset.* **Note: This command does not apply to all layouts**.

#### ToggleFullScreen

Toggles the currently focused window between full screen and not full screen.

Example:

```
[[keybind]]
command = "ToggleFullScreen"
modifier = ["modkey"]
key = "f"
```



#### ToggleScratchPad

Toggles the specified scratchpad.

Example:

```
[[keybind]]
command = "ToggleScratchPad"
value = "Alacritty" # Name set for the scratchpad
modifier = ["modkey"]
key = "p"
```

**Note: This command requires a value field to be specified**. 



## Authors

The leftwm development team



## Copyright

2021 - leftwm