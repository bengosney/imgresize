.PHONY: all test help package-linux package-windows package build-linux build-windows build clean release

BINARY_NAME := imgsize
BUILD_DIR := target

LINUX_TARGET := x86_64-unknown-linux-gnu
WINDOWS_TARGET := x86_64-pc-windows-gnu

SOURCE_FILES := $(shell find src -type f -name "*.rs")
VERSION := $(shell cargo pkgid | cut -f 2 -d\#)


help: ## Display this help
	@echo
	@echo "\033[1mimgsize Makefile\033[0m"
	@echo
	@echo "Usage: make [target]"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

# Build the binarys
$(BUILD_DIR)/%/release/$(BINARY_NAME): $(SOURCE_FILES) Cargo.toml Cargo.lock
	cargo build --release --target $*

$(BUILD_DIR)/%/release/$(BINARY_NAME).exe: $(SOURCE_FILES) Cargo.toml Cargo.lock
	cargo build --release --target $*

build-linux: $(BUILD_DIR)/$(LINUX_TARGET)/release/$(BINARY_NAME) ## Build the Linux binary

build-windows: $(BUILD_DIR)/$(WINDOWS_TARGET)/release/$(BINARY_NAME).exe ## Build the Windows binary

build: build-linux build-windows ## Build all the binaries

# Package the binaries
$(BUILD_DIR)/$(BINARY_NAME)-$(VERSION)-linux.tar.gz: $(BUILD_DIR)/$(LINUX_TARGET)/release/$(BINARY_NAME) README.md LICENSE
	tar -czvf $@ --transform 's,.*/,,g' $^

$(BUILD_DIR)/$(BINARY_NAME)-$(VERSION)-windows.zip: $(BUILD_DIR)/$(WINDOWS_TARGET)/release/$(BINARY_NAME).exe README.md LICENSE
	zip -j $@ $^

package-linux: $(BUILD_DIR)/$(BINARY_NAME)-$(VERSION)-linux.tar.gz ## Package the Linux binary

package-windows: $(BUILD_DIR)/$(BINARY_NAME)-$(VERSION)-windows.zip ## Package the Windows binary

package: package-windows package-linux ## Package all the binaries

# Create a GitHub release
release: package ## Create a GitHub release
	gh release create $(VERSION) $(BUILD_DIR)/$(BINARY_NAME)-$(VERSION)-linux.tar.gz $(BUILD_DIR)/$(BINARY_NAME)-$(VERSION)-windows.zip --title "Release $(VERSION)" --generate-notes

# Clean the build artifacts
clean: ## Clean the build artifacts
	cargo clean
	rm -rf $(BUILD_DIR)
