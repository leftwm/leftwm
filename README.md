<div align="center">
  <h1><strong>LeftWM</strong></h1>
  <p>
    <strong>A window manager for adventurers</strong>
  </p>
  <p>
    <a href="https://github.com/leftwm/leftwm/actions?query=workflow%3ACI">
        <img src="https://github.com/leftwm/leftwm/workflows/CI/badge.svg" alt="build status" />
    </a>
    <a href="https://github.com/leftwm/leftwm/wiki">
        <img src="https://img.shields.io/badge/wiki-0.2.8-green.svg" alt="wiki" />
    </a>
    <a href="https://discord.gg/c9rB3wDnVs">
        <img src="https://img.shields.io/discord/889371782388256818?color=%235865F2&label=Discord" alt="discord" />
    </a>
    <a href="https://docs.rs/leftwm/">
        <img src="https://docs.rs/leftwm/badge.svg" alt="Documentation" />
    </a>
  </p>
</div>

![Screenshot of LeftWM in action](screenshots/4.jpg)

# Table of contents

- [Why go left](#why-go-left)
- [Dependencies](#dependencies)
- [Installation (with package manager)](#installation-with-package-manager)
- [Manual Installation (no package manager)](#manual-installation-no-package-manager)
  - [Using a graphical login such as LightDM, GDM, LXDM, and others](#using-a-graphical-login-such-as-lightdm-gdm-lxdm-and-others)
  - [Optional Development Installation](#optional-development-installation)
  - [Using the Makefile](#using-the-makefile)
  - [Starting with startx or a login such as slim](#starting-with-startx-or-a-login-such-as-slim)
- [Theming](#theming)
  - [With LeftWM-Theme](#with-leftwm-theme)
  - [Without LeftWM-Theme](#without-leftwm-theme)
- [Configuring](#configuring)
  - [Default keys](#default-keys)
  - [Floating Windows](#floating-windows)
  - [Workspaces](#workspaces)
  - [Tags / Desktops](#tags--desktops)
  - [Layouts](#layouts)
- [Troubleshooting](#troubleshooting)
- [Support](#support)

# Why go left

LeftWM is a tiling window manager written in [Rust] that aims to be stable and performant. LeftWM is
[designed to do one thing and to do that one thing well][unix-philosophy]: _be a window manager_.
LeftWM follows the following mantra:

> LeftWM is not a compositor.  
> LeftWM is not a lock screen.  
> LeftWM is not a bar. But, there are lots of good bars out there. With themes, picking one is as
  simple as setting a symlink.

Because you probably want more than just a black screen, LeftWM is built around the concept of
themes. With themes, you can choose between different bars, compositors, backgrounds, colors, docks,
and whatever else makes you happy.

LeftWM was built from the very beginning to support multiple screens and ultrawide monitors. The
default keybindings support ultrawide monitors and multiple screens.

[Rust]: https://github.com/rust-lang/rust
[unix-philosophy]: https://en.wikipedia.org/wiki/Unix_philosophy#Do_One_Thing_and_Do_It_Well

## One of the core concepts/features of LeftWM is theming

With LeftWM, there are two types of configuration files:

- **LeftWM Configuration files:** LeftWM configurations are specific to you and don’t change for
  different themes. These are settings like keybindings, workspace locations, and names of
  desktops/tags. These settings can be found in `~/.config/leftwm/config.toml`.

- **Theme Configuration files:** The appearance of your desktop is different. It’s fun to try new
  looks and feels. It’s fun to tweak and customize the appearance (AKA: [ricing]). It’s fun to share
  so others can experience your awesome desktop! LeftWM is built around this concept. By pulling all
  these settings out into themes, you can now easily tweak, switch, and share your experiences. This
  configuration is spread between `theme.toml` and related files contained within a theme's folder.

[ricing]: https://www.reddit.com/r/unixporn/comments/3iy3wd/stupid_question_what_is_ricing/

# Dependencies

While LeftWM has very few dependencies, this isn't always the case for themes.
Themes typically require the following to be installed. However, this is up to the
author of the theme and could be different.

List of LeftWM dependencies:  

- xorg (runtime, build): specifically libx11, xrandr, xorg-server, libxinerama  
- sh (runtime): any posix-compliant shell for starting up and down files
- rust (build): >= 1.56.0
- bash (optional): Most of the themes available use bash, though the scripts maybe converted to any posix-compliant shell

List of common dependencies for themes:

| Dependency<br>(git) | Ubuntu 20.4.1<br> _sudo apt install {}_ | Arch<br> _sudo pacman -S {}_ | Fedora 33<br> _sudo dnf install {}_ | PKGS |
|--------------------------|-----------|-------------------|-----------|--------------------------|
| [feh][feh-git]           | feh       | feh               | feh       | [feh][feh-pkg]           |
| [compton][compton-git]   | compton   | picom             | compton   | [compton][compton-pkg]   |
| [picom][picom-git]       | manual ** | picom             | picom     | [picom][picom-pkg]       |
| [polybar][polybar-git]   | manual ** | polybar           | polybar   | [polybar][polybar-pkg]   |
| [xmobar][xmobar-git]     | xmobar    | xmobar            | xmobar    | [xmobar][xmobar-pkg]     |
| [lemonbar][lemonbar-git] | lemonbar  | paru -S lemonbar* | manual ** | [lemonbar][lemonbar-pkg] |
| [conky][conky-git]       | conky     | conky             | conky     | [conky][conky-pkg]       |
| [dmenu][dmenu-git]       | dmenu     | dmenu             | dmenu     | [dmenu][dmenu-pkg]       |

[feh-git]: https://github.com/derf/feh
[feh-pkg]: https://pkgs.org/search/?q=feh&on=provides
[compton-git]: https://github.com/chjj/compton
[compton-pkg]: https://pkgs.org/download/compton
[picom-git]: https://github.com/yshui/picom
[picom-pkg]: https://pkgs.org/download/picom
[polybar-git]: https://github.com/polybar/polybar
[polybar-pkg]: https://pkgs.org/download/polybar
[xmobar-git]: https://github.com/jaor/xmobar
[xmobar-pkg]: https://pkgs.org/download/xmobar
[lemonbar-git]: https://github.com/LemonBoy/bar
[lemonbar-pkg]: https://pkgs.org/download/lemonbar
[conky-git]: https://github.com/brndnmtthws/conky
[conky-pkg]: https://pkgs.org/download/conky
[dmenu-git]: https://git.suckless.org/dmenu
[dmenu-pkg]: https://pkgs.org/download/dmenu

> \* You can use whichever AUR wrapper you like. See [paru] and [yay].  
> \*\* See the git page (link in first column) for how to install these manually

# Installation (with package manager)

## Gentoo ([GURU])

```sh
sudo layman -a guru && sudo emerge --sync 
sudo emerge --ask --verbose x11-wm/leftwm
```

## Archlinux ([AUR])

```sh
paru -S leftwm
```

[paru] is an AUR helper like [yay], but written in [Rust].

## Fedora ([copr])

```sh
sudo dnf copr enable atim/leftwm -y && sudo dnf install leftwm
```

## NetBSD ([Official repositories])

```sh
pkgin install leftwm
```

or, if you prefer to build it from source

```sh
cd /usr/pkgsrc/wm/leftwm
make install
```

## Void ([XBPS])

```sh
sudo xbps-install -S leftwm
```

## Cargo ([crates.io])

```sh
cargo install leftwm
```

If you install LeftWM with crates.io, you will need to link to the xsession desktop file if you want
to be able to login to LeftWM from a display manager (GDM, SSDM, LightDM, etc.):

```sh
sudo cp PATH_TO_LEFTWM/leftwm.desktop /usr/share/xsessions
```

[AUR]: https://aur.archlinux.org/packages/leftwm
[GURU]: https://gitweb.gentoo.org/repo/proj/guru.git/tree/x11-wm/leftwm
[copr]: https://copr.fedorainfracloud.org/coprs/atim/leftwm/
[Official repositories]: https://pkgsrc.se/wm/leftwm/
[XBPS]: https://voidlinux.org/packages/?arch=x86_64&q=leftwm
[crates.io]: https://crates.io/crates/leftwm
[paru]: https://github.com/Morganamilo/paru
[yay]: https://github.com/Jguer/yay

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
   sudo install -s -Dm755 ./target/release/leftwm ./target/release/leftwm-worker ./target/release/leftwm-state ./target/release/leftwm-check ./target/release/leftwm-command -t /usr/bin
   ```

5. Copy leftwm.desktop to xsessions folder

   ```bash
   sudo cp leftwm.desktop /usr/share/xsessions/
   ```

You should now see LeftWM in your list of available window managers.  At this point, expect only a
simple black screen on login.  For a more customized look, install a theme.

## Optional Development Installation

If your goal is to continuously build leftwm and keep up to date with the latest releases, you may
prefer to symlink the leftwm executables instead of copying them.  If you choose to install this
way, make sure you do not move the build directory as it will break your installation.

1. Dependencies: Rust, Cargo
2. Clone the repository and cd into the directory

   ```bash
   git clone https://github.com/leftwm/leftwm.git
   cd leftwm
   ```

3. Build leftwm

   ```bash
   # Without systemd logging
   cargo build --release
 
   # OR with systemd logging (view with 'journalctl -f -t leftwm-worker')
   cargo build --release --features=journald
   ```

4. Create the symlinks

   ```bash
   sudo ln -s "$(pwd)"/target/release/leftwm /usr/bin/leftwm
   sudo ln -s "$(pwd)"/target/release/leftwm-worker /usr/bin/leftwm-worker
   sudo ln -s "$(pwd)"/target/release/leftwm-state /usr/bin/leftwm-state
   sudo ln -s "$(pwd)"/target/release/leftwm-check /usr/bin/leftwm-check
   sudo ln -s "$(pwd)"/target/release/leftwm-command /usr/bin/leftwm-command
   ```

5. Copy leftwm.desktop to xsessions folder

   ```bash
   sudo cp leftwm.desktop /usr/share/xsessions/
   ```

You should now see LeftWM in your list of available window managers.  At this point, expect only a
simple black screen on login.  For a more customized look, install a theme.

### Rebuilding the development installation

1. Now if you want to get the newest version of leftwm run this command from your build directory:  

   ```bash
   git pull origin main
   ```

2. Build leftwm

   ```bash
   # Without systemd logging
   cargo build --release
 
   # OR with systemd logging (view with 'journalctl -f -t leftwm-worker')
   cargo build --release --features=journald
   ```

3. And press the following keybind to reload leftwm

   ```bash
   Mod + Shift + R
   ```

## Using the Makefile

For conveniece we also have a Makefile with the following rules:

|make ... | info |
| - | - |
|all | implies `build` and `test` |
|test | runs same tests as CI on github |
| build | builds with cargo flag `--release` |
| clean | clean all buildfiles |
| install | install by copying binaries to `/usr/bin`, also places `leftwm.desktop` file to `/usr/shar/xsession` and cleans build files |
| install-dev | installs by symlinking, copies `leftwm.desktop`, no clean |
| uninstall | removes `leftwm-*` files from `/usr/bin` and `leftwm.desktop` file |

## Starting with startx or a login such as slim

Make sure this is at the end of your .xinitrc file:

```bash
# .xinitrc
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

LeftWM comes packaged with a couple of default themes. There is also a
[community repository][community-repo] for sharing themes.

For more information about themes check out our [theme guide][theme-guide] or the [wiki].

[community-repo]: https://github.com/leftwm/leftwm-community-themes
[theme-guide]: https://github.com/leftwm/leftwm/tree/main/themes
[wiki]: https://github.com/leftwm/leftwm/wiki/Themes

# Configuring

The settings file to change key bindings and the default mod key can be found at

```bash
~/.config/leftwm/config.toml
```

the file is automatically generated when leftwm or leftwm-check is run for the first time.

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

**Note:** Although we encourage you to use [Alacritty](https://github.com/alacritty/alacritty),
LeftWM will set your default terminal to the first terminal it finds in this list (in the order
presented):

1. Alacritty
2. Termite
3. Kitty
4. URXVT
5. RXVT
6. ST
7. ROXTerm
8. Eterm
9. XTerm
10. Terminator
11. Terminology
12. Gnome Terminal
13. XFCE4 Terminal
14. Konsole
15. UXTerm
16. Guake

## Floating Windows

You can optionally switch between tiling or floating mode for any window.

| Keybinding              | Description                             |
|-------------------------|-----------------------------------------|
| Mod + MouseDrag         | Switch a tiled window to floating mode  |
| Mod + RightMouseDrag    | Resize a window                         |
| Drag window onto a tile | Switch a floating window to tiling mode |
| Mod + Shift + (1-9)     | Switch a floating window to tiling mode |

## Workspaces

By default, workspaces have a one-to-one relationship with screens, but this is configurable. There
are many reasons you might want to change this, but the main reason is for ultrawide monitors. You
might want to have two or even three workspaces on a single screen.

Here is an example config changing the way workspaces are defined (~/.config/leftwm/config.toml)

---
**NOTE**
The line `workspaces = []` needs to be removed or commented out if a configuration like the
following example is used.

---

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

You may optionally specify an ID for your defined workspaces. This is helpful if you want to assign different gutter settings to each workspace in your theme.

```toml
[[workspaces]]
y = 0
x = 1720
height = 1440
width = 1720
id = 1

[[workspaces]]
y = 0
x = 0
height = 1440
width = 1720
id = 0
```

---
**NOTE**
You do not have to define an ID for each workspace, but if you assign an ID to one workspace all subsequently defined workspaces without an ID will be assigned an ID incrementing from the largest ID currently assigned to any workspace. In the above example if the second workspace was not defined with ID = 0 it would be assigned ID = 2. Keep this in mind when creating or customizing themes.

---

## Tags / Desktops

The default tags are 1-9. They can be renamed in the config file by setting the
list of tags.

Here is an example config changing the list of available tags. NOTE: tag navigation (Mod + #)
doesn't change based on the name of the tag.

```toml
tags = ["Web", "Code", "Shell", "Music", "Connect"]
```

## Layouts

By default, all layouts are enabled. There are a lot of layouts so you might want to consider only
enabling the ones you use. To do this add a layout section to your config.toml file. This enables
only the layouts you specify.

Example:

```toml
layouts = ["MainAndHorizontalStack", "GridHorizontal", "Fibonacci", "EvenVertical", "EvenHorizontal", "CenterMain", "CenterMainFluid"]
```

Layouts may also be specified on individual workspaces, this is useful if you have monitors with different aspect ratios or orientation.

Example:

```toml
[[workspaces]]
id = 0
x = 0
y = 480
width = 3840
height = 1600
layouts = ["CenterMain", "CenterMainBalanced", "EvenHorizontal"]

[[workspaces]]
id = 1
x = 3840
y = 0
width = 1440
height = 2560
layouts = ["MainAndHorizontalStack", "EvenVertical"]
```

**NOTE**
When defining layouts per workspace, you will need to define workspace IDs explicitely.

[More detailed configuration information can be found in the Wiki.][config-wiki]

[config-wiki]: https://github.com/leftwm/leftwm/wiki/Config

## LeftWM is [EWMH](https://en.wikipedia.org/wiki/Extended_Window_Manager_Hints) compliant

The default layouts are [all of the kinds](leftwm-core/src/layouts/mod.rs#L21) described by the Layout enum.

## Troubleshooting

| Issue | Description | Solution |
|-|-|:-:|
| LeftWM not listed by login manager | You likely need to add the xsessions file to the right folder. | See [installation](#installation-with-package-manager) |
| No config.toml file exists | LeftWM does not always ship with a `config.toml`. You will need to execute LeftWM at least once for one to be generated. | Try the following: ``` leftwm-worker ``` |
| Config.toml is not being parsed | LeftWM ships with a binary called leftwm-check. It might not be installed by the AUR. | Try the following: ``` leftwm-check ``` |
| Keybinding doesn't work | It's likely you need to specify a value or have a typo. | See Wiki |

## Support

LeftWM now has a [Discord](https://discord.gg/c9rB3wDnVs) channel for faster help.
