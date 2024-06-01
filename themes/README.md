# Why have themes

With LeftWM, there are two types of configs:
* **LeftWM Configuration files:** LeftWM configurations are specific to you and don’t change for different themes. These are settings like keybindings, workspace locations, and names of desktops/tags. These settings are loaded from `$XDG_CONFIG_HOME/leftwm/config.ron`.

* **Theme Configuration files:** The appearance of your desktop is different. It’s fun to try new looks and feels. It’s fun to tweak and customize the appearance (AKA: [ricing](https://www.reddit.com/r/unixporn/comments/3iy3wd/stupid_question_what_is_ricing/)). It’s fun to share so others can experience your awesome desktop! LeftWM is built around this concept. By pulling all these settings out into themes, you can now easily tweak, switch, and share your experiences. This configuration is spread between `theme.ron` and related files contained within a theme's folder.


# We want your themes

We are looking to expand the list of available themes for an upcoming release. If you enjoy making desktops look good please consider sharing by making a pull request on [the community themes repository](https://github.com/leftwm/leftwm-community-themes).

# Requirements for a theme

A theme has only two requirements. An `up` and a `down` executable/script. They can be written in whatever makes you happy. The `up` script starts up all the things that make your script unique and awesome. The `down` script restores the environment to an un-themed state.

The `up` script is contained in the theme directory, and it should copy the `down` script to the expected location `/tmp/leftwm-theme-down`. See the example below for a template.

Note: if you are writing your own theme, please make sure the `up` and `down` scripts are executable!

```sh
chmod a+x up down
```

A theme should be self contained if possible so that it can be shared and doesn’t interfere with other themes. For example when booting an application with a config file, put the config file in the theme folder instead of ~/.config. This way other themes can use the same application.

Common applications booted with themes are compositors and bars (e.g., `picom` and `polybar`).

## Examples of files in a theme directory

Here is a prototypical example of an `up` file.
```bash
#!/usr/bin/env bash
export SCRIPTPATH="$( cd "$(dirname "$0")" ; pwd -P )"

# unload the old theme
if [ -f "/tmp/leftwm-theme-down" ]; then
	/tmp/leftwm-theme-down
	rm /tmp/leftwm-theme-down
fi

# load the down script to the expected location
ln -s $SCRIPTPATH/down /tmp/leftwm-theme-down

# set the theme.ron config
leftwm-command "LoadTheme $SCRIPTPATH/theme.ron"

# set background using .fehbg file if it exists
if [ -x "$(command -v feh)" ] && [ -f "$HOME/.fehbg" ]; then
  $HOME/.fehbg
fi

# load polybar
polybar -c $SCRIPTPATH/polybar.ini &> /dev/null &

# load picom (note: cannot set daemon command from config file)
picom --daemon --config $SCRIPTPATH/picom.conf
```

Here is an example `down` script compatible with the above `up` script.
```bash
#!/usr/bin/env bash

leftwm-command "UnloadTheme"

pkill polybar
pkill picom
```

Here is an example `theme.ron` file:
```rust
(
	border_width: 2,
	margin: (18, 18, 18, 18),
	focused_border_color: "#FF0000",
	default_border_color: "#00FF00",
	floating_border_color: "#0000FF",
)
```

For information on `picom.conf` or `polybar.ini`, the user is referred to the manual pages for `picom` and `polybar`.

# Setup / selection of theme

There are two ways to setup a theme: you can use [leftwm-theme](https://github.com/leftwm/leftwm-theme/) or you can set a symlink yourself.

## Using LeftWM-theme
Install LeftWM-theme, as per the directions on [its Github](https://github.com/leftwm/leftwm-theme).

Update your list of themes:
```bash
leftwm-theme update
```
Install the theme you like:
```bash
leftwm-theme install "THEME NAME GOES HERE"
```
Apply the theme you like as the current theme:
```bash
leftwm-theme apply "THEME NAME GOES HERE"
```

## Using symlinks

To select a theme all that is required is that it’s located at: `$XDG_CONFIG_HOME/leftwm/themes/current`
It is strongly recommended that you do this with a symlink rather than creating a folder. Using a symlink makes it easy to save all your themes in the folder `$XDG_CONFIG_HOME/leftwm/themes` and switch between them using a symlink easily.

The following command would set the included basic_polybar to the current theme:

```bash
ln -s $XDG_CONFIG_HOME/leftwm/themes/basic_polybar $XDG_CONFIG_HOME/leftwm/themes/current
```
