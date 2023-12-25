# Absolute path to project directory, required for symbolic links
# or when 'make' is run from another directory.
# - credit: https://stackoverflow.com/a/23324703/2726733
ROOT_DIR := $(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))

SHARE_DIR := /usr/share
TARGET_DIR := /usr/local/bin

# Set default profile if unset
ifndef profile
	profile := optimized
endif

# Set corresponding target folder name
ifeq ($(profile),dev)
	folder := debug
else
	folder := $(profile)
endif

# default rule is to run build/test
all: build test

# runs tests and linters
test:
	cd $(ROOT_DIR) && cargo test --all-targets --all-features
	cd $(ROOT_DIR) && cargo fmt -- --check
	cd $(ROOT_DIR) && cargo clippy -- -D warnings -W clippy::pedantic

test-nix:
	cd $(ROOT_DIR) && NIX_PATH=nixpkgs=channel:nixos-unstable nix flake check --extra-experimental-features "nix-command flakes" --verbose
	cd $(ROOT_DIR) && NIX_PATH=nixpkgs=channel:nixos-unstable nix build --extra-experimental-features "nix-command flakes" --verbose

# deprecated
test-full: test
	@printf '\n\033[38;5;9mNote: \"make test-full\" is deprecated. You can use \"make test\" instead with the same functionality\n'

test-full-nix: test test-nix

# builds the project
build:
	@echo "Building with $(profile) profile"
	@echo "Change the profile by adding profile=release or profile=dev to the command"
	cd $(ROOT_DIR) && cargo build --profile $(profile)

# removes the generated binaries
clean:
	cd $(ROOT_DIR) && cargo clean
	rm $(ROOT_DIR)/result
	@echo "Build files have been cleaned"

# builds the project and installs the binaries (and .desktop)
install: build
	sudo cp $(ROOT_DIR)/leftwm.desktop $(SHARE_DIR)/xsessions/
	sudo cp $(ROOT_DIR)/leftwm/doc/leftwm.1 /usr/local/share/man/man1/leftwm.1
	[ -d '/usr/share/leftwm' ] || sudo mkdir $(SHARE_DIR)/leftwm
	sudo cp -rL $(ROOT_DIR)/examples $(SHARE_DIR)/leftwm
	sudo install -s -Dm755\
		$(ROOT_DIR)/target/$(folder)/leftwm\
		$(ROOT_DIR)/target/$(folder)/leftwm-log\
		$(ROOT_DIR)/target/$(folder)/leftwm-worker\
		$(ROOT_DIR)/target/$(folder)/lefthk-worker\
		$(ROOT_DIR)/target/$(folder)/leftwm-state\
		$(ROOT_DIR)/target/$(folder)/leftwm-check\
		$(ROOT_DIR)/target/$(folder)/leftwm-command\
		-t $(TARGET_DIR)
	cd $(ROOT_DIR) && cargo clean
	@echo "Binaries, '.desktop' file, manpage, theme and config templates have been installed"

# Function to build and link a specific profile
install-linked: build
	sudo cp $(ROOT_DIR)/leftwm.desktop $(SHARE_DIR)/
	sudo cp $(ROOT_DIR)/leftwm/doc/leftwm.1 /usr/local/share/man/man1/leftwm.1
	[ -d '/usr/share/leftwm' ] || sudo mkdir $(SHARE_DIR)/leftwm
	sudo cp -rL $(ROOT_DIR)/examples $(SHARE_DIR)/leftwm
	sudo ln -sf $(ROOT_DIR)/target/$(folder)/leftwm $(TARGET_DIR)/leftwm
	sudo ln -sf $(ROOT_DIR)/target/$(folder)/leftwm-log $(TARGET_DIR)/leftwm-log
	sudo ln -sf $(ROOT_DIR)/target/$(folder)/leftwm-worker $(TARGET_DIR)/leftwm-worker
	sudo ln -sf $(ROOT_DIR)/target/$(folder)/lefthk-worker $(TARGET_DIR)/lefthk-worker
	sudo ln -sf $(ROOT_DIR)/target/$(folder)/leftwm-state $(TARGET_DIR)/leftwm-state
	sudo ln -sf $(ROOT_DIR)/target/$(folder)/leftwm-check $(TARGET_DIR)/leftwm-check
	sudo ln -sf $(ROOT_DIR)/target/$(folder)/leftwm-command $(TARGET_DIR)/leftwm-command
	@echo "binaries have been linked, '.desktop' file, manpage, theme and config templates have been installed"

# Uninstalls leftwm from the system.
uninstall:
	sudo rm -f $(SHARE_DIR)/leftwm.desktop
	sudo rm /usr/local/share/man/man1/leftwm.1
	sudo rm -rf $(SHARE_DIR)/leftwm
	sudo rm -f\
		$(TARGET_DIR)/leftwm\
		$(TARGET_DIR)/leftwm-log\
		$(TARGET_DIR)/leftwm-worker\
		$(TARGET_DIR)/lefthk-worker\
		$(TARGET_DIR)/leftwm-state\
		$(TARGET_DIR)/leftwm-check\
		$(TARGET_DIR)/leftwm-command
	@echo "Binaries and manpage have been uninstalled and '.desktop' file, theme and config templates have been removed"
