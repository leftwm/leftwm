# Why have themes

With LeftWM, there are two types of configs:
* **LeftWM Configuration files:** LeftWM configurations are specific to you and don’t change for different themes. These are settings like keybindings, workspace locations, and names of desktops/tags. These settings can be found in `~/.config/leftwm/config.toml`.

* **Theme Configuration files:** The appearance of your desktop is different. It’s fun to try new looks and feels. It’s fun to tweak and customize the appearance (AKA: [ricing](https://www.reddit.com/r/unixporn/comments/3iy3wd/stupid_question_what_is_ricing/)). It’s fun to share so others can experience your awesome desktop! LeftWM is built around this concept. By pulling all these settings out into themes, you can now easily tweak, switch, and share your experiences. This configuration is spread between `theme.toml` and related files contained within a theme's folder.


# We want your themes

We are looking to expand the list of available themes for an upcoming release. If you enjoy making desktops look good please consider sharing by making a pull request on [the community themes repository](https://github.com/leftwm/leftwm-community-themes).


# Requirements for a theme

A theme has only two requirements. An “up” and a “down” executable/script. They can be written in whatever makes you happy. The up script you guessed it starts up all the things that make your script unique and awesome. The down script restores the environment to an un-themes state. A theme should be self contained if possible so that it can be shared and doesn’t interfere with other themes. For example when booting an application with a config file, put the config file in the theme folder instead of ~/.config. This way other themes can use the same application 


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

To select a theme all that is required is that it’s located at: `~/.config/leftwm/themes/current`
It is strongly recommended that you do this with a symlink rather than creating a folder. Using a symlink makes it easy to save all your themes in the folder `~/.config/leftwm/themes` and switch between them using a symlink easily.

The following command would set the included basic_polybar to the current theme:

```bash
ln -s ~/.config/leftwm/themes/basic_polybar ~/.config/leftwm/themes/current
```
