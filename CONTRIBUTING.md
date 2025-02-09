# Contributing to LeftWM

Thank you for your interest in contributing to LeftWM!

Table of Contents:

1. [Feature Requests](#feature-requests)
2. [Bug Reports](#bug-reports)
3. [Patches / Pull Requests](#patches--pull-requests)
    1. [Testing](#testing)
    2. [Performance](#performance)
    3. [Code Documentation](#code-documentation)
    4. [Style](#style)
    5. [User Documentation](#user-documentation)

## Feature Requests

Feature requests should be reported in the
[LeftWM issue tracker](https://github.com/leftwm/leftwm/issues).

## Bug Reports

Bug reports should be reported in the
[LeftWM issue tracker](https://github.com/leftwm/leftwm/issues). 
Before reporting a bug, please check the troubleshooting steps in the README and previous issues.

## Patches / Pull Requests

All patches have to be sent on Github as [pull requests](https://github.com/leftwm/leftwm/pulls).

Please note that the minimum supported version of Rust capable of compiling LeftWM is Rust 1.52.0.

### Testing

To run the provided tests or any tests you have added use:

```
cargo test --all-targets --all-features
```
These tests are run by CI, but it is always easier to check before pushing.

### Code Documentation

Code documentation is generated with `cargo doc --all-features`.

The existing code can be used as a guidance here and the general rustfmt rules can be followed for formatting, which can be run with:
```
cargo fmt -- --check
```
Or many text editors/IDE's have a supported plugin that will auto-format your code, e.g. for vim there is [rust.vim](https://github.com/rust-lang/rust.vim).

### Style

Similar to documentation rustfmt along with clippy are used for general style guidance. Clippy can be run using:
```
cargo clippy --release
```
Again these are checked with CI, but it is always easier to check them before creating a pull request.

*Note: if you want to further improve the style you could also use these additional flags:*
```
cargo clippy -- -W clippy::pedantic -A clippy::must_use_candidate -A clippy::cast_precision_loss -A clippy::cast_possible_truncation -A clippy::cast_possible_wrap -A clippy::cast_sign_loss -A clippy::mut_mut

```

### User Documentation

When introducing new commands or config options it is helpful to provide some user level documentation and keep the initial PR message body updated with this documentation.

If possible please provide a snippet for the relevant wiki-article/section, so this can be updated accordingly as quick as possible.

Here are the wiki pages that must be updated once the PR is merged in `main` branch, if applicable:

- [External commands wiki page](https://github.com/leftwm/leftwm/wiki/External-Commands)
- [Config wiki page](https://github.com/leftwm/leftwm/wiki/Config)

#### Manual Page

If possible, please document your newly added commands/configuration options to the `leftwm` manual page, as this will help users
who need offline documentation available or a "quick look" at any command. To do so, search for `leftwm/doc/leftwm.1` and document your changes in there.

### Tips and Tricks

There is also a [tips and tricks](https://github.com/leftwm/leftwm/wiki/Contributing-to-Leftwm---Tips-and-Tricks) section in the wiki full of info from different contributors on the LeftWM team.
