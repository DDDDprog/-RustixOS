#!/usr/bin/env python3
"""
RustixOS Rust Build Script
Professional Rust compilation with advanced features
"""

import argparse
import os
import subprocess
import sys
import json
import shutil
from pathlib import Path
from typing import List, Dict, Optional

class RustBuilder:
    def __init__(self):
        self.rustc_path = self._find_rustc()
        self.cargo_path = self._find_cargo()
        
    def _find_rustc(self) -> str:
        """Find rustc executable"""
        rustc = shutil.which('rustc')
        if not rustc:
            raise RuntimeError("rustc not found in PATH")
        return rustc
    
    def _find_cargo(self) -> str:
        """Find cargo executable"""
        cargo = shutil.which('cargo')
        if not cargo:
            raise RuntimeError("cargo not found in PATH")
        return cargo
    
    def build_static_library(self, args) -> int:
        """Build Rust static library"""
        cmd = [
            self.rustc,
            '--crate-name', args.crate_name,
            '--crate-type', 'staticlib',
            '--edition', args.edition,
            args.crate_root,
            '-o', args.output,
        ]
        
        # Add target specification
        if args.target:
            target_spec = self._get_target_spec(args.target)
            if target_spec:
                cmd.extend(['--target', target_spec])
        
        # Add optimization flags
        if args.debug:
            cmd.extend(['-C', 'opt-level=0', '-C', 'debuginfo=2'])
        else:
            cmd.extend(['-C', 'opt-level=3', '-C', 'debuginfo=1'])
        
        # Add features
        for feature in args.features:
            cmd.extend(['--cfg', f'feature="{feature}"'])
        
        # Add custom cfg flags
        for cfg in args.cfg:
            cmd.extend(['--cfg', cfg])
        
        # Add rustc flags
        for flag in args.rustc_flags:
            cmd.append(flag)
        
        # Add environment variables
        env = os.environ.copy()
        for env_var in args.env:
            key, value = env_var.split('=', 1)
            env[key] = value
        
        # Kernel-specific flags
        cmd.extend([
            '-C', 'panic=abort',
            '-C', 'code-model=kernel',
            '--emit', 'obj,metadata',
            '-Z', 'build-std=core,alloc,compiler_builtins',
            '-Z', 'build-std-features=compiler-builtins-mem',
        ])
        
        print(f"Building Rust static library: {args.crate_name}")
        print(f"Command: {' '.join(cmd)}")
        
        try:
            result = subprocess.run(cmd, env=env, check=True, 
                                  capture_output=True, text=True)
            print("Build successful!")
            return 0
        except subprocess.CalledProcessError as e:
            print(f"Build failed with exit code {e.returncode}")
            print(f"stdout: {e.stdout}")
            print(f"stderr: {e.stderr}")
            return e.returncode
    
    def build_executable(self, args) -> int:
        """Build Rust executable"""
        cmd = [
            self.rustc,
            '--crate-name', args.crate_name,
            '--crate-type', 'bin',
            '--edition', args.edition,
            args.crate_root,
            '-o', args.output,
        ]
        
        # Add target specification
        if args.target:
            target_spec = self._get_target_spec(args.target)
            if target_spec:
                cmd.extend(['--target', target_spec])
        
        # Add optimization flags
        if args.debug:
            cmd.extend(['-C', 'opt-level=0', '-C', 'debuginfo=2'])
        else:
            cmd.extend(['-C', 'opt-level=3', '-C', 'debuginfo=1'])
        
        # Add features
        for feature in args.features:
            cmd.extend(['--cfg', f'feature="{feature}"'])
        
        # Kernel-specific flags
        cmd.extend([
            '-C', 'panic=abort',
            '-C', 'link-arg=-nostartfiles',
            '-C', 'link-arg=-static',
            '-Z', 'build-std=core,alloc,compiler_builtins',
        ])
        
        print(f"Building Rust executable: {args.crate_name}")
        print(f"Command: {' '.join(cmd)}")
        
        try:
            result = subprocess.run(cmd, check=True, capture_output=True, text=True)
            print("Build successful!")
            return 0
        except subprocess.CalledProcessError as e:
            print(f"Build failed with exit code {e.returncode}")
            print(f"stdout: {e.stdout}")
            print(f"stderr: {e.stderr}")
            return e.returncode
    
    def _get_target_spec(self, target: str) -> Optional[str]:
        """Get target specification file"""
        target_specs = {
            'x64': 'x86_64-rustix.json',
            'x86': 'i686-rustix.json',
            'arm': 'armv7-rustix.json',
            'arm64': 'aarch64-rustix.json',
        }
        return target_specs.get(target)
    
    def create_target_specs(self):
        """Create target specification files"""
        specs = {
            'x86_64-rustix.json': {
                "llvm-target": "x86_64-unknown-none",
                "data-layout": "e-m:e-i64:64-f80:128-n8:16:32:64-S128",
                "arch": "x86_64",
                "target-endian": "little",
                "target-pointer-width": "64",
                "target-c-int-width": "32",
                "os": "none",
                "executables": True,
                "linker-flavor": "ld.lld",
                "linker": "rust-lld",
                "panic-strategy": "abort",
                "disable-redzone": True,
                "features": "-mmx,-sse,+soft-float",
                "code-model": "kernel"
            },
            'i686-rustix.json': {
                "llvm-target": "i686-unknown-none",
                "data-layout": "e-m:e-p:32:32-f64:32:64-f80:32-n8:16:32-S128",
                "arch": "x86",
                "target-endian": "little",
                "target-pointer-width": "32",
                "target-c-int-width": "32",
                "os": "none",
                "executables": True,
                "linker-flavor": "ld.lld",
                "linker": "rust-lld",
                "panic-strategy": "abort",
                "features": "-mmx,-sse,+soft-float",
                "code-model": "kernel"
            },
            'armv7-rustix.json': {
                "llvm-target": "armv7-unknown-none-eabihf",
                "data-layout": "e-m:e-p:32:32-Fi8-i64:64-v128:64:128-a:0:32-n32-S64",
                "arch": "arm",
                "target-endian": "little",
                "target-pointer-width": "32",
                "target-c-int-width": "32",
                "os": "none",
                "executables": True,
                "linker-flavor": "ld.lld",
                "linker": "rust-lld",
                "panic-strategy": "abort",
                "features": "+v7,+thumb2,+neon,+vfp3",
                "max-atomic-width": 64
            },
            'aarch64-rustix.json': {
                "llvm-target": "aarch64-unknown-none",
                "data-layout": "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128",
                "arch": "aarch64",
                "target-endian": "little",
                "target-pointer-width": "64",
                "target-c-int-width": "32",
                "os": "none",
                "executables": True,
                "linker-flavor": "ld.lld",
                "linker": "rust-lld",
                "panic-strategy": "abort",
                "features": "+strict-align,+neon,+fp-armv8",
                "max-atomic-width": 128
            }
        }
        
        for filename, spec in specs.items():
            with open(filename, 'w') as f:
                json.dump(spec, f, indent=2)
            print(f"Created target specification: {filename}")

