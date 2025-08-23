#!/usr/bin/env python3
"""
RustixOS Rust Test Build Script
Professional test compilation and execution
"""

import argparse
import os
import subprocess
import sys
import json
import shutil
import tempfile
from pathlib import Path
from typing import List, Dict, Optional

class RustTestBuilder:
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
    
    def build_test(self, args) -> int:
        """Build Rust test executable"""
        cmd = [
            self.rustc,
            '--crate-name', args.crate_name,
            '--crate-type', 'bin',
            '--edition', args.edition,
            '--test',
            args.crate_root,
            '-o', args.output,
        ]
        
        # Add target specification
        if args.target:
            target_spec = self._get_target_spec(args.target)
            if target_spec:
                cmd.extend(['--target', target_spec])
        
        # Add optimization flags (tests usually built in debug mode)
        cmd.extend(['-C', 'opt-level=0', '-C', 'debuginfo=2'])
        
        # Add features
        for feature in args.features:
            cmd.extend(['--cfg', f'feature="{feature}"'])
        
        # Add test-specific flags
        cmd.extend([
            '--cfg', 'test',
            '-L', 'dependency=target/debug/deps',
        ])
        
        # Add dependencies
        for dep in args.deps:
            cmd.extend(['--extern', dep])
        
        print(f"Building Rust test: {args.crate_name}")
        print(f"Command: {' '.join(cmd)}")
        
        try:
            result = subprocess.run(cmd, check=True, capture_output=True, text=True)
            print("Test build successful!")
            return 0
        except subprocess.CalledProcessError as e:
            print(f"Test build failed with exit code {e.returncode}")
            print(f"stdout: {e.stdout}")
            print(f"stderr: {e.stderr}")
            return e.returncode
    
    def run_test(self, test_executable: str, args) -> int:
        """Run the test executable"""
        if not os.path.exists(test_executable):
            print(f"Test executable not found: {test_executable}")
            return 1
        
        # Make executable
        os.chmod(test_executable, 0o755)
        
        cmd = [test_executable]
        
        # Add test arguments
        if args.test_threads:
            cmd.extend(['--test-threads', str(args.test_threads)])
        
        if args.nocapture:
            cmd.append('--nocapture')
        
        if args.test_filter:
            cmd.append(args.test_filter)
        
        print(f"Running tests: {' '.join(cmd)}")
        
        try:
            result = subprocess.run(cmd, check=True)
            print("All tests passed!")
            return 0
        except subprocess.CalledProcessError as e:
            print(f"Tests failed with exit code {e.returncode}")
            return e.returncode
    
    def run_kernel_tests(self, args) -> int:
        """Run kernel tests in QEMU"""
        if not args.target:
            print("Target architecture required for kernel tests")
            return 1
        
        qemu_cmd = self._get_qemu_command(args.target)
        if not qemu_cmd:
            print(f"QEMU not available for target: {args.target}")
            return 1
        
        # Build test kernel
        test_kernel = f"{args.output}_kernel"
        build_result = self._build_test_kernel(args, test_kernel)
        if build_result != 0:
            return build_result
        
        # Run in QEMU
        cmd = [
            qemu_cmd,
            '-kernel', test_kernel,
            '-serial', 'stdio',
            '-display', 'none',
            '-device', 'isa-debug-exit,iobase=0xf4,iosize=0x04',
            '-no-reboot',
            '-no-shutdown',
        ]
        
        # Add architecture-specific flags
        if args.target == 'x64':
            cmd.extend(['-m', '512M', '-cpu', 'qemu64', '-machine', 'q35'])
        elif args.target == 'x86':
            cmd.extend(['-m', '256M', '-cpu', 'pentium3', '-machine', 'pc'])
        elif args.target == 'arm':
            cmd.extend(['-m', '256M', '-cpu', 'cortex-a15', '-machine', 'vexpress-a15'])
        elif args.target == 'arm64':
            cmd.extend(['-m', '512M', '-cpu', 'cortex-a57', '-machine', 'virt'])
        
        print(f"Running kernel tests in QEMU: {' '.join(cmd)}")
        
        try:
            result = subprocess.run(cmd, timeout=300)  # 5 minute timeout
            if result.returncode == 33:  # Success exit code
                print("Kernel tests passed!")
                return 0
            else:
                print(f"Kernel tests failed with exit code {result.returncode}")
                return 1
        except subprocess.TimeoutExpired:
            print("Kernel tests timed out")
            return 1
        except subprocess.CalledProcessError as e:
            print(f"QEMU failed with exit code {e.returncode}")
            return e.returncode
    
    def _build_test_kernel(self, args, output: str) -> int:
        """Build test kernel for QEMU execution"""
        cmd = [
            self.rustc,
            '--crate-name', f"{args.crate_name}_test_kernel",
            '--crate-type', 'bin',
            '--edition', args.edition,
            '--test',
            args.crate_root,
            '-o', output,
        ]
        
        # Add target specification
        if args.target:
            target_spec = self._get_target_spec(args.target)
            if target_spec:
                cmd.extend(['--target', target_spec])
        
        # Add kernel-specific flags
        cmd.extend([
            '-C', 'panic=abort',
            '-C', 'code-model=kernel',
            '-Z', 'build-std=core,alloc,compiler_builtins',
            '-Z', 'build-std-features=compiler-builtins-mem',
            '--cfg', 'test',
            '--cfg', 'kernel_test',
        ])
        
        # Add features
        for feature in args.features:
            cmd.extend(['--cfg', f'feature="{feature}"'])
        
        try:
            result = subprocess.run(cmd, check=True, capture_output=True, text=True)
            return 0
        except subprocess.CalledProcessError as e:
            print(f"Test kernel build failed: {e.stderr}")
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
    
    def _get_qemu_command(self, target: str) -> Optional[str]:
        """Get QEMU command for target architecture"""
        qemu_commands = {
            'x64': 'qemu-system-x86_64',
            'x86': 'qemu-system-i386',
            'arm': 'qemu-system-arm',
            'arm64': 'qemu-system-aarch64',
        }
        
        qemu_cmd = qemu_commands.get(target)
        if qemu_cmd and shutil.which(qemu_cmd):
            return qemu_cmd
        return None
    
    def run_integration_tests(self, args) -> int:
        """Run integration tests"""
        test_dir = Path("tests/integration")
        if not test_dir.exists():
            print("No integration tests found")
            return 0
        
        failed_tests = 0
        total_tests = 0
        
        for test_file in test_dir.glob("*.rs"):
            total_tests += 1
            print(f"\nRunning integration test: {test_file.name}")
            
            # Build test
            test_output = f"target/test_{test_file.stem}"
            build_cmd = [
                self.rustc,
                '--crate-name', test_file.stem,
                '--crate-type', 'bin',
                '--edition', args.edition,
                '--test',
                str(test_file),
                '-o', test_output,
            ]
            
            try:
                subprocess.run(build_cmd, check=True, capture_output=True)
            except subprocess.CalledProcessError as e:
                print(f"Failed to build {test_file.name}: {e.stderr}")
                failed_tests += 1
                continue
            
            # Run test
            try:
                subprocess.run([test_output], check=True, timeout=60)
                print(f"✓ {test_file.name} passed")
            except (subprocess.CalledProcessError, subprocess.TimeoutExpired):
                print(f"✗ {test_file.name} failed")
                failed_tests += 1
        
        print(f"\nIntegration test results: {total_tests - failed_tests}/{total_tests} passed")
        return failed_tests

