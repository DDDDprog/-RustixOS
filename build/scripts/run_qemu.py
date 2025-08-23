#!/usr/bin/env python3
"""
RustixOS QEMU Runner Script
Professional QEMU automation with advanced features
"""

import argparse
import os
import subprocess
import sys
import signal
import time
import json
import shutil
from pathlib import Path
from typing import List, Dict, Optional, Tuple

class QemuRunner:
    def __init__(self):
        self.qemu_commands = {
            'x64': 'qemu-system-x86_64',
            'x86': 'qemu-system-i386', 
            'arm': 'qemu-system-arm',
            'arm64': 'qemu-system-aarch64',
        }
        
        self.default_configs = {
            'x64': {
                'memory': '512M',
                'cpu': 'qemu64',
                'machine': 'q35',
                'features': ['+smep', '+smap', '+rdrand'],
                'devices': ['isa-debug-exit,iobase=0xf4,iosize=0x04'],
            },
            'x86': {
                'memory': '256M',
                'cpu': 'pentium3',
                'machine': 'pc',
                'features': [],
                'devices': ['isa-debug-exit,iobase=0xf4,iosize=0x04'],
            },
            'arm': {
                'memory': '256M',
                'cpu': 'cortex-a15',
                'machine': 'vexpress-a15',
                'features': [],
                'devices': [],
            },
            'arm64': {
                'memory': '512M',
                'cpu': 'cortex-a57',
                'machine': 'virt',
                'features': ['+sve'],
                'devices': [],
            },
        }
    
    def run_kernel(self, args) -> int:
        """Run kernel in QEMU"""
        if not os.path.exists(args.kernel):
            print(f"Kernel not found: {args.kernel}")
            return 1
        
        qemu_cmd = self._get_qemu_command(args.arch)
        if not qemu_cmd:
            print(f"QEMU not available for architecture: {args.arch}")
            return 1
        
        cmd = self._build_qemu_command(qemu_cmd, args)
        
        print(f"Starting QEMU: {' '.join(cmd)}")
        print(f"Architecture: {args.arch}")
        print(f"Kernel: {args.kernel}")
        
        if args.log:
            return self._run_with_logging(cmd, args.log)
        else:
            return self._run_interactive(cmd)
    
    def run_iso(self, args) -> int:
        """Run ISO image in QEMU"""
        if not os.path.exists(args.iso):
            print(f"ISO not found: {args.iso}")
            return 1
        
        qemu_cmd = self._get_qemu_command(args.arch)
        if not qemu_cmd:
            print(f"QEMU not available for architecture: {args.arch}")
            return 1
        
        cmd = self._build_iso_command(qemu_cmd, args)
        
        print(f"Starting QEMU with ISO: {' '.join(cmd)}")
        print(f"Architecture: {args.arch}")
        print(f"ISO: {args.iso}")
        
        if args.log:
            return self._run_with_logging(cmd, args.log)
        else:
            return self._run_interactive(cmd)
    
    def debug_kernel(self, args) -> int:
        """Run kernel in QEMU with GDB support"""
        if not os.path.exists(args.kernel):
            print(f"Kernel not found: {args.kernel}")
            return 1
        
        qemu_cmd = self._get_qemu_command(args.arch)
        if not qemu_cmd:
            print(f"QEMU not available for architecture: {args.arch}")
            return 1
        
        cmd = self._build_qemu_command(qemu_cmd, args)
        
        # Add GDB support
        cmd.extend(['-s', '-S'])  # -s = gdbserver on port 1234, -S = freeze at startup
        
        print(f"Starting QEMU in debug mode: {' '.join(cmd)}")
        print("Connect with GDB using: gdb <kernel> -ex 'target remote :1234'")
        
        if args.log:
            return self._run_with_logging(cmd, args.log)
        else:
            return self._run_interactive(cmd)
    
    def run_tests(self, args) -> int:
        """Run automated tests in QEMU"""
        if not os.path.exists(args.kernel):
            print(f"Test kernel not found: {args.kernel}")
            return 1
        
        qemu_cmd = self._get_qemu_command(args.arch)
        if not qemu_cmd:
            print(f"QEMU not available for architecture: {args.arch}")
            return 1
        
        cmd = self._build_qemu_command(qemu_cmd, args)
        
        # Add test-specific options
        cmd.extend(['-display', 'none'])  # No GUI for tests
        
        print(f"Running tests in QEMU: {' '.join(cmd)}")
        
        try:
            result = subprocess.run(cmd, timeout=args.timeout or 300)
            
            # Check exit code
            if result.returncode == 33:  # Test success exit code
                print("Tests passed!")
                return 0
            elif result.returncode == 35:  # Test failure exit code
                print("Tests failed!")
                return 1
            else:
                print(f"Unexpected exit code: {result.returncode}")
                return result.returncode
                
        except subprocess.TimeoutExpired:
            print("Tests timed out")
            return 1
        except KeyboardInterrupt:
            print("Tests interrupted")
            return 1
    
    def _build_qemu_command(self, qemu_cmd: str, args) -> List[str]:
        """Build QEMU command line"""
        config = self.default_configs.get(args.arch, {})
        
        cmd = [qemu_cmd]
        
        # Basic options
        cmd.extend(['-kernel', args.kernel])
        cmd.extend(['-m', args.memory or config.get('memory', '512M')])
        cmd.extend(['-serial', 'stdio'])
        cmd.extend(['-no-reboot', '-no-shutdown'])
        
        # CPU and machine
        if args.cpu or config.get('cpu'):
            cpu = args.cpu or config['cpu']
            features = config.get('features', [])
            if features:
                cpu += ',' + ','.join(features)
            cmd.extend(['-cpu', cpu])
        
        if args.machine or config.get('machine'):
            cmd.extend(['-machine', args.machine or config['machine']])
        
        # SMP support
        if args.smp:
            cmd.extend(['-smp', str(args.smp)])
        
        # Devices
        for device in config.get('devices', []):
            cmd.extend(['-device', device])
        
        # Additional devices
        if args.devices:
            for device in args.devices:
                cmd.extend(['-device', device])
        
        # Network
        if args.network:
            cmd.extend(['-netdev', 'user,id=net0'])
            cmd.extend(['-device', 'e1000,netdev=net0'])
        
        # Graphics
        if args.no_graphic:
            cmd.extend(['-display', 'none'])
        elif args.vnc:
            cmd.extend(['-display', 'vnc=:1'])
        
        # Monitor
        if args.monitor:
            cmd.extend(['-monitor', args.monitor])
        
        # Additional arguments
        if args.qemu_args:
            cmd.extend(args.qemu_args.split())
        
        return cmd
    
    def _build_iso_command(self, qemu_cmd: str, args) -> List[str]:
        """Build QEMU command for ISO boot"""
        config = self.default_configs.get(args.arch, {})
        
        cmd = [qemu_cmd]
        
        # Basic options
        cmd.extend(['-cdrom', args.iso])
        cmd.extend(['-m', args.memory or config.get('memory', '512M')])
        cmd.extend(['-serial', 'stdio'])
        cmd.extend(['-no-reboot', '-no-shutdown'])
        
        # CPU and machine
        if args.cpu or config.get('cpu'):
            cpu = args.cpu or config['cpu']
            features = config.get('features', [])
            if features:
                cpu += ',' + ','.join(features)
            cmd.extend(['-cpu', cpu])
        
        if args.machine or config.get('machine'):
            cmd.extend(['-machine', args.machine or config['machine']])
        
        # Boot from CD
        cmd.extend(['-boot', 'd'])
        
        return cmd
    
    def _get_qemu_command(self, arch: str) -> Optional[str]:
        """Get QEMU command for architecture"""
        qemu_cmd = self.qemu_commands.get(arch)
        if qemu_cmd and shutil.which(qemu_cmd):
            return qemu_cmd
        return None
    
    def _run_interactive(self, cmd: List[str]) -> int:
        """Run QEMU interactively"""
        try:
            return subprocess.run(cmd).returncode
        except KeyboardInterrupt:
            print("\nQEMU interrupted")
            return 1
    
    def _run_with_logging(self, cmd: List[str], log_file: str) -> int:
        """Run QEMU with output logging"""
        try:
            with open(log_file, 'w') as f:
                process = subprocess.Popen(
                    cmd,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.STDOUT,
                    universal_newlines=True,
                    bufsize=1
                )
                
                # Real-time output and logging
                for line in process.stdout:
                    print(line, end='')
                    f.write(line)
                    f.flush()
                
                return process.wait()
                
        except KeyboardInterrupt:
            print("\nQEMU interrupted")
            if 'process' in locals():
                process.terminate()
            return 1
    
    def create_disk_image(self, args) -> int:
        """Create disk image for testing"""
        if not shutil.which('qemu-img'):
            print("qemu-img not found")
            return 1
        
        cmd = [
            'qemu-img', 'create',
            '-f', args.format or 'qcow2',
            args.output,
            args.size or '1G'
        ]
        
        print(f"Creating disk image: {' '.join(cmd)}")
        
        try:
            return subprocess.run(cmd).returncode
        except Exception as e:
            print(f"Failed to create disk image: {e}")
            return 1
    
    def list_machines(self, arch: str) -> int:
        """List available machines for architecture"""
        qemu_cmd = self._get_qemu_command(arch)
        if not qemu_cmd:
            print(f"QEMU not available for architecture: {arch}")
            return 1
        
        cmd = [qemu_cmd, '-machine', 'help']
        
        try:
            return subprocess.run(cmd).returncode
        except Exception as e:
            print(f"Failed to list machines: {e}")
            return 1
    
    def list_cpus(self, arch: str) -> int:
        """List available CPUs for architecture"""
        qemu_cmd = self._get_qemu_command(arch)
        if not qemu_cmd:
            print(f"QEMU not available for architecture: {arch}")
            return 1
        
        cmd = [qemu_cmd, '-cpu', 'help']
        
        try:
            return subprocess.run(cmd).returncode
        except Exception as e:
            print(f"Failed to list CPUs: {e}")
            return 1

