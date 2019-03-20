# leftwm
A window manager for Adventurers

## Setup 

### using a graphical login such as LightDM, GDM, LXDM, and others

1) copy leftwm.desktop to /usr/share/xsessions
2) create a symlink to the build of leftwm so that it is in your path
```bash
cd /usr/bin
sudo ln -s PATH_TO_LEFTWM/target/debug/leftwm
```
You should now see leftwm in your list of available window managers.

### starting with startx or a login such as slim
make sure this is at the end of your .xinitrc file
```bash .xinitrc
exec dbus-launch /path_to_leftwm/leftwm
```



