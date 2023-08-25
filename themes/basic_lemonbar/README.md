# On using lemonbar

This theme is an example of running leftwm with lemonbar as the status bar. Lemonbar is the most lightweight and simple statusbar for X. Out of the box it's just a blank bar. It doesn't provide any status text on its own. That's up to the user to provide it with.

Text is given to lemonbar via piping. The simplest example would be: `echo 'some text' | lemonbar -p` (the `-p` flag prevents it from immediately exiting, press Ctrl+c to kill it).

Typically what you want is to have a command that is running continuously and feeding updated status text when needed into lemonbar. One line at a time. An example of a simple clock would be `while true; do; date && sleep 1; done | lemonbar -p`.

You can program a status text generator in whatever way and language you desire. As long as you end up with a stream of lines of text. The code often needs some form of inter-process communication where you have multiple writers write to lemonbar "at once". Then you can have your window manager information, a clock, and current battery status for example all run in separate processes but still write to the same instance of lemonbar. In shellscripting this typically takes the form of a [named pipe](https://www.linuxjournal.com/content/using-named-pipes-fifos-bash). This pipe stream is then handled in a case statement before giving a new full status line to lemonbar. See the `up` file for an example of this.

## Formatting

You can format your text using special syntax to customize colors and add clickable areas etc. This takes the form of percentage signs and curly braces. For example: `echo '%{F#0000ff}blue text%{F-}' | lemonbar -p`. See `man lemonbar` for everything you can do.

## Fonts and icons

Lemonbar can only display bitmap fonts. In order to use non-bitmap fonts and icon fonts you need the [lemonbar-xft fork](https://github.com/drscream/lemonbar-xft). Specifying fonts is done with the `-f` flag. Specifying an XFT font size is done by appending `:size=SIZE` to the font name. A font can also be made bold with `:style=bold`. Example: `Ubuntu Mono:style=bold:size=12`.

# Using lemonbar with leftwm

Leftwm comes with the `leftwm-state` program which can print out continuous status of the window manager (tags, window title, current layout etc.). What the output looks like is further defined by [.liquid template files](https://github.com/leftwm/leftwm/wiki/Customizing-Themes-with-%60liquid%60-templates). See `template.liquid` and `sizes.liquid` for  examples of utilizing this specifically for lemonbar. In the `up` script the `leftwm-state` command is run into a named pipe (along with other commands) which is then sorted and given to lemonbar.

# Ready made status text generators or script managers

If you don't want to program lemonbar entirely from scratch here are some of the many options that exists:
- [luastatus](https://github.com/shdown/luastatus) - status text generator
- [slstatus](https://tools.suckless.org/slstatus/) - status text generator
- [succade](https://github.com/domsson/succade) - manages your status bar scripts and launches lemonbar for you
- [lemon](https://github.com/algor512/lemon) - another manager

# Links and resources

- [Lemonbar github repo](https://github.com/LemonBoy/bar)
- [ArchWiki](https://wiki.archlinux.org/title/Lemonbar)
- [How to lemonbar blog post](https://sacules.github.io/post/how-to-lemonbar)
- [Lemonbar example implementation for the bspwm window manager](https://github.com/baskerville/bspwm/tree/master/examples/panel)
- [Lemonbar: Don't Like Polybar... Just Remake It From Scratch Youtube video](https://www.youtube.com/watch?v=XBtLwpUHw4s)
- [About my lemonbar-xft fork YouTube video](https://www.youtube.com/watch?v=q7JxIO6Vddg)