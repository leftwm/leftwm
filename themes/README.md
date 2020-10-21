# Why have themes

With left there are two types of configs. First there are config settings that are specific to you but don’t really change. These are settings like keybindings. Workspace locations, and names of desktops/tags. These settings can be found in ~/.config/leftwm/config.toml

The appearance of your desktop is different. It’s fun to try new looks and feels. It’s fun to tweak and customize the appearance ( AKA: ricing ). It’s fun to share so others can experience your cool awesome desktop. Leftwm is built around this concept. By pulling all these settings out into themes, you can now easily tweak, switch, and share your experiences. 


# We want your themes

We are looking to expand the list of available themes for an upcoming release. If you enjoy making desktops look good please consider sharing (making a pull request).


# Requirements for a theme

A theme has only two requirements. An “up” and a “down” executable/script. They can be written in whatever makes you happy. The up script you guessed it starts up all the things that make your script unique and awesome. The down script restores the environment to an un-themes state. A theme should be self contained if possible so that it can be shared and doesn’t interfere with other themes. For example when booting an application with a config file, put the config file in the theme folder instead of ~/.config. This way other themes can use the same application 


# Setup / selection of theme

To select a theme all that is required is that it’s located at: ~/.config/leftwm/themes/current
It is strongly recommended that you do this with a symlink. For example up all your themes in the folder ~/.config/leftwm/themes and switch between them using a symlink

A command such as this would set basic_polybar to the current theme.

```
mkdir -p ~/.config/leftwm/themes/
ln -s ~/.config/leftwm/themes/basic_polybar ~/.config/leftwm/themes/current
```


