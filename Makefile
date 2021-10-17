# The flags to pass to the `cargo build` command
BUILDFLAGS := --release --features=journald

# Absolute path to project directory, required for symbolic links
# or when 'make' is run from another directory.
# - credit: https://stackoverflow.com/a/23324703/2726733
ROOT_DIR:=$(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))

# default rule is to run build/test
all: build test

# runs tests and linters
test: 
	cargo test --all-targets --all-features
	cargo fmt -- --check
	cargo clippy --release

# builds the project
build:
	cd $(ROOT_DIR) && cargo build ${BUILDFLAGS}

# removes the generated binaries
clean:
	rm -rf $(ROOT_DIR)/target $(ROOT_DIR)/leftwm/target $(ROOT_DIR)/leftwm-core/target
	@echo "target directories have been cleaned"

# builds the project and installs the binaries (and .desktop)
install: build
	sudo cp $(ROOT_DIR)/leftwm.desktop /usr/share/xsessions/
	sudo install -s -Dm755 $(ROOT_DIR)/target/release/leftwm $(ROOT_DIR)/target/release/leftwm-worker $(ROOT_DIR)/target/release/leftwm-state $(ROOT_DIR)/target/release/leftwm-check $(ROOT_DIR)/target/release/leftwm-command -t /usr/bin
	@echo "binaries and '.desktop' file have been installed"

# build the project and links the binaries, will also install the .desktop file
install-dev: build
	sudo cp $(ROOT_DIR)/leftwm.desktop /usr/share/xsessions/
	sudo ln -s $(ROOT_DIR)/target/release/leftwm /usr/bin/leftwm
	sudo ln -s $(ROOT_DIR)/target/release/leftwm-worker /usr/bin/leftwm-worker
	sudo ln -s $(ROOT_DIR)/target/release/leftwm-state /usr/bin/leftwm-state
	sudo ln -s $(ROOT_DIR)/target/release/leftwm-check /usr/bin/leftwm-check
	sudo ln -s $(ROOT_DIR)/target/release/leftwm-command /usr/bin/leftwm-command
	@echo "binaries have been linked and '.desktop' file installed"

# uninstalls leftwm from the system, no matter if installed via 'install' or 'install-dev'
uninstall:
	sudo rm -f /usr/share/xsessions/leftwm.desktop
	sudo rm -f /usr/bin/leftwm /usr/bin/leftwm-worker /usr/bin/leftwm-state /usr/bin/leftwm-check /usr/bin/leftwm-command
	@echo "binaries have been uninstalled and '.desktop' file removed"
