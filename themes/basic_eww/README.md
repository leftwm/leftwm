This is a very basic README, merely containing a bunch of heads-up notes for using `eww` with `leftwm`

Important:
Copy the `eww-bar` folder to `~/.config/eww/` otherwise every `eww` command needs to pass the path to the folder where the `eww.yuck` and `eww.scss` files are located.
It is also possible to symlink instead of copy, though `eww` isn't to happy about this and will log some errors, even though working just fine.
The previously used xml config is still included in this example in the `legacy_eww_xml_config` folder for reference.

Previously (legacy xml configured) `eww` and `leftwm` failed to properly negotiate the `reserved` space for the `bar` windows, this fixed now though. To prevent windows from overlapping your bar please use the `gutter` setting in `theme.toml` to force this reserved space.
Here is a snippet of how such gutter configuration might look like, if your bar is 24px in height:
```toml
[[gutter]]
side = "Top"
value = 24
```

Since `eww` is still rapidly changing, if stuff breaks please check their [github](https://github.com/elkowar/eww) for documentation on changes and existing issues.
