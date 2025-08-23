#!/usr/bin/env python3
"""
RustixOS ISO Creation Script
Professional ISO image creation with GRUB support
"""

import argparse
import os
import subprocess
import sys
import shutil
import tempfile
from pathlib import Path
from typing import List, Dict, Optional

class IsoCreator:
    def __init__(self):
        self.grub_mkrescue = self._find_grub_mkrescue()
        self.required_tools = ['grub-mkrescue', 'xorriso']
        
    def _find_grub_mkrescue(self) -> Optional[str]:
        """Find grub-mkrescue executable"""
        candidates = ['grub-mkrescue', 'grub2-mkrescue']
        for cmd in candidates:
            path = shutil.which(cmd)
            if path:
                return path
        return None
    
    def check_dependencies(self) -> bool:
        """Check if required tools are available"""
        missing = []
        
        if not self.grub_mkrescue:
            missing.append('grub-mkrescue')
        
        if not shutil.which('xorriso'):
            missing.append('xorriso')
        
        if missing:
            print(f"Missing required tools: {', '.join(missing)}")
            print("Install with:")
            print("  Ubuntu/Debian: sudo apt install grub-pc-bin grub-efi-amd64-bin xorriso")
            print("  Fedora: sudo dnf install grub2-tools-extra xorriso")
            print("  Arch: sudo pacman -S grub xorriso")
            return False
        
        return True
    
    def create_iso(self, args) -> int:
        """Create bootable ISO image"""
        if not self.check_dependencies():
            return 1
        
        if not os.path.exists(args.kernel):
            print(f"Kernel not found: {args.kernel}")
            return 1
        
        if not os.path.exists(args.grub_cfg):
            print(f"GRUB config not found: {args.grub_cfg}")
            return 1
        
        # Create temporary directory structure
        with tempfile.TemporaryDirectory() as temp_dir:
            iso_root = Path(temp_dir) / "iso"
            boot_dir = iso_root / "boot"
            grub_dir = boot_dir / "grub"
            
            # Create directory structure
            grub_dir.mkdir(parents=True)
            
            # Copy kernel
            kernel_dest = boot_dir / "kernel.elf"
            shutil.copy2(args.kernel, kernel_dest)
            print(f"Copied kernel: {args.kernel} -> {kernel_dest}")
            
            # Copy GRUB configuration
            grub_cfg_dest = grub_dir / "grub.cfg"
            shutil.copy2(args.grub_cfg, grub_cfg_dest)
            print(f"Copied GRUB config: {args.grub_cfg} -> {grub_cfg_dest}")
            
            # Copy additional files
            if args.initrd:
                if os.path.exists(args.initrd):
                    initrd_dest = boot_dir / "initrd.img"
                    shutil.copy2(args.initrd, initrd_dest)
                    print(f"Copied initrd: {args.initrd} -> {initrd_dest}")
                else:
                    print(f"Warning: initrd not found: {args.initrd}")
            
            # Copy modules directory if it exists
            modules_src = Path("modules")
            if modules_src.exists():
                modules_dest = boot_dir / "modules"
                shutil.copytree(modules_src, modules_dest)
                print(f"Copied modules: {modules_src} -> {modules_dest}")
            
            # Create ISO
            return self._create_grub_iso(iso_root, args.output, args)
    
    def _create_grub_iso(self, iso_root: Path, output: str, args) -> int:
        """Create ISO using GRUB"""
        cmd = [
            self.grub_mkrescue,
            '-o', output,
            str(iso_root)
        ]
        
        # Add GRUB modules
        grub_modules = [
            'biosdisk', 'part_msdos', 'part_gpt', 'fat', 'ext2',
            'normal', 'multiboot2', 'boot', 'linux', 'chain',
            'configfile', 'gzio', 'gfxterm', 'gfxmenu', 'all_video',
            'font', 'echo', 'help', 'ls', 'cat', 'test', 'true',
            'sleep', 'loopback', 'videotest'
        ]
        
        if args.efi:
            grub_modules.extend(['efi_gop', 'efi_uga'])
        
        # Add architecture-specific modules
        if args.arch == 'x86_64':
            grub_modules.extend(['cpuid', 'msr'])
        elif args.arch == 'x86':
            grub_modules.extend(['cpuid'])
        
        # Build module list
        modules_str = ' '.join(grub_modules)
        cmd.extend(['--modules', modules_str])
        
        # Add platform support
        if args.efi:
            cmd.extend(['--format', 'x86_64-efi'])
        else:
            cmd.extend(['--format', 'i386-pc'])
        
        # Add verbose output if requested
        if args.verbose:
            cmd.append('--verbose')
        
        print(f"Creating ISO: {' '.join(cmd)}")
        
        try:
            result = subprocess.run(cmd, check=True, capture_output=True, text=True)
            print(f"ISO created successfully: {output}")
            
            # Show ISO information
            self._show_iso_info(output)
            
            return 0
        except subprocess.CalledProcessError as e:
            print(f"Failed to create ISO: {e.returncode}")
            print(f"stdout: {e.stdout}")
            print(f"stderr: {e.stderr}")
            return e.returncode
    
    def _show_iso_info(self, iso_path: str):
        """Show information about created ISO"""
        if not os.path.exists(iso_path):
            return
        
        stat = os.stat(iso_path)
        size_mb = stat.st_size / (1024 * 1024)
        
        print(f"\nISO Information:")
        print(f"  File: {iso_path}")
        print(f"  Size: {size_mb:.2f} MB ({stat.st_size} bytes)")
        
        # Try to show ISO contents
        if shutil.which('isoinfo'):
            try:
                result = subprocess.run(
                    ['isoinfo', '-l', '-i', iso_path],
                    capture_output=True, text=True, timeout=10
                )
                if result.returncode == 0:
                    print(f"  Contents preview:")
                    lines = result.stdout.split('\n')[:20]  # First 20 lines
                    for line in lines:
                        if line.strip():
                            print(f"    {line}")
                    if len(result.stdout.split('\n')) > 20:
                        print("    ...")
            except (subprocess.TimeoutExpired, subprocess.CalledProcessError):
                pass
    
    def create_grub_config(self, args) -> int:
        """Create GRUB configuration file"""
        config_content = self._generate_grub_config(args)
        
        try:
            with open(args.output, 'w') as f:
                f.write(config_content)
            print(f"GRUB configuration created: {args.output}")
            return 0
        except Exception as e:
            print(f"Failed to create GRUB config: {e}")
            return 1
    
    def _generate_grub_config(self, args) -> str:
        """Generate GRUB configuration content"""
        config = []
        
        # Basic settings
        config.append(f"set timeout={args.timeout or 5}")
        config.append(f"set default={args.default or 0}")
        config.append("")
        
        # Graphics settings
        if args.graphics:
            config.extend([
                "if loadfont /boot/grub/fonts/unicode.pf2 ; then",
                "  set gfxmode=auto",
                "  insmod efi_gop",
                "  insmod efi_uga", 
                "  insmod ieee1275_fb",
                "  insmod vbe",
                "  insmod vga",
                "  insmod video_bochs",
                "  insmod video_cirrus",
                "  insmod gfxterm",
                "  terminal_output gfxterm",
                "fi",
                ""
            ])
        
        # Menu entries
        entries = [
            {
                'title': 'RustixOS',
                'kernel': '/boot/kernel.elf',
                'args': '',
            },
            {
                'title': 'RustixOS (Debug)',
                'kernel': '/boot/kernel.elf',
                'args': 'debug=1 loglevel=7',
            },
            {
                'title': 'RustixOS (Safe Mode)',
                'kernel': '/boot/kernel.elf',
                'args': 'safe_mode=1 nosmp=1',
            }
        ]
        
        if args.custom_entries:
            # Parse custom entries from JSON or simple format
            try:
                import json
                with open(args.custom_entries, 'r') as f:
                    custom = json.load(f)
                    entries.extend(custom)
            except:
                print(f"Warning: Could not parse custom entries: {args.custom_entries}")
        
        for i, entry in enumerate(entries):
            config.append(f"menuentry \"{entry['title']}\" {{")
            
            if args.arch in ['x86_64', 'x86']:
                config.append(f"    multiboot2 {entry['kernel']} {entry['args']}")
                if args.initrd:
                    config.append(f"    module2 /boot/initrd.img")
            else:
                # For ARM architectures, use different boot method
                config.append(f"    linux {entry['kernel']} {entry['args']}")
                if args.initrd:
                    config.append(f"    initrd /boot/initrd.img")
            
            config.append("    boot")
            config.append("}")
            
            if i < len(entries) - 1:
                config.append("")
        
        # Advanced menu
        config.extend([
            "",
            "submenu \"Advanced Options\" {",
            "    menuentry \"Memory Test\" {",
            "        linux16 /boot/memtest86+.bin",
            "    }",
            "    menuentry \"System Information\" {",
            "        multiboot2 /boot/kernel.elf sysinfo=1",
            "    }",
            "    menuentry \"Recovery Mode\" {",
            "        multiboot2 /boot/kernel.elf recovery=1",
            "    }",
            "}"
        ])
        
        return '\n'.join(config)
    
    def verify_iso(self, iso_path: str) -> int:
        """Verify ISO image integrity"""
        if not os.path.exists(iso_path):
            print(f"ISO not found: {iso_path}")
            return 1
        
        print(f"Verifying ISO: {iso_path}")
        
        # Check if it's a valid ISO
        try:
            with open(iso_path, 'rb') as f:
                # Check ISO 9660 signature
                f.seek(32768)  # Skip to volume descriptor
                signature = f.read(5)
                if signature != b'CD001':
                    print("Invalid ISO 9660 signature")
                    return 1
        except Exception as e:
            print(f"Error reading ISO: {e}")
            return 1
        
        # Use isoinfo if available
        if shutil.which('isoinfo'):
            try:
                result = subprocess.run(
                    ['isoinfo', '-d', '-i', iso_path],
                    capture_output=True, text=True, check=True
                )
                print("ISO verification successful")
                print("Volume information:")
                for line in result.stdout.split('\n')[:10]:
                    if line.strip():
                        print(f"  {line}")
                return 0
            except subprocess.CalledProcessError as e:
                print(f"ISO verification failed: {e}")
                return 1
        
        print("ISO appears to be valid (basic check)")
        return 0