def main():
    parser = argparse.ArgumentParser(description='RustixOS Rust Test Build Script')
    parser.add_argument('--crate-name', required=True, help='Crate name')
    parser.add_argument('--crate-root', required=True, help='Crate root file')
    parser.add_argument('--output', required=True, help='Output file')
    parser.add_argument('--edition', default='2021', help='Rust edition')
    parser.add_argument('--target', help='Target architecture')
    parser.add_argument('--feature', dest='features', action='append', 
                       default=[], help='Enable feature')
    parser.add_argument('--dep', dest='deps', action='append',
                       default=[], help='Dependency')
    parser.add_argument('--run', action='store_true', help='Run tests after building')
    parser.add_argument('--kernel-test', action='store_true', help='Run kernel tests in QEMU')
    parser.add_argument('--integration', action='store_true', help='Run integration tests')
    parser.add_argument('--test-threads', type=int, help='Number of test threads')
    parser.add_argument('--nocapture', action='store_true', help='Don\'t capture test output')
    parser.add_argument('--test-filter', help='Filter tests by name')
    
    args = parser.parse_args()
    
    builder = RustTestBuilder()
    
    # Ensure output directory exists
    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    
    if args.integration:
        return builder.run_integration_tests(args)
    
    if args.kernel_test:
        return builder.run_kernel_tests(args)
    
    # Build test
    result = builder.build_test(args)
    if result != 0:
        return result
    
    # Run test if requested
    if args.run:
        return builder.run_test(args.output, args)
    
    return 0

if __name__ == '__main__':
    sys.exit(main())