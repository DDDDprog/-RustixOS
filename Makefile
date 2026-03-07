# RustixOS - Advanced Rust Kernel Makefile
# Professional build system for multi-architecture kernel

# Project configuration
PROJECT_NAME := rustix-kernel
VERSION := 0.1.0
BUILD_DIR := build
TARGET_DIR := target
ISO_DIR := $(BUILD_DIR)/iso
GRUB_DIR := $(ISO_DIR)/boot/grub

# Architecture configuration
ARCH ?= x86_64
SUPPORTED_ARCHS := x86_64 x86 arm aarch64

# Build configuration
BUILD_MODE ?= debug
RUST_TARGET_x86_64 := x86_64-unknown-none
RUST_TARGET_x86 := i686-unknown-none
RUST_TARGET_arm := armv7-unknown-none-gnueabihf
RUST_TARGET_aarch64 := aarch64-unknown-none

# Toolchain configuration
RUST_TARGET := $(RUST_TARGET_$(ARCH))
CARGO_FLAGS := --target $(RUST_TARGET)

ifeq ($(BUILD_MODE),release)
    CARGO_FLAGS += --release
    TARGET_SUBDIR := release
else
    TARGET_SUBDIR := debug
endif

# Cross-compilation toolchains
CC_x86_64 := gcc
LD_x86_64 := ld
OBJCOPY_x86_64 := objcopy
OBJDUMP_x86_64 := objdump

CC_x86 := i686-elf-gcc
LD_x86 := i686-elf-ld
OBJCOPY_x86 := i686-elf-objcopy
OBJDUMP_x86 := i686-elf-objdump

CC_arm := arm-none-eabi-gcc
LD_arm := arm-none-eabi-ld
OBJCOPY_arm := arm-none-eabi-objcopy
OBJDUMP_arm := arm-none-eabi-objdump

CC_aarch64 := aarch64-none-elf-gcc
LD_aarch64 := aarch64-none-elf-ld
OBJCOPY_aarch64 := aarch64-none-elf-objcopy
OBJDUMP_aarch64 := aarch64-none-elf-objdump

# Select toolchain based on architecture
CC := $(CC_$(ARCH))
LD := $(LD_$(ARCH))
OBJCOPY := $(OBJCOPY_$(ARCH))
OBJDUMP := $(OBJDUMP_$(ARCH))

# File paths
KERNEL_BIN := $(TARGET_DIR)/$(RUST_TARGET)/$(TARGET_SUBDIR)/$(PROJECT_NAME)
KERNEL_ELF := target/x86_64-unknown-none/debug/kernel
KERNEL_IMG := $(BUILD_DIR)/kernel-$(ARCH).img
ISO_FILE := $(BUILD_DIR)/$(PROJECT_NAME)-$(ARCH).iso

# Assembly source files
ASM_SOURCES_x86_64 := 
ASM_SOURCES_x86 := 
ASM_SOURCES_arm := 
ASM_SOURCES_aarch64 := src/arch/aarch64/boot/boot.s

ASM_SOURCES := $(ASM_SOURCES_$(ARCH))
ASM_OBJECTS := $(patsubst src/%.asm,target/%.o,$(patsubst src/%.s,target/%.o,$(ASM_SOURCES)))

# Assembly build rules
target/boot/multiboot_header.o: src/boot/multiboot_header.asm | target/boot
	@mkdir -p target/boot
	@nasm -f elf64 $< -o $@

target/%.o: src/%.asm | target
	@nasm -f elf64 $< -o $@

target/%.o: src/%.s | target
	@$(CC) -c $< -o $@

target:
	@mkdir -p target target/boot

# Linker scripts
LINKER_SCRIPT_x86_64 := linker.ld
LINKER_SCRIPT_x86 := linker.ld
LINKER_SCRIPT_arm := src/arch/arm/linker.ld
LINKER_SCRIPT_aarch64 := src/arch/aarch64/linker.ld