def main():
    parser = argparse.ArgumentParser(description='RustixOS ISO Creation Script')
    subparsers = parser.add_subparsers(dest='command', help='Commands')
    
    # Create ISO command
    create_parser = subparsers.add_parser('create', help='Create ISO image')
    create_parser.add_argument('--kernel', required=True, help='Kernel file')
    create_parser.add_argument('--grub-cfg', required=True, help='GRUB configuration file')
    create_parser.add_argument('--output', required=True, help='Output ISO file')
    create_parser.add_argument('--arch', default='x86_64', 
                              choices=['x86_64', 'x86', 'arm', 'arm64'],
                              help='Target architecture')
    create_parser.add_argument('--initrd', help='Initial ramdisk file')
    create_parser.add_argument('--efi', action='store_true', help='Create EFI-bootable ISO')
    create_parser.add_argument('--verbose', action='store_true', help='Verbose output')
    
    # Create GRUB config command
    config_parser = subparsers.add_parser('create-config', help='Create GRUB configuration')
    config_parser.add_argument('--output', required=True, help='Output configuration file')
    config_parser.add_argument('--arch', default='x86_64',
                              choices=['x86_64', 'x86', 'arm', 'arm64'],
                              help='Target architecture')
    config_parser.add_argument('--timeout', type=int, help='Boot timeout in seconds')
    config_parser.add_argument('--default', type=int, help='Default menu entry')
    config_parser.add_argument('--graphics', action='store_true', help='Enable graphics mode')
    config_parser.add_argument('--initrd', help='Initial ramdisk file')
    config_parser.add_argument('--custom-entries', help='Custom menu entries JSON file')
    
    # Verify ISO command
    verify_parser = subparsers.add_parser('verify', help='Verify ISO image')
    verify_parser.add_argument('iso', help='ISO file to verify')
    
    # Handle legacy direct execution
    if len(sys.argv) > 1 and sys.argv[1] not in ['create', 'create-config', 'verify']:
        # Legacy mode
        parser.add_argument('--kernel', required=True, help='Kernel file')
        parser.add_argument('--grub-cfg', required=True, help='GRUB configuration file')
        parser.add_argument('--output', required=True, help='Output ISO file')
        parser.add_argument('--arch', default='x86_64', help='Target architecture')
        parser.add_argument('--initrd', help='Initial ramdisk file')
        parser.add_argument('--efi', action='store_true', help='Create EFI-bootable ISO')
        parser.add_argument('--verbose', action='store_true', help='Verbose output')
        args = parser.parse_args()
        args.command = 'create'
    else:
        args = parser.parse_args()
    
    if not args.command:
        parser.print_help()
        return 1
    
    creator = IsoCreator()
    
    if args.command == 'create':
        return creator.create_iso(args)
    elif args.command == 'create-config':
        return creator.create_grub_config(args)
    elif args.command == 'verify':
        return creator.verify_iso(args.iso)
    else:
        print(f"Unknown command: {args.command}")
        return 1

if __name__ == '__main__':
    sys.exit(main())