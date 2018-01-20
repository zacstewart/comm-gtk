TARGET = comm-gtk

APP_NAME = Comm.app
ASSETS_DIR = assets
RELEASE_DIR = target/release
RESOURCES_DIR = resources
APP_TEMPLATE = $(ASSETS_DIR)/macos/$(APP_NAME)
APP_DIR = $(RELEASE_DIR)/macos
APP_BINARY = $(RELEASE_DIR)/$(TARGET)
APP_BINARY_DIR = $(APP_DIR)/$(APP_NAME)/Contents/MacOS
APP_RESOURCES_DIR = $(APP_DIR)/$(APP_NAME)/Contents/Resources

DMG_NAME = Comm.dmg
DMG_DIR = $(RELEASE_DIR)/macos

vpath $(TARGET) $(RELEASE_DIR)
vpath $(APP_NAME) $(APP_DIR)
vpath $(DMG_NAME) $(APP_DIR)

all: help

help: ## Prints help for targets with comments
	@grep -E '^[a-zA-Z._-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

binary: | $(APP_BINARY) ## Build release binary with cargo
$(APP_BINARY):
	cargo build --release

app: | $(APP_NAME) ## Clone Comm.app template and mount binary
$(APP_NAME): $(TARGET) $(APP_TEMPLATE)
	@mkdir -p $(APP_BINARY_DIR)
	@cp -fRp $(APP_TEMPLATE) $(APP_DIR)
	@cp -fp $(APP_BINARY) $(APP_BINARY_DIR)
	@cp -fRp $(RESOURCES_DIR)/* $(APP_RESOURCES_DIR)
	@echo "Created '$@' in '$(APP_DIR)'"

dmg: | $(DMG_NAME) ## Pack Comm.app into .dmg
$(DMG_NAME): $(APP_NAME)
	@echo "Packing disk image..."
	@hdiutil create $(DMG_DIR)/$(DMG_NAME) \
		-volname "Comm" \
		-fs HFS+ \
		-srcfolder $(APP_DIR) \
		-ov -format UDZO
	@echo "Packed '$@' in '$(APP_DIR)'"

install: $(DMG_NAME) ## Mount disk image
	@open $(DMG_DIR)/$(DMG_NAME)

.PHONY: app binary clean dmg install

clean: ## Remove all artifacts
	-rm -rf $(APP_BINARY)
	-rm -rf $(APP_DIR)