LINKER_SCRIPT := $(LINKER_SCRIPT_$(ARCH))

# QEMU configuration
QEMU_x86_64 := qemu-system-x86_64
QEMU_x86 := qemu-system-i386
QEMU_arm := qemu-system-arm
QEMU_aarch64 := qemu-system-aarch64

QEMU := $(QEMU_$(ARCH))

QEMU_FLAGS_x86_64 := -m 512M -cpu qemu64 -machine q35 -serial stdio -no-reboot -no-shutdown
QEMU_FLAGS_x86 := -m 256M -cpu pentium3 -machine pc -serial stdio -no-reboot -no-shutdown
QEMU_FLAGS_arm := -m 256M -cpu cortex-a15 -machine vexpress-a15 -serial stdio -no-reboot -no-shutdown
QEMU_FLAGS_aarch64 := -m 512M -cpu cortex-a57 -machine virt -serial stdio -no-reboot -no-shutdown

QEMU_FLAGS := $(QEMU_FLAGS_$(ARCH))

# Colors for output
RED := \033[0;31m
GREEN := \033[0;32m
YELLOW := \033[0;33m
BLUE := \033[0;34m
PURPLE := \033[0;35m
CYAN := \033[0;36m
WHITE := \033[0;37m
NC := \033[0m # No Color

# Default target
.PHONY: all
all: kernel

# Help target
.PHONY: help
help:
	@echo "$(CYAN)RustixOS Build System$(NC)"
	@echo "$(CYAN)=====================$(NC)"
	@echo ""
	@echo "$(YELLOW)Available targets:$(NC)"
	@echo "  $(GREEN)kernel$(NC)          - Build the kernel for current architecture ($(ARCH))"
	@echo "  $(GREEN)all-archs$(NC)       - Build kernel for all supported architectures"
	@echo "  $(GREEN)iso$(NC)             - Create bootable ISO image"
	@echo "  $(GREEN)run$(NC)             - Run kernel in QEMU"
	@echo "  $(GREEN)run-iso$(NC)         - Run ISO image in QEMU"
	@echo "  $(GREEN)debug$(NC)           - Run kernel in QEMU with GDB support"
	@echo "  $(GREEN)test$(NC)            - Run kernel tests"
	@echo "  $(GREEN)clean$(NC)           - Clean build artifacts"
	@echo "  $(GREEN)distclean$(NC)       - Clean everything including dependencies"
	@echo "  $(GREEN)format$(NC)          - Format source code"
	@echo "  $(GREEN)clippy$(NC)          - Run Clippy linter"
	@echo "  $(GREEN)doc$(NC)             - Generate documentation"
	@echo "  $(GREEN)install-deps$(NC)    - Install build dependencies"
	@echo "  $(GREEN)toolchain$(NC)       - Install cross-compilation toolchain"
	@echo ""
	@echo "$(YELLOW)Configuration:$(NC)"
	@echo "  $(BLUE)ARCH$(NC)            - Target architecture ($(SUPPORTED_ARCHS))"
	@echo "  $(BLUE)BUILD_MODE$(NC)      - Build mode (debug/release)"
	@echo ""
	@echo "$(YELLOW)Examples:$(NC)"
	@echo "  make kernel ARCH=x86_64 BUILD_MODE=release"
	@echo "  make run ARCH=arm"
	@echo "  make iso ARCH=x86_64"

# Validate architecture
.PHONY: validate-arch
validate-arch:
	@if ! echo "$(SUPPORTED_ARCHS)" | grep -wq "$(ARCH)"; then \
		echo "$(RED)Error: Unsupported architecture '$(ARCH)'$(NC)"; \
		echo "$(YELLOW)Supported architectures: $(SUPPORTED_ARCHS)$(NC)"; \
		exit 1; \
	fi

# Create build directories
$(BUILD_DIR):
	@mkdir -p $(BUILD_DIR)
	@mkdir -p $(BUILD_DIR)/src/arch/$(ARCH)

# Build Rust kernel
.PHONY: rust-kernel
rust-kernel: validate-arch target/boot/multiboot_header.o
	@echo "$(BLUE)Building Rust kernel for $(ARCH)...$(NC)"
	@RUSTFLAGS="$(RUSTFLAGS)" cargo build $(CARGO_FLAGS)

# Assemble architecture-specific files
$(BUILD_DIR)/%.o: src/%.s | $(BUILD_DIR)
	@echo "$(YELLOW)Assembling $<...$(NC)"
	@mkdir -p $(dir $@)
	@$(CC) -c $< -o $@

# Link kernel
$(KERNEL_ELF): rust-kernel $(ASM_OBJECTS) $(LINKER_SCRIPT) | $(BUILD_DIR)
	@echo "$(PURPLE)Linking kernel...$(NC)"
	@$(LD) -T $(LINKER_SCRIPT) -o $@ $(ASM_OBJECTS) $(KERNEL_BIN)

# Create kernel image
$(KERNEL_IMG): $(KERNEL_ELF)
	@echo "$(CYAN)Creating kernel image...$(NC)"
	@$(OBJCOPY) -O binary $< $@

# Main kernel target
.PHONY: kernel
kernel: target/x86_64-unknown-none/debug/kernel
	@echo "$(GREEN)Kernel built successfully for x86_64!$(NC)"

# Build for all architectures
.PHONY: all-archs
all-archs:
	@for arch in $(SUPPORTED_ARCHS); do \
		echo "$(BLUE)Building for $$arch...$(NC)"; \
		$(MAKE) kernel ARCH=$$arch || exit 1; \
	done
	@echo "$(GREEN)All architectures built successfully!$(NC)"

# Create GRUB configuration
$(GRUB_DIR)/grub.cfg: | $(GRUB_DIR)
	@echo "$(YELLOW)Creating GRUB configuration...$(NC)"
	@echo 'set timeout=0' > $@
	@echo 'set default=0' >> $@
	@echo '' >> $@
	@echo 'menuentry "RustixOS" {' >> $@
	@echo '    multiboot2 /boot/kernel.elf' >> $@
	@echo '    boot' >> $@
	@echo '}' >> $@

$(GRUB_DIR):
	@mkdir -p $(GRUB_DIR)

# Create ISO image
.PHONY: iso
iso: $(KERNEL_ELF) $(GRUB_DIR)/grub.cfg
	@echo "$(PURPLE)Creating ISO image...$(NC)"
	@cp $(KERNEL_ELF) $(ISO_DIR)/boot/kernel.elf
	@grub-mkrescue -o $(ISO_FILE) $(ISO_DIR) 2>/dev/null || \
		(echo "$(RED)Error: grub-mkrescue not found. Install GRUB tools.$(NC)" && exit 1)
	@echo "$(GREEN)ISO created: $(ISO_FILE)$(NC)"

# Run kernel in QEMU
.PHONY: run
run: $(KERNEL_ELF) validate-arch
	@echo "$(BLUE)Running kernel in QEMU ($(ARCH))...$(NC)"
	@$(QEMU) $(QEMU_FLAGS) -kernel $(KERNEL_ELF)

# Run ISO in QEMU
.PHONY: run-iso
run-iso: iso validate-arch
	@echo "$(BLUE)Running ISO in QEMU ($(ARCH))...$(NC)"
	@$(QEMU) $(QEMU_FLAGS) -cdrom $(ISO_FILE)

# Debug kernel with GDB
.PHONY: debug
debug: $(KERNEL_ELF) validate-arch
	@echo "$(PURPLE)Starting kernel debug session...$(NC)"
	@echo "$(YELLOW)Connect with: gdb $(KERNEL_ELF) -ex 'target remote :1234'$(NC)"
	@$(QEMU) $(QEMU_FLAGS) -kernel $(KERNEL_ELF) -s -S

# Run tests
.PHONY: test
test: validate-arch
	@echo "$(BLUE)Running tests for $(ARCH)...$(NC)"
	@RUSTFLAGS="$(RUSTFLAGS)" cargo test $(CARGO_FLAGS)

# Format code
.PHONY: format
format:
	@echo "$(YELLOW)Formatting code...$(NC)"
	@cargo fmt

# Run Clippy
.PHONY: clippy
clippy:
	@echo "$(YELLOW)Running Clippy...$(NC)"
	@cargo clippy $(CARGO_FLAGS) -- -D warnings

# Generate documentation
.PHONY: doc
doc:
	@echo "$(YELLOW)Generating documentation...$(NC)"
	@cargo doc $(CARGO_FLAGS) --open

# Clean build artifacts
.PHONY: clean
clean:
	@echo "$(RED)Cleaning build artifacts...$(NC)"
	@cargo clean
	@rm -rf $(BUILD_DIR)

# Clean everything
.PHONY: distclean
distclean: clean
	@echo "$(RED)Cleaning everything...$(NC)"
	@rm -rf target/

# Install build dependencies
.PHONY: install-deps
install-deps:
	@echo "$(BLUE)Installing build dependencies...$(NC)"
	@rustup component add rust-src
	@rustup component add llvm-tools-preview
	@cargo install bootimage
	@cargo install cargo-xbuild || true

# Install cross-compilation toolchain
.PHONY: toolchain
toolchain:
	@echo "$(BLUE)Installing cross-compilation toolchain...$(NC)"
	@echo "$(YELLOW)This will install toolchains for all supported architectures$(NC)"
	@# x86_64 toolchain
	@if ! command -v x86_64-elf-gcc >/dev/null 2>&1; then \
		echo "$(YELLOW)Installing x86_64 toolchain...$(NC)"; \
		echo "$(RED)Please install x86_64-elf-gcc manually$(NC)"; \
	fi
	@# ARM toolchain
	@if ! command -v arm-none-eabi-gcc >/dev/null 2>&1; then \
		echo "$(YELLOW)Installing ARM toolchain...$(NC)"; \
		echo "$(RED)Please install arm-none-eabi-gcc manually$(NC)"; \
	fi
	@# AArch64 toolchain
	@if ! command -v aarch64-none-elf-gcc >/dev/null 2>&1; then \
		echo "$(YELLOW)Installing AArch64 toolchain...$(NC)"; \
		echo "$(RED)Please install aarch64-none-elf-gcc manually$(NC)"; \
	fi

# Check system requirements
.PHONY: check-deps
check-deps:
	@echo "$(BLUE)Checking system dependencies...$(NC)"
	@command -v rustc >/dev/null 2>&1 || (echo "$(RED)Rust not found$(NC)" && exit 1)
	@command -v cargo >/dev/null 2>&1 || (echo "$(RED)Cargo not found$(NC)" && exit 1)
	@command -v qemu-system-x86_64 >/dev/null 2>&1 || echo "$(YELLOW)QEMU not found (optional)$(NC)"
	@command -v grub-mkrescue >/dev/null 2>&1 || echo "$(YELLOW)GRUB tools not found (optional)$(NC)"
	@echo "$(GREEN)Dependencies check complete$(NC)"

# Show build information
.PHONY: info
info:
	@echo "$(CYAN)Build Information$(NC)"
	@echo "$(CYAN)=================$(NC)"
	@echo "Project: $(PROJECT_NAME) v$(VERSION)"
	@echo "Architecture: $(ARCH)"
	@echo "Build Mode: $(BUILD_MODE)"
	@echo "Rust Target: $(RUST_TARGET)"
	@echo "Kernel Binary: $(KERNEL_BIN)"
	@echo "Kernel ELF: $(KERNEL_ELF)"
	@echo "ISO File: $(ISO_FILE)"
	@echo ""
	@echo "$(YELLOW)Toolchain:$(NC)"
	@echo "CC: $(CC)"
	@echo "LD: $(LD)"
	@echo "OBJCOPY: $(OBJCOPY)"
	@echo "QEMU: $(QEMU)"

# Create target specification files
.PHONY: create-targets
create-targets:
	@echo "$(YELLOW)Creating target specification files...$(NC)"
	@$(MAKE) -f scripts/create-targets.mk

# Benchmark
.PHONY: bench
bench:
	@echo "$(BLUE)Running benchmarks...$(NC)"
	@cargo bench $(CARGO_FLAGS)

# Create release
.PHONY: release
release:
	@echo "$(PURPLE)Creating release build...$(NC)"
	@$(MAKE) all-archs BUILD_MODE=release
	@$(MAKE) iso ARCH=x86_64 BUILD_MODE=release
	@echo "$(GREEN)Release build complete!$(NC)"

# Install kernel (for development)
.PHONY: install
install: kernel
	@echo "$(BLUE)Installing kernel...$(NC)"
	@mkdir -p /boot/rustix
	@cp $(KERNEL_ELF) /boot/rustix/kernel-$(ARCH).elf
	@echo "$(GREEN)Kernel installed to /boot/rustix/$(NC)"

# Uninstall kernel
.PHONY: uninstall
uninstall:
	@echo "$(RED)Uninstalling kernel...$(NC)"
	@rm -rf /boot/rustix
	@echo "$(GREEN)Kernel uninstalled$(NC)"

# Show kernel size
.PHONY: size
size: $(KERNEL_ELF)
	@echo "$(CYAN)Kernel Size Information$(NC)"
	@echo "$(CYAN)======================$(NC)"
	@size $(KERNEL_ELF)
	@echo ""
	@ls -lh $(KERNEL_ELF)

# Disassemble kernel
.PHONY: disasm
disasm: $(KERNEL_ELF)
	@echo "$(YELLOW)Disassembling kernel...$(NC)"
	@$(OBJDUMP) -d $(KERNEL_ELF) > $(BUILD_DIR)/kernel-$(ARCH).disasm
	@echo "$(GREEN)Disassembly saved to $(BUILD_DIR)/kernel-$(ARCH).disasm$(NC)"

# Memory map
.PHONY: memmap
memmap: $(KERNEL_ELF)
	@echo "$(CYAN)Kernel Memory Map$(NC)"
	@echo "$(CYAN)=================$(NC)"
	@$(OBJDUMP) -h $(KERNEL_ELF)

# Symbol table
.PHONY: symbols
symbols: $(KERNEL_ELF)
	@echo "$(YELLOW)Generating symbol table...$(NC)"
	@$(OBJDUMP) -t $(KERNEL_ELF) > $(BUILD_DIR)/kernel-$(ARCH).symbols
	@echo "$(GREEN)Symbol table saved to $(BUILD_DIR)/kernel-$(ARCH).symbols$(NC)"

# Create distribution package
.PHONY: dist
dist: release
	@echo "$(PURPLE)Creating distribution package...$(NC)"
	@mkdir -p $(BUILD_DIR)/dist
	@tar -czf $(BUILD_DIR)/dist/$(PROJECT_NAME)-$(VERSION).tar.gz \
		--exclude=target --exclude=build --exclude=.git \
		--transform 's,^,$(PROJECT_NAME)-$(VERSION)/,' .
	@echo "$(GREEN)Distribution package created: $(BUILD_DIR)/dist/$(PROJECT_NAME)-$(VERSION).tar.gz$(NC)"

# Continuous integration target
.PHONY: ci
ci: check-deps format clippy test all-archs
	@echo "$(GREEN)CI pipeline completed successfully!$(NC)"

# Show this help
.PHONY: list
list: help