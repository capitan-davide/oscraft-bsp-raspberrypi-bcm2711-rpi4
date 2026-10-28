# Build and run the Raspberry Pi 4 BSP. See README.md.

BUILD   := build
QNX_SDP ?= /opt/qnx/qnx800
QNX_BSP_ZIP ?= $(QNX_SDP)/bsp/BSP_raspberrypi-bcm2711-rpi4_be-800_SVN1028919_JBN670.zip

CLANG ?= clang
LLD   ?= ld.lld

QEMU_DIR := $(BUILD)/qemu
BSP_DIR  := $(BUILD)/bsp

# Options for startup-bcm2711-rpi4 in images/rpi4.build: -u reg takes the
# DTB address from x0, which is what the firmware (and fwstub) passes.
STARTUP_OPTS ?= -v -D miniuart -u reg

# mkifs, run with the SDP environment (sourced by bash: the script does not
# support every shell). MKIFS_PATH is the search path for the image's
# files; $(1) is put in front of it, to choose which startup is found.
SDP_ENV  := . $(QNX_SDP)/qnxsdp-env.sh >/dev/null
SDP_PATH := $(addprefix $(QNX_SDP)/target/qnx/aarch64le/,boot/sys sbin usr/sbin bin usr/bin lib usr/lib)
BSP_PATH := $(addprefix $(abspath $(BSP_DIR))/prebuilt/aarch64le/,boot/sys sbin)
define mkifs
	@mkdir -p $(@D)
	sed 's|$${STARTUP_OPTS}|$(STARTUP_OPTS)|' $< > $@.build
	bash -c '$(SDP_ENV); MKIFS_PATH=$(subst $(eval) ,:,$(strip $(1) $(BSP_PATH) $(SDP_PATH))) mkifs $@.build $@'
endef

.PHONY: all qemu bsp qnx-reference qnx-reference-run clean

all: qemu

# QEMU support files: firmware stub and device tree.
qemu: $(QEMU_DIR)/fwstub.elf $(QEMU_DIR)/bcm2711-rpi-4-b.dtb

$(QEMU_DIR)/fwstub.o: tools/qemu/fwstub/fwstub.S
	@mkdir -p $(@D)
	$(CLANG) --target=aarch64-none-elf -c $< -o $@

$(QEMU_DIR)/fwstub.elf: $(QEMU_DIR)/fwstub.o tools/qemu/fwstub/fwstub.ld
	$(LLD) -T tools/qemu/fwstub/fwstub.ld $< -o $@

$(QEMU_DIR)/bcm2711-rpi-4-b.dtb: tools/rpi4/fetch-firmware.sh
	tools/rpi4/fetch-firmware.sh $(@F) $@

# QNX's prebuilt RPi4 BSP binaries (drivers, and the reference startup).
bsp: $(BSP_DIR)/.unpacked

$(BSP_DIR)/.unpacked: $(QNX_BSP_ZIP)
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
