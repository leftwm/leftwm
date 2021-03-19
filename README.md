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

# Table of contents

- [Why go left](#why-go-left)
- [Dependencies](#dependencies)
- [Installation (with package manager)](#installation-with-package-manager)
- [Manual Installation (no package manager)](#manual-installation-no-package-manager)
  - [Using a graphical login such as LightDM, GDM, LXDM, and others](#using-a-graphical-login-such-as-lightdm-gdm-lxdm-and-others)
  - [Starting with startx or a login such as slim](#starting-with-startx-or-a-login-such-as-slim)
- [Theming](#theming)
  - [With LeftWM-Theme](#with-leftwm-theme)
  - [Without LeftWM-Theme](#without-leftwm-theme)
- [Configuring](#configuring)
  - [Default keys](#default-keys)
  - [Workspaces](#workspaces)
  - [Tags / Desktops](#tags--desktops)
  - [Layouts](#layouts)
- [Troubleshooting](#troubleshooting)

# Why go left

Left is a tiling window manager written in [Rust](https://github.com/rust-lang/rust) that aims to be stable and performant. Left is [designed to do one thing and to do that one thing well](https://en.wikipedia.org/wiki/Unix_philosophy#Do_One_Thing_and_Do_It_Well): _be a window manager_. Left therefore follows the following mantra:

> Left is not a compositor.  
> Left is not a lock screen.  
> Left is not a bar. But, there are lots of good bars out there. With themes, picking one is as simple as setting a symlink.

Because you probably want more than just a black screen, LeftWM is built around the concept of themes. With themes, you can choose between different bars, compositors, backgrounds, colors, docks, and whatever else makes you happy.

LeftWM was built from the very beginning to support multiple screens and ultrawide monitors. The default keybindings support ultrawide monitors and multiple screens.

## One of the core concepts/features of LeftWM is theming

With LeftWM, there are two types of configuration files:

- **LeftWM Configuration files:** LeftWM configurations are specific to you and don’t change for different themes. These are settings like keybindings, workspace locations, and names of desktops/tags. These settings can be found in `~/.config/leftwm/config.toml`.

- **Theme Configuration files:** The appearance of your desktop is different. It’s fun to try new looks and feels. It’s fun to tweak and customize the appearance (AKA: [ricing](https://www.reddit.com/r/unixporn/comments/3iy3wd/stupid_question_what_is_ricing/)). It’s fun to share so others can experience your awesome desktop! LeftWM is built around this concept. By pulling all these settings out into themes, you can now easily tweak, switch, and share your experiences. This configuration is spread between `theme.toml` and related files contained within a theme's folder.

# Dependencies

While LeftWM has very few dependencies, this isn't always the case for themes.
Themes typically require the following to be installed. However, this is up to the
author of the theme, and could be different.

List of LeftWM dependencies:  

- xorg (libxinerama, xrandr, xorg-server)  
- bash
- rust  

List of common dependencies for themes:

| Dependency<br>(git)| Ubuntu 20.4.1<br> _sudo apt install {}_  | Arch<br> _sudo pacman -S {}_  | Fedora 33<br> _sudo dnf install {}_  | PKGS  |
|- |- |- |- |- |
| [feh](https://github.com/derf/feh)  | feh  | feh  | feh  | [feh](https://pkgs.org/search/?q=feh&on=provides)  |
| [compton](https://github.com/chjj/compton)  | compton  | yay -S picom*  | compton  | [compton](https://pkgs.org/download/compton)  |
| [picom](https://github.com/yshui/picom)  | manual **  | picom  | picom  | [picom](https://pkgs.org/download/picom)  |
| [polybar](https://github.com/polybar/polybar)  | manual **  | yay -S polybar*  | polybar  | [polybar](https://pkgs.org/download/polybar)  |
| [xmobar](https://github.com/jaor/xmobar)  | xmobar  | xmobar  | xmobar  | [xmobar](https://pkgs.org/download/xmobar)  |
| [lemonbar](https://github.com/LemonBoy/bar)  | lemonbar  | yay -S lemonbar*  | manual **  | [lemonbar](https://pkgs.org/download/lemonbar)  |
| [conky](https://github.com/brndnmtthws/conky)  | conky  | conky  | conky  | [conky](https://pkgs.org/download/conky)  |
| [dmenu](https://git.suckless.org/dmenu)  | dmenu  | dmenu  | dmenu  | [dmenu](https://pkgs.org/download/dmenu)  |

> \* You can use whichever AUR wrapper you like  
> \*\* See the git page (link in first column) for how to install these manually

# Installation (with package manager)

LeftWM is available in the AUR as well as crates.io. Both are good options for simple installation. If you install LeftWM with crates.io, you will need to link to the xsession desktop file if you want to be able to login to LeftWM from a graphical login screen:

```bash
sudo cp PATH_TO_LEFTWM/leftwm.desktop /usr/share/xsessions
```

LeftWM is also available in Fedora [Copr](https://copr.fedorainfracloud.org/coprs/atim/leftwm/):

```bash
sudo dnf copr enable atim/leftwm -y && sudo dnf install leftwm
```

# Manual Installation (no package manager)

## Using a graphical login such as LightDM, GDM, LXDM, and others

1. Dependencies: Rust, Cargo
2. Clone the repository and cd into the directory

```bash
git clone https://github.com/leftwm/leftwm.git
cd leftwm
```

3. Build leftwm

```bash
cargo build --release
```

4. Copy leftwm executables to the /usr/bin folder

```bash
sudo cp ./target/release/leftwm /usr/bin/leftwm
sudo cp ./target/release/leftwm-worker /usr/bin/leftwm-worker
sudo cp ./target/release/leftwm-state /usr/bin/leftwm-state
sudo cp ./target/release/leftwm-check /usr/bin/leftwm-check
```

5. Copy leftwm.desktop to xsessions folder

```bash
sudo cp leftwm.desktop /usr/share/xsessions/
```

You should now see LeftWM in your list of available window managers.  At this point, expect only a simple black screen on login.  For a more customized look, install a theme.

## Optional Development Installation

If your goal is to continously build leftwm and keep up to date with the latest releases, you may prefer to symlink the leftwm executables instead of copying them.  If you choose to install this way, make sure you do not move the build directory as it will break your installation.  Normal installation and development installation only differ on step 4.

1. Dependencies: Rust, Cargo
2. Clone the repository and cd into the directory

```bash
git clone https://github.com/leftwm/leftwm.git
cd leftwm
```

3. Build leftwm

```bash
cargo build --release
```

4. Create the symlinks

```bash
sudo ln -s "$(pwd)"/target/release/leftwm /usr/bin/leftwm
sudo ln -s "$(pwd)"/target/release/leftwm-worker /usr/bin/leftwm-worker
sudo ln -s "$(pwd)"/target/release/leftwm-state /usr/bin/leftwm-state
sudo ln -s "$(pwd)"/target/release/leftwm-check /usr/bin/leftwm-check
```

5. Copy leftwm.desktop to xsessions folder

```bash
sudo cp leftwm.desktop /usr/share/xsessions/
```

You should now see LeftWM in your list of available window managers.  At this point, expect only a simple black screen on login.  For a more customized look, install a theme.

### Rebuilding the development installation

1.  Now if you want to get the newest version of leftwm run this command from your build directory:  

```bash
git pull origin master
```

2. Build leftwm

```bash
cargo build --release
```
 
3. Sign out/in to use the new leftwm executables.  


## Starting with startx or a login such as slim

Make sure this is at the end of your .xinitrc file:

```bash .xinitrc
exec dbus-launch leftwm
```

# Theming

If you want to see more than a black screen when you login, select a theme:

## With [LeftWM-Theme](https://github.com/leftwm/leftwm-theme)

```bash
leftwm-theme update
leftwm-theme install NAME_OF_THEME_YOU_LIKE
leftwm-theme apply NAME_OF_THEME_YOU_LIKE
```

## Without [LeftWM-Theme](https://github.com/leftwm/leftwm-theme)

```bash
mkdir -p ~/.config/leftwm/themes
cd ~/.config/leftwm/themes
ln -s PATH_TO_THE_THEME_YOU_LIKE current
```

LeftWM comes packaged with a couple of default themes. There is also a [community repository for sharing themes](https://github.com/leftwm/leftwm-community-themes)

For more information about themes check out our theme guide [here](https://github.com/leftwm/leftwm/tree/master/themes) or the wiki [here](https://github.com/leftwm/leftwm/wiki/Themes).

# Configuring

The settings file to change key bindings and the default mod key can be found at

```bash
~/.config/leftwm/config.toml
```

## Default keys

| Keybinding          | Description                                                            |
|---------------------|------------------------------------------------------------------------|
| Mod + (1-9)         | Switch to a desktop/tag                                                |
| Mod + Shift + (1-9) | Move the focused window to desktop/tag                                 |
| Mod + W             | Switch the desktops for each screen. Desktops [1][2] changes to [2][1] |
| Mod + Shift + W     | Move window to the other desktop                                       |
| Mod + (⬆️⬇️)          | Focus on the different windows in the current workspace                |
| Mod + Shift + (⬆️⬇️)  | Move the different windows in the current workspace                    |
| Mod + Enter         | Move selected window to the top of the stack in the current workspace  |
| Mod + Ctrl + (⬆️⬇️)   | Switch between different layouts                                       |
| Mod + Shift + (⬅➡)  | Switch between different workspaces                                    |
| Mod + Shift + Enter | Open a terminal                                                        |
| Mod + Ctrl + L      | Lock the screen                                                        |
| Mod + Shift + X     | Exit LeftWM                                                            |
| Mod + Shift + Q     | Close the current window                                               |
| Mod + Shift + R     | Reload LeftWM and its config                                           |
| Mod + p             | Use dmenu to start application                                         |

## Floating Windows

You can optionally switch between tiling or floating mode for any window.

| Keybinding              | Description                             |
|-------------------------|-----------------------------------------|
| Mod + MouseDrag         | Switch a tiled window to floating mode  |
| Mod + RightMouseDrag    | Resize a window                         |
| Drag window onto a tile | Switch a floating window to tiling mode |
| Mod + Shift + (1-9)     | Switch a floating window to tiling mode |

## Workspaces

By default, workspaces have a one-to-one relationship with screens, but this is configurable. There are many reasons you might want to change this, but the main reason is for ultrawide monitors. You might want to have two or even three workspaces on a single screen.

Here is an example config changing the way workspaces are defined (~/.config/leftwm/config.toml)

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

## Tags / Desktops

The default tags are 1-9. They can be renamed in the config file by setting the
list of tags.

Here is an example config changing the list of available tags. NOTE: tag navigation (Mod + #) doesn't change based on the name of the tag

```toml
tags = ["Web", "Code", "Shell", "Music", "Connect"]
```

## Layouts

By default, all layouts are enabled. There are a lot of layouts so you might want to consider only enabling the ones you use. To do this add a layout section to your config.toml file. This enables only the layouts you specify 

Example:
```toml
layouts = ["MainAndHorizontalStack", "GridHorizontal", "Fibonacci", "EvenVertical", "EvenHorizontal", "CenterMain"]
```

[More detailed configuration information can be found in the Wiki](https://github.com/leftwm/leftwm/wiki/Config).

## LeftWM is [EWMH](https://en.wikipedia.org/wiki/Extended_Window_Manager_Hints) compliant

The default layouts are [all of the kinds](src/layouts/mod.rs#L16) described by the Layout enum.

## Troubleshooting

| Issue | Description | Solution |
|-|-|:-:|
| LeftWM not listed by login manager | It's likely you need to add the xsessions file to the right folder. | See [installation](#installation-with-package-manager) |
| No config.toml file exists | LeftWM does not always ship with a `config.toml`. You will need to execute LeftWM at least once for one to be generated. | Try the following: ``` leftwm-worker ``` |
| Config.toml is not being parsed | LeftWM ships with a binary called leftwm-check. It might not be installed by the AUR. | Try the following: ``` leftwm-check ``` |
| Keybinding doesn't work | It's likely you need to specify a value or have a typo. | See Wiki |