def main():
    parser = argparse.ArgumentParser(description='RustixOS QEMU Runner')
    subparsers = parser.add_subparsers(dest='command', help='Commands')
    
    # Run kernel command
    run_parser = subparsers.add_parser('run', help='Run kernel in QEMU')
    run_parser.add_argument('--kernel', required=True, help='Kernel file')
    run_parser.add_argument('--arch', required=True, choices=['x64', 'x86', 'arm', 'arm64'],
                           help='Target architecture')
    run_parser.add_argument('--memory', help='Memory size (e.g., 512M)')
    run_parser.add_argument('--cpu', help='CPU model')
    run_parser.add_argument('--machine', help='Machine type')
    run_parser.add_argument('--smp', type=int, help='Number of CPUs')
    run_parser.add_argument('--devices', action='append', help='Additional devices')
    run_parser.add_argument('--network', action='store_true', help='Enable network')
    run_parser.add_argument('--no-graphic', action='store_true', help='Disable graphics')
    run_parser.add_argument('--vnc', action='store_true', help='Use VNC display')
    run_parser.add_argument('--monitor', help='Monitor interface')
    run_parser.add_argument('--qemu-args', help='Additional QEMU arguments')
    run_parser.add_argument('--log', help='Log output to file')
    
    # Run ISO command
    iso_parser = subparsers.add_parser('run-iso', help='Run ISO in QEMU')
    iso_parser.add_argument('--iso', required=True, help='ISO file')
    iso_parser.add_argument('--arch', required=True, choices=['x64', 'x86', 'arm', 'arm64'],
                           help='Target architecture')
    iso_parser.add_argument('--memory', help='Memory size')
    iso_parser.add_argument('--cpu', help='CPU model')
    iso_parser.add_argument('--machine', help='Machine type')
    iso_parser.add_argument('--log', help='Log output to file')
    
    # Debug command
    debug_parser = subparsers.add_parser('debug', help='Debug kernel with GDB')
    debug_parser.add_argument('--kernel', required=True, help='Kernel file')
    debug_parser.add_argument('--arch', required=True, choices=['x64', 'x86', 'arm', 'arm64'],
                             help='Target architecture')
    debug_parser.add_argument('--memory', help='Memory size')
    debug_parser.add_argument('--cpu', help='CPU model')
    debug_parser.add_argument('--machine', help='Machine type')
    debug_parser.add_argument('--log', help='Log output to file')
    
    # Test command
    test_parser = subparsers.add_parser('test', help='Run tests in QEMU')
    test_parser.add_argument('--kernel', required=True, help='Test kernel file')
    test_parser.add_argument('--arch', required=True, choices=['x64', 'x86', 'arm', 'arm64'],
                            help='Target architecture')
    test_parser.add_argument('--timeout', type=int, help='Test timeout in seconds')
    test_parser.add_argument('--memory', help='Memory size')
    
    # Create disk command
    disk_parser = subparsers.add_parser('create-disk', help='Create disk image')
    disk_parser.add_argument('--output', required=True, help='Output file')
    disk_parser.add_argument('--size', help='Disk size (e.g., 1G)')
    disk_parser.add_argument('--format', help='Image format (qcow2, raw)')
    
    # List machines command
    machines_parser = subparsers.add_parser('list-machines', help='List available machines')
    machines_parser.add_argument('--arch', required=True, choices=['x64', 'x86', 'arm', 'arm64'],
                                 help='Target architecture')
    
    # List CPUs command
    cpus_parser = subparsers.add_parser('list-cpus', help='List available CPUs')
    cpus_parser.add_argument('--arch', required=True, choices=['x64', 'x86', 'arm', 'arm64'],
                            help='Target architecture')
    
    # Handle direct script execution (legacy support)
    if len(sys.argv) > 1 and not sys.argv[1] in ['run', 'run-iso', 'debug', 'test', 'create-disk', 'list-machines', 'list-cpus']:
        # Legacy mode - assume run command
        parser.add_argument('--kernel', help='Kernel file')
        parser.add_argument('--iso', help='ISO file')
        parser.add_argument('--arch', required=True, choices=['x64', 'x86', 'arm', 'arm64'],
                           help='Target architecture')
        parser.add_argument('--log', help='Log output to file')
        args = parser.parse_args()
        args.command = 'run' if args.kernel else 'run-iso'
    else:
        args = parser.parse_args()
    
    if not args.command:
        parser.print_help()
        return 1
    
    runner = QemuRunner()
    
    if args.command == 'run':
        return runner.run_kernel(args)
    elif args.command == 'run-iso':
        return runner.run_iso(args)
    elif args.command == 'debug':
        return runner.debug_kernel(args)
    elif args.command == 'test':
        return runner.run_tests(args)
    elif args.command == 'create-disk':
        return runner.create_disk_image(args)
    elif args.command == 'list-machines':
        return runner.list_machines(args.arch)
    elif args.command == 'list-cpus':
        return runner.list_cpus(args.arch)
    else:
        print(f"Unknown command: {args.command}")
        return 1

if __name__ == '__main__':
    sys.exit(main())