def main():
    parser = argparse.ArgumentParser(description='RustixOS Rust Build Script')
    parser.add_argument('--crate-name', required=True, help='Crate name')
    parser.add_argument('--crate-root', required=True, help='Crate root file')
    parser.add_argument('--output', required=True, help='Output file')
    parser.add_argument('--edition', default='2021', help='Rust edition')
    parser.add_argument('--target', help='Target architecture')
    parser.add_argument('--crate-type', default='staticlib', 
                       choices=['staticlib', 'bin'], help='Crate type')
    parser.add_argument('--debug', action='store_true', help='Debug build')
    parser.add_argument('--release', action='store_true', help='Release build')
    parser.add_argument('--feature', dest='features', action='append', 
                       default=[], help='Enable feature')
    parser.add_argument('--cfg', dest='cfg', action='append', 
                       default=[], help='Configuration flag')
    parser.add_argument('--rustc-flag', dest='rustc_flags', action='append',
                       default=[], help='Additional rustc flag')
    parser.add_argument('--env', dest='env', action='append',
                       default=[], help='Environment variable (KEY=VALUE)')
    parser.add_argument('--dep', dest='deps', action='append',
                       default=[], help='Dependency')
    parser.add_argument('--create-targets', action='store_true',
                       help='Create target specification files')
    
    args = parser.parse_args()
    
    builder = RustBuilder()
    
    if args.create_targets:
        builder.create_target_specs()
        return 0
    
    # Ensure output directory exists
    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    
    if args.crate_type == 'staticlib':
        return builder.build_static_library(args)
    elif args.crate_type == 'bin':
        return builder.build_executable(args)
    else:
        print(f"Unsupported crate type: {args.crate_type}")
        return 1

if __name__ == '__main__':
    sys.exit(main())