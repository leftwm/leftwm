This is a very basic README merely containing a bunch of heads-up notes for using `eww` with `leftwm`

Important:
Copy the `eww-bar` folder to `~/.config/eww/` otherwise every `eww` command needs to pass the path to the folder where the `eww.xml` and `eww.scss` files are located.

Currently `eww` and `leftwm` fail to properly negotiate the `reserved` space for the `bar` windows. To prevent windows from overlapping your bar please use the `gutter` setting in `theme.toml` to force this reserved space.
Here is a snippet of how such gutter configuration might look like, if your bar is 24px in height:
```toml
[[gutter]]
side = "Top"
value = 24
```

Since `eww` is rapidly changing at this times, if stuff breaks please check their [github](https://github.com/elkowar/eww) for documentation on changes and existing issues.
