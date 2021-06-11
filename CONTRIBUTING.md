# Contributing to Leftwm

Thank you for your interest in contributing to Leftwm!

Table of Contents:

1. [Feature Requests](#feature-requests)
2. [Bug Reports](#bug-reports)
3. [Patches / Pull Requests](#patches--pull-requests)
    1. [Testing](#testing)
    2. [Performance](#performance)
    3. [Documentation](#documentation)
    4. [Style](#style)

## Feature Requests

Feature requests should be reported in the
[Leftwm issue tracker](https://github.com/leftwm/leftwm/issues).

## Bug Reports

Bug reports should be reported in the
[Leftwm issue tracker](https://github.com/leftwm/leftwm/issues). 
Before reporting a bug, please check the troubleshooting steps in the README and previous issues.

## Patches / Pull Requests

All patches have to be sent on Github as [pull requests](https://github.com/leftwm/leftwm/pulls).

Please note that the minimum supported version of Leftwm is Rust 1.51.0. 

### Testing

To run the provided tests or any tests you have added use:

```
cargo test --all-targets --all-features
```
These tests are run by CI, but it is always easier to check before pushing.

### Documentation

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

### User Documentation

When introducing new commands or config options it is helpfull to provide some user level documentation and keep the initial PR message body updated with this documentation.
If possilbe please provide a snippet for the relevant wiki-article/section, so this can be updated acordingly as quick as possible.

### Tips and Tricks

There is also a [tips and tricks](https://github.com/leftwm/leftwm/wiki/Contributing-to-Leftwm---Tips-and-Tricks) section in the wiki full of info from different contributors on the LeftWM team.

### User Documentatio

When introducing new commands or config options it is helpfull to provide some user level documentation and keep the initial PR message body updated with this documentation.
If possilbe please provide a snippet for the relevant wiki-article/section, so this can be updated acordingly as quick as possible.
