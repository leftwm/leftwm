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

### Setting up your Envriroment

This section has some tips and tricks provided by contributors to help you get on your way in contributing.

Tip 1. To make your life easier once your done coding, adding this function
```
checkleft() {
        if cargo fmt ; then
          if cargo test; then
            if cargo clippy --all-targets --all-features ; then
                if git add -A $1 ; then
                    if git status ; then
                        git commit -S -m "$2"
                    else
                        echo "Status failed"
                     fi
                 else
                     echo "Add failed"
                fi
            else
                echo "Clippy failed"
            fi
          else
            echo "Test failed"
          fi
         else
                echo "FMT failed"
          fi
}
```
to your ```~/.bashrc``` and running it will automatically run all the cargo checks and commit the specified files to git. It assumes you have a gpg-key, remove ```-S ``` flag from git commit if not.
Example usage:
```
# Example, adds all of src/, Cargo.toml, and Cargo.lock to be committed.
# You can pass anything that would go to git add -A 
checkleft 'src/ Cargo.toml Cargo.lock' 'Commit message'
```
