# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added

### Fixed

## [0.4.2]
### Added
- Created `Matrix` chatrooms as an pure OSS alternative to `Discord`
- Added check to `leftwm-check` to ensure all necessary binaries are installed (via PR #1077 by @mautamu)
- Make Workspace IDs unique and predictable (via PR 1043 by @hertg)
- Add optional behaviour Settings for `FocusNextTag` and `FocusPreviousTag` to skip or expricitly go to (un-)used tags (via #1086 by @Silicasandwhich)
- Enhance visibility of configuration errors in `leftwm-check` output (via PR 1072 by @realGWM)
- Add `SwapWindowTop` command as an alternative to `MoveWindowTop` (via #1005 by @emiliode)
- Reposition mouse cursor to bottom-right corner of window when resizing (via #1009 by @nemalex)
- The currently supported MSRV is 1.65.0

### Fixed
- Fix build error caused by deprecated macro in `git_version` (via PR #1075 by VuiMuich)
- Fix `template.liquid` in `basic_lemonbar` example theme (via PR #1080 by VuiMuich)
- Fix to correctly restore order of windows after exiting fullscreen (via PR #1053 by @jean-santos)
- Fix that focus is found after the focused window got destroyed (via PR # 1055 by @jean-santos)
- Correct writing and formatting of docs in `themes/basic_eww` (via PR #107 by @NAHO)
- Various performance improvements for `command pipe`, `event loop`, `set_layout` and more (via PRs #1016, #1017, #1018, #1067, #1068 by @vrmiguel)
- Made AUR packages better compliant to packaging specs and use correct detection for `systemd` based systems (via gists by @eclairevoyant)
- Fix minor regression where `res_name` and `name` window properties were no longer checked by window rules (via #1002 by @guigot)
- Simplified async test (via PR #1069 by @vrmiguel)

## [0.4.1]
### Fixed
- Temp fix for MainWidth inconsistency until layout-lib
- Fix crash when swapping tag at startup
- Swap for new copr repo
- Using args[0] instead of current_exe() under OpenBSD. current_exe() is not supported in this OS.
- replace `<module-name>/mod.rs` with `<module-name>.rs`
- Fix manual install instructions
- Change default layout mode to `Tag`
- Exclude NetBSD in `build.rs` and avoid build-hangs
- Print `lefthk` warning only when no binary present
- use `implicit_some` extension in ron deserializer
- small update to readme
- Make help pages work
- better clap and fix broken `workspace` flag for `leftwm-state`
- Fix binary arguments (clap)
- Some fixes for the `Makefile` and `flake.nix`
- Make `test-full` fail on warnings
- bump several depedencies
- Make `leftwm --version` work
- Improve output of `leftwm-check --verbose`
- Fix `--no-default-features` build failing
- Refactor: Move xlib display server to it's own crate
- Add link to `leftfwm.desktop` to the README
- Fix new windows always getting stacked topmost
- Also a bunch of little papercuts and code cleanup
### Added
- feat(config): allow regexps in window_rules for window_class,
- Option to hide border when only one window is visible
- added lefthk feature checks and errors
- add features log to leftwm-check.
- handle theme templates within Makefile
- Add optimized build profile
- Add leftwm config subcommand
- Add `background_color` to theme config
- update theme examples to ron
- update eww example
- Feature: Multiple `up` scripts
- Add install-linked-dev to Makefile
- feat: Add a `spawn_as_type` to the `window_rules` configuration
- Added nix tests to Makefile.
- Added a few options to the pr template
- Add a few window rules
### Minimum Supported Rust Version
- The currently supported MSRV is 1.60.0

## [0.4.0]
### Fixed
- Fixed again a lot of small papercuts
- Command parity between keybinds and external aka `leftwm-command` commands
- Improved window snapping
- Improved behaviour of floating windows
- `dock` windows not recieving `click` after some window was `fullscreen`
### Added
- Commands `AttachScratchpad`, `ReleaseScratchpad`, `NextScratchpadWindow`, `PrevScratchpadWindow`
- Commands `MoveWindowToNextTag`, `MoveWindowToPreviousTag`
- Window rules by `WINDOW_CLASS` or `WINDOW_TITLE`
- `test-full` to `Makefile` using padantic clippy as well
- Introduced `lefthk` as default daemon to handle keybinds
- Changed config language from `TOML` to `RON`
- Journald logging is default for AUR packages and when building from source
- Tags support `urgent` flag
- `sloppy_mouse_follows_focus`
- `disable_window_snap`
### Minimum Supported Rust Version
- The currently supported MSRV is 1.59.0

## [0.3.0]
### Fixed
### Added
### Minimum Supported Rust Version
- The currently supported MSRV is 1.52.0

## [0.2.10] - 2021-12-04
To view the full list of changes visit the [milestone](https://github.com/leftwm/leftwm/milestone/1?closed=1).

### Fixed
- Fixed a command in basic-eww theme
- Window width resetting on tag resize
- Render emoji's in window titles
- Greatly improved dialog handling
- Many focus fixes
- leftwm-check not running needed checks
- Xterm loading times
- Floating windows going behind newly tiled windows
### Added
- Windows opened from a terminal will spawn on the same tag
- Added Makefile
- When in ClickTo, windows are focused when moved/resized
- ToggleFloating and TileToFloating commands
- Manpage

## [0.2.9] - 2021-10-11
### Fixed
- lots of small papercuts
- xdg-autostart
- Better config checking
### Added
- Layouts by Tags not Workspaces
- New Commands
- Split into two crates

## [0.2.8] - 2021-7-6
### Fixed
- So So many Bug fixes
### Added
- Add the command 'leftwm-command' to run commands

## [0.2.7] - 2021-4-6
### Fixed
- `SwapTags` now works properly for multi-workspace users
- Xlib events now read asynchronously
- Weird focus issues
- Some floating windows are now sized properly
- Floating windows are now always on top
- Basic-polybar theme now reports seconds correctly
- Startup banners no longer take up the whole screen
- Bug Fixes in tests
- Window order is now restored on reload
- Several code refactors
### Added
#### Layouts
- MainAndHorizontalStack layout
- CenterMainBalanced layout
- Monocle layout
- RightWiderLeftStack layout
- LeftWiderRightStack layout
- MainAndDeck layout
#### Configuration
- Active layouts may now be set in config.toml with `layouts`
- GotoTag sends screen to previously viewed tag
	- Added `disable_current_tag_swap` to config.toml to maintain old behaviour
- Mousekey for floating drag/resize can now be set with `mousekey` in config.toml
- FloatingToTile command that allows a window to be set to float in config.toml
- Theme Margins [top/bottom side] or [top right bottom left] added for theme.toml
#### Miscellaneous
- Added `leftwm-check` binary that allows for configuration files (config.toml, theme.toml) to be checked.
- Autostart applications can now be disabled by setting X-Gnome-Autostart to false in the appropriate .desktop file
- SetLayout command added to set specific layouts externally
- Lots of notes have been added to the library documentation

## [0.2.6] - 2021-1-29
### Fixed
- Performance improvements (async update)
- Bug fixes
### Added
- layout sizing by user (Mod+h / Mod+l)

## [0.2.5] - 2020-11-13
### Fixed
- Performance improvements
- Bug fixes
### Added

## [0.2.4] - 2020-8-10
### Fixed
- Bug fixes
### Added
- New Layouts

## [0.2.3] - 2020-5-15
### Fixed
- Sizing and loading issues with docks
- Many small bug fixes
### Added
- Much better logging
- `workspace.layout` to `leftwm-state` output.
- Layouts are now preserved between reloads.

## [0.2.2] - 2019-12-27
### Fixed
- fix build with latest version of rust

## [0.2.1] - 2019-12-16
### Fixed
- Stability and performance updates
- EWMH compatibility fixes
- The way floating windows move with workspaces
### Added
- layout Grid
- layout Fibonacci


## [0.1.10] - 2019-06-17
### Fixed
- Stability and performance updates
- EWMH compatibility fixes
- Improvements to theme system
### Added
- Better default key bindings


## [0.1.9] - 2019-05-05
### Fixed
- Fix Several small papercuts and bug fixes
### Added
- reloading while keeping current window state
- Keyboard navigation between workspaces
- min/max window size support
- callback to call theme scripts on new window


## [0.1.8] - 2019-04-19
### Fixed
- Fix Several small papercuts and bug fixes


## [0.1.7] - 2019-04-15
### Fixed
- Fix issues with multiscreen bars
- Cleanup and refactor code
### Added
- leftwm-state to get state info from leftwm for bars
- Added template system for theme creation


## [0.1.6] - 2019-04-5
### Fixed
- Bugs and issues with bars
- Several small bugs fixes
### Added
- Renaming of tags
- keycombos to refocusing windows
- Added layout - Main/Stacked
