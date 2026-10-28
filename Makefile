# Build and run the Raspberry Pi 4 BSP. See README.md.
#
# Needs the QNX SDP 8.0 environment: source qnxsdp-env.sh from the SDP
# first, which sets QNX_HOST and QNX_TARGET and puts mkifs on the PATH.

BUILD := build

CLANG ?= clang
LLD   ?= ld.lld

QEMU_DIR := $(BUILD)/qemu
BSP_DIR  := $(BUILD)/bsp

# The Rust startup: cargo profile "release" (default) or "dev".
TARGET      := aarch64-unknown-none-softfloat
PROFILE     ?= release
PROFILE_DIR := $(if $(filter dev,$(PROFILE)),debug,$(PROFILE))
STARTUP_LIB := target/$(TARGET)/$(PROFILE_DIR)/libstartup_bcm2711_rpi4.a
STARTUP_DIR := $(BUILD)/startup
STARTUP     := $(STARTUP_DIR)/startup-bcm2711-rpi4
IMAGE       := $(BUILD)/rpi4.ifs

ifeq ($(QNX_TARGET),)
$(error QNX_TARGET is not set: source qnxsdp-env.sh from the QNX SDP 8.0 first)
endif

# QNX's RPi4 BSP, as installed by the QNX Software Center into the SDP's
# bsp/ directory (the newest one, if there are several). Its prebuilt
# drivers go into the image, and its startup makes the reference image.
QNX_BSP_ZIP ?= $(lastword $(sort $(wildcard $(QNX_TARGET)/../../bsp/BSP_raspberrypi-bcm2711-rpi4_*.zip)))

# Options for startup-bcm2711-rpi4 in images/rpi4.build: -u reg takes the
# DTB address from x0, which is what the firmware (and fwstub) passes.
STARTUP_OPTS ?= -v -D miniuart -u reg

# mkifs's search path for the image's files: $(1), then the BSP's
# prebuilt binaries, then the SDP's. The first directory that has a file
# wins, which is how an image chooses its startup.
SDP_PATH := $(addprefix $(QNX_TARGET)/aarch64le/,boot/sys sbin usr/sbin bin usr/bin lib usr/lib)
BSP_PATH := $(addprefix $(abspath $(BSP_DIR))/prebuilt/aarch64le/,boot/sys sbin)
empty :=
space := $(empty) $(empty)
define mkifs
	@mkdir -p $(@D)
	sed 's|$${STARTUP_OPTS}|$(STARTUP_OPTS)|' $< > $@.build
	MKIFS_PATH=$(subst $(space),:,$(strip $(1) $(BSP_PATH) $(SDP_PATH))) mkifs $@.build $@
endef

.PHONY: all startup image run qemu bsp qnx-reference qnx-reference-run clean

all: image qemu

# ANCHOR: make_startup
# The startup program: the board crate as a static library, partially
# linked (ld -r) into the one relocatable object mkifs links into the
# image. -u _start pulls in the entry point and, from there, everything
# it uses.
startup:
	cargo build -p startup-bcm2711-rpi4 --target $(TARGET) --profile $(PROFILE)
	@mkdir -p $(STARTUP_DIR)
	$(LLD) -r -u _start -T oscraft-startup/startup.ld -o $(STARTUP) $(STARTUP_LIB)
# ANCHOR_END: make_startup

# The image, with our startup: build/startup comes first in mkifs's path.
image: $(IMAGE)

$(IMAGE): images/rpi4.build startup $(BSP_DIR)/.unpacked
	$(call mkifs,$(abspath $(STARTUP_DIR)))

run: $(IMAGE) qemu
	tools/qemu/run.sh $< -no-reboot

# QEMU support files: firmware stub and device tree.
qemu: $(QEMU_DIR)/fwstub.elf $(QEMU_DIR)/bcm2711-rpi-4-b.dtb

$(QEMU_DIR)/fwstub.o: tools/qemu/fwstub/fwstub.S
	@mkdir -p $(@D)
	$(CLANG) --target=aarch64-none-elf -c $< -o $@

$(QEMU_DIR)/fwstub.elf: $(QEMU_DIR)/fwstub.o tools/qemu/fwstub/fwstub.ld
	$(LLD) -T tools/qemu/fwstub/fwstub.ld $< -o $@

$(QEMU_DIR)/bcm2711-rpi-4-b.dtb: tools/rpi4/fetch-firmware.sh
	tools/rpi4/fetch-firmware.sh $(@F) $@

# QNX's prebuilt RPi4 BSP binaries, unpacked from the BSP zip.
bsp: $(BSP_DIR)/.unpacked

$(BSP_DIR)/.unpacked: $(QNX_BSP_ZIP)
	@test -n "$<" || { echo "QNX RPi4 BSP zip not found: install the BSP with the QNX Software Center, or set QNX_BSP_ZIP" >&2; exit 1; }
	@mkdir -p $(@D)
	unzip -oq $< 'prebuilt/*' 'binary_files_with_symbols/*' -d $(BSP_DIR)
	touch $@

# The reference image: the same build file with QNX's own startup.
qnx-reference: $(BUILD)/qnx-reference.ifs

$(BUILD)/qnx-reference.ifs: images/rpi4.build $(BSP_DIR)/.unpacked
	$(call mkifs,)

qnx-reference-run: $(BUILD)/qnx-reference.ifs qemu
	tools/qemu/run.sh $< -no-reboot

clean:
	rm -rf $(BUILD)
