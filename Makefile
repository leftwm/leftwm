# The flags to pass to the `cargo build` command
BUILDFLAGS := --release

# Absolute path to project directory, required for symbolic links
# or when 'make' is run from another directory.
# - credit: https://stackoverflow.com/a/23324703/2726733
ROOT_DIR := $(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))

SHARE_DIR := /usr/share/xsessions
TARGET_DIR := /usr/local/bin

# default rule is to run build/test
all: build test

# runs tests and linters
test:
	cd $(ROOT_DIR) && cargo test --all-targets --all-features
	cd $(ROOT_DIR) && cargo fmt -- --check
	cd $(ROOT_DIR) && cargo clippy

test-nix:
	cd $(ROOT_DIR) && NIX_PATH=nixpkgs=channel:nixos-unstable nix flake check --extra-experimental-features "nix-command flakes" --verbose
	cd $(ROOT_DIR) && NIX_PATH=nixpkgs=channel:nixos-unstable nix build --extra-experimental-features "nix-command flakes" --verbose

test-full: test
	cargo clippy --\
		-D warnings\
		-W clippy::pedantic\
		-A clippy::must_use_candidate\
		-A clippy::cast_precision_loss\
		-A clippy::cast_possible_truncation\
		-A clippy::cast_possible_wrap\
		-A clippy::cast_sign_loss\
		-A clippy::mut_mut

test-full-nix: test-full test-nix

# builds the project
build:
	cd $(ROOT_DIR) && cargo build ${BUILDFLAGS}

# removes the generated binaries
clean:
	cd $(ROOT_DIR) && cargo clean
	rm $(ROOT_DIR)/result
	@echo "build files have been cleaned"

# builds the project and installs the binaries (and .desktop)
install: build
	sudo cp $(ROOT_DIR)/leftwm.desktop /usr/share/xsessions/
	sudo cp $(ROOT_DIR)/leftwm/doc/leftwm.1 /usr/local/share/man/man1/leftwm.1
	sudo install -s -Dm755\
		$(ROOT_DIR)/target/release/leftwm\
		$(ROOT_DIR)/target/release/leftwm-worker\
		$(ROOT_DIR)/target/release/lefthk-worker\
		$(ROOT_DIR)/target/release/leftwm-state\
		$(ROOT_DIR)/target/release/leftwm-check\
		$(ROOT_DIR)/target/release/leftwm-command\
		-t $(TARGET_DIR)
	cd $(ROOT_DIR) && cargo clean
	@echo "binaries, '.desktop' file and manual page have been installed"

# build the project and links the binaries, will also install the .desktop file
install-linked: build
	sudo cp $(ROOT_DIR)/leftwm.desktop $(SHARE_DIR)/
	sudo cp $(ROOT_DIR)/leftwm/doc/leftwm.1 /usr/local/share/man/man1/leftwm.1
	sudo ln -sf $(ROOT_DIR)/target/release/leftwm $(TARGET_DIR)/leftwm
	sudo ln -sf $(ROOT_DIR)/target/release/leftwm-worker $(TARGET_DIR)/leftwm-worker
	sudo ln -sf $(ROOT_DIR)/target/release/lefthk-worker $(TARGET_DIR)/lefthk-worker
	sudo ln -sf $(ROOT_DIR)/target/release/leftwm-state $(TARGET_DIR)/leftwm-state
	sudo ln -sf $(ROOT_DIR)/target/release/leftwm-check $(TARGET_DIR)/leftwm-check
	sudo ln -sf $(ROOT_DIR)/target/release/leftwm-command $(TARGET_DIR)/leftwm-command
	@echo "binaries have been linked, manpage and '.desktop' file have been installed"

# same as above, but builds the project in debug mode (no optimisations, faster builds)
install-linked-dev:
	cd $(ROOT_DIR) && cargo build
	sudo cp $(ROOT_DIR)/leftwm.desktop $(SHARE_DIR)/
	sudo cp $(ROOT_DIR)/leftwm/doc/leftwm.1 /usr/local/share/man/man1/leftwm.1
	sudo ln -sf $(ROOT_DIR)/target/debug/leftwm $(TARGET_DIR)/leftwm
	sudo ln -sf $(ROOT_DIR)/target/debug/leftwm-worker $(TARGET_DIR)/leftwm-worker
	sudo ln -sf $(ROOT_DIR)/target/debug/lefthk-worker $(TARGET_DIR)/lefthk-worker
	sudo ln -sf $(ROOT_DIR)/target/debug/leftwm-state $(TARGET_DIR)/leftwm-state
	sudo ln -sf $(ROOT_DIR)/target/debug/leftwm-check $(TARGET_DIR)/leftwm-check
	sudo ln -sf $(ROOT_DIR)/target/debug/leftwm-command $(TARGET_DIR)/leftwm-command
	@echo "binaries have been linked, manpage and '.desktop' file have been linked."


# uninstalls leftwm from the system, no matter if installed via 'install', 'install-linked' or 'install-linked-dev'
uninstall:
	sudo rm -f $(SHARE_DIR)/leftwm.desktop
	sudo rm /usr/local/share/man/man1/leftwm.1
	sudo rm -f\
		$(TARGET_DIR)/leftwm\
		$(TARGET_DIR)/leftwm-worker\
		$(TARGET_DIR)/lefthk-worker\
		$(TARGET_DIR)/leftwm-state\
		$(TARGET_DIR)/leftwm-check\
		$(TARGET_DIR)/leftwm-command
	@echo "binaries and manpage have been uninstalled and '.desktop' file has been removed"
