# Leftwm - A window manager for Adventurers


## Why go left 

Left is a tiling window manager written in rust for stability and performance. The core of left is designed to do one thing and one thing well. Be a window manager. Because you probably want more than just a black screen leftwm is built around the concept of theming. With themes you can choose between different bar / compositor / background / colors, whatever makes you happy.   

Leftwm has been built from the very beginning to support multiple screens and has been built around ultrawide monitors. You will see this with the default key bindings

## Left is NOT

Left is not a compositor
Left is not a lock screen. 
Left is not a bar there are lots of good bars out there. With themes, picking one is as simple as setting a symlink.




## Config
The settings file to change key bindings and the default mod key can be found at
```
~/.config/leftwm/config.toml
```

### Default keys
```
Mod + (1-9) => switch to a desktop/tag
Mod + shift + (1-9) => move the focused window to desktop/tag
Mod + w => switch the desktops for each screen. Desktops [1][2] changes to [2][1]
Mod + shift + w => move window to the other desktop
Mod + shift + (⬆️⬇️) => switch between layouts
Mod + shift + enter => open a terminal
Mod + alt + L => lock the screen
Mod + shift + Q => close the current window
Mod + shift + R => reload leftwm and it's config
```

### Workspaces
By default workspaces have a one to one relationship with screens, but this is configurable. There are many reasons you might want to change this, but the main reason is for ultrawide monitors. You might want to have two or even three workspaces on a single screen. 

Here is an example config changing the way workspaces are defined (~/.config/leftwm/config.toml)
```
[[workspace]]
y = 0
x = 0
height = 1440
width = 1720

[[workspace]]
y = 0
x = 1720
height = 1440
width = 1720
```



### Leftwm is EWMH compliant. 




## One of the core concepts/featchers of leftwm is theming 

With left there are two types of configs. First there are config settings that are specific to you but don’t really change. These are settings like keybindings. Workspace locations, and names of desktops/tags. These settings can be found in ~/.config/leftwm/config.toml

The appearance of your desktop is different. It’s fun to try new looks and feels. It’s fun to tweak and customize the appearance ( AKA: ricing ). It’s fun to share so others can experience your cool awesome desktop. Leftwm is built around this concept. By pulling all these settings out into themes, you can now easily tweak, switch, and share your experiences. 


## Manual Setup (no-package manager)

### Using a graphical login such as LightDM, GDM, LXDM, and others

1) copy leftwm.desktop to /usr/share/xsessions
2) create a symlink to the build of leftwm so that it is in your path
```bash
cd /usr/bin
sudo ln -s PATH_TO_LEFTWM/target/debug/leftwm
```
You should now see leftwm in your list of available window managers.

### Starting with startx or a login such as slim
make sure this is at the end of your .xinitrc file
```bash .xinitrc
exec dbus-launch /path_to_leftwm/leftwm
```

