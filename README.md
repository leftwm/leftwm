<div align="center">
  <h1><strong>LeftWM</strong></h1>
  <p>
    <strong>A window manager for adventurers</strong>
  </p>
  <p>
    <a href="https://github.com/leftwm/leftwm/actions?query=workflow%3ACI"><img src="https://github.com/leftwm/leftwm/workflows/CI/badge.svg" alt="build status" /></a>
    <a href="https://github.com/leftwm/leftwm/wiki"><img src="https://img.shields.io/badge/wiki-0.2.6-green.svg" alt="wiki" /></a>
    <a href="https://docs.rs/leftwm/"><img src="https://docs.rs/leftwm/badge.svg" alt="Documentation" /></a>
  </p>
</div>

![Screenshot of LeftWM in action](screenshots/4.jpg)
## Table of contents

- [Why go left](#why-go-left)
- [Config](#config)
  - [Default keys](#default-keys)
  - [Workspaces](#workspaces)
  - [Tags / Desktops](#tags--desktops)
- [Setup (with package manager)](#setup-with-package-manager)
- [Manual Setup (no package manager)](#manual-setup-no-package-manager)
  - [Using a graphical login such as LightDM, GDM, LXDM, and others](#using-a-graphical-login-such-as-lightdm-gdm-lxdm-and-others)
  - [Starting with startx or a login such as slim](#starting-with-startx-or-a-login-such-as-slim)
- [Themes](#themes)
  - [With LeftWM-Theme](#with-leftwm-theme)
  - [Without LeftWM-Theme](#without-leftwm-theme)
- [Dependencies](#dependencies)

## Why go left 

Left is a tiling window manager written in [Rust](https://github.com/rust-lang/rust) that aims to be stable and performant. Left is [designed to do one thing and to do that one thing well](https://en.wikipedia.org/wiki/Unix_philosophy#Do_One_Thing_and_Do_It_Well): _be a window manager_. Left therefore follows the following mantra:

> Left is not a compositor.  
> Left is not a lock screen.  
> Left is not a bar. But, there are lots of good bars out there. With themes, picking one is as simple as setting a symlink.

Because you probably want more than just a black screen, LeftWM is built around the concept of themes. With themes, you can choose between different bars, compositors, backgrounds, colors, docks, and whatever else makes you happy.   

LeftWM was built from the very beginning to support multiple screens and ultrawide monitors. The default keybindings support ultrawide monitors and multiple screens.



## Config
The settings file to change key bindings and the default mod key can be found at
```
~/.config/leftwm/config.toml
```

### Default keys
```
Mod + (1-9) => Switch to a desktop/tag
Mod + Shift + (1-9) => Move the focused window to desktop/tag
Mod + W => Switch the desktops for each screen. Desktops [1][2] changes to [2][1]
Mod + Shift + W => Move window to the other desktop
Mod + (⬆️⬇️) => Focus on the different windows in the current workspace
Mod + Shift + (⬆️⬇️) => Move the different windows in the current workspace
Mod + Enter => Move selected window to the top of the stack in the current workspace
Mod + Ctrl + (⬆️⬇️) => Switch between different layouts
Mod + Shift + (⬅➡) => Switch between different workspaces
Mod + Shift + Enter => Open a terminal
Mod + Ctrl + L => Lock the screen
Mod + Shift + X => Exit LeftWM
Mod + Shift + Q => Close the current window
Mod + Shift + R => Reload LeftWM and its config
Mod + p => Use dmenu to start application
```

### Workspaces
By default, workspaces have a one-to-one relationship with screens, but this is configurable. There are many reasons you might want to change this, but the main reason is for ultrawide monitors. You might want to have two or even three workspaces on a single screen. 

Here is an example config changing the way workspaces are defined (~/.config/leftwm/config.toml)
```
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

### Tags / Desktops
The default tags are 1-9. They can be renamed in the config file by setting the
list of tags.

Here is an example config changing the list of available tags. NOTE: tag navigation (Mod + #) doesn't change based on the name of the tag
```
tags = ["Web", "Code", "Shell", "Music", "Connect"]
```

[More detailed configuration information can be found in the Wiki](https://github.com/leftwm/leftwm/wiki/Config).


### LeftWM is [EWMH](https://en.wikipedia.org/wiki/Extended_Window_Manager_Hints) compliant.




## One of the core concepts/features of LeftWM is theming 

With Left, there are two types of configs. First, there are config settings that are specific to you but don’t change. These are settings like keybindings, workspace locations, and names of desktops/tags. These settings can be found in ~/.config/leftwm/config.toml

The appearance of your desktop is different. It’s fun to try new looks and feels. It’s fun to tweak and customize the appearance (also known as ricing). It’s fun to share so others can experience your cool awesome desktop. LeftWM is built around this concept. By pulling all these settings out into themes, you can now easily tweak, switch, and share your experiences. 

## Setup (with package manager)

LeftWM is available in AUR as well as crates.io. Both are good options for simple installation. Please note, if installing with crates.io you will need to link to the xsession desktop file if you want to be able to login to LeftWM from a graphical login screen. 
```bash
sudo cp PATH_TO_LEFTWM/leftwm.desktop /usr/share/xsessions
```

LeftWM is also available in Fedora [Copr](https://copr.fedorainfracloud.org/coprs/atim/leftwm/): `sudo dnf copr enable atim/leftwm -y && sudo dnf install leftwm`

## Manual Setup (no package manager)

### Using a graphical login such as LightDM, GDM, LXDM, and others

1) Copy leftwm.desktop to /usr/share/xsessions
2) Create a symlink to the build of leftwm so that it is in your path:
```bash
cd /usr/bin
sudo ln -s PATH_TO_LEFTWM/target/debug/leftwm
sudo ln -s PATH_TO_LEFTWM/target/debug/leftwm-worker
sudo ln -s PATH_TO_LEFTWM/target/debug/leftwm-state
sudo ln -s PATH_TO_LEFTWM/target/debug/leftwm-check
```
and
```bash
sudo cp PATH_TO_LEFTWM/leftwm.desktop /usr/share/xsessions
```
You should now see LeftWM in your list of available window managers.

### Starting with startx or a login such as slim
Make sure this is at the end of your .xinitrc file:
```bash .xinitrc
exec dbus-launch leftwm
```

## Themes
If you want to see more than a black screen when you login, select a theme:
### With [LeftWM-Theme](https://github.com/leftwm/leftwm-theme)
```bash
leftwm-theme update
leftwm-theme install NAME_OF_THEME_YOU_LIKE
leftwm-theme apply NAME_OF_THEME_YOU_LIKE
```
### Without [LeftWM-Theme](https://github.com/leftwm/leftwm-theme)
```bash 
mkdir -p ~/.config/leftwm/themes
cd ~/.config/leftwm/themes
ln -s PATH_TO_THE_THEME_YOU_LIKE current
```
LeftWM comes packaged with a couple of default themes. There is also a [community repository for sharing themes](https://github.com/leftwm/leftwm-community-themes)

For more information about themes check out our theme guide [here](https://github.com/leftwm/leftwm/tree/master/themes) or the wiki [here](https://github.com/leftwm/leftwm/wiki/Themes).

## Dependencies 
While LeftWM has very few dependencies, this isn't always the case for themes.
Themes typically require the following to be installed. However, this is up to the
author of the theme, and could be different. 
List of common dependencies for themes: 


| Build Dependency | ubuntu20.4.1              |
| ---------------- | ------------------------- |
| feh              | sudo apt install feh      |
| compton          | sudo apt install compton  |
| picom            | manual                    |
| polybar          | manual                    |
| xmobar           | sudo apt install xmobar   |
| lemonbar         | sudo apt install lemonbar |
| conky            | sudo apt install conky    |
| dmenu            | sudo apt install dmenu    |
