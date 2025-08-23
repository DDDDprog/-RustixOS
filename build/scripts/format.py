#!/usr/bin/env python3
"""
RustixOS Code Formatting Script
Professional code formatting with multiple language support
"""

import argparse
import os
import subprocess
import sys
import shutil
from pathlib import Path
from typing import List, Dict, Optional, Tuple

class CodeFormatter:
    def __init__(self):
        self.formatters = {
            'rust': self._format_rust,
            'c': self._format_c,
            'cpp': self._format_cpp,
            'python': self._format_python,
            'shell': self._format_shell,
            'markdown': self._format_markdown,
            'json': self._format_json,
            'yaml': self._format_yaml,
        }
        
        self.file_extensions = {
            '.rs': 'rust',
            '.c': 'c',
            '.h': 'c',
            '.cpp': 'cpp',
            '.cxx': 'cpp',
            '.cc': 'cpp',
            '.hpp': 'cpp',
            '.hxx': 'cpp',
            '.py': 'python',
            '.sh': 'shell',
            '.bash': 'shell',
            '.md': 'markdown',
            '.json': 'json',
            '.yaml': 'yaml',
            '.yml': 'yaml',
        }
    
    def format_project(self, args) -> int:
        """Format entire project"""
        source_dir = Path(args.source_dir)
        if not source_dir.exists():
            print(f"Source directory not found: {source_dir}")
            return 1
        
        # Find all source files
        files_to_format = []
        for ext, lang in self.file_extensions.items():
            if not args.languages or lang in args.languages:
                pattern = f"**/*{ext}"
                files = list(source_dir.glob(pattern))
                files_to_format.extend([(f, lang) for f in files])
        
        # Filter out excluded paths
        if args.exclude:
            exclude_patterns = args.exclude
            filtered_files = []
            for file_path, lang in files_to_format:
                excluded = False
                for pattern in exclude_patterns:
                    if pattern in str(file_path):
                        excluded = True
                        break
                if not excluded:
                    filtered_files.append((file_path, lang))
            files_to_format = filtered_files
        
        if not files_to_format:
            print("No files to format")
            return 0
        
        print(f"Found {len(files_to_format)} files to format")
        
        # Format files
        failed_files = []
        formatted_files = []
        
        for file_path, lang in files_to_format:
            if args.verbose:
                print(f"Formatting {file_path} ({lang})")
            
            try:
                result = self.formatters[lang](file_path, args)
                if result == 0:
                    formatted_files.append(file_path)
                else:
                    failed_files.append((file_path, lang))
            except Exception as e:
                print(f"Error formatting {file_path}: {e}")
                failed_files.append((file_path, lang))
        
        # Report results
        print(f"\nFormatting complete:")
        print(f"  Successfully formatted: {len(formatted_files)} files")
        if failed_files:
            print(f"  Failed to format: {len(failed_files)} files")
            for file_path, lang in failed_files:
                print(f"    {file_path} ({lang})")
        
        # Create stamp file
        if args.stamp:
            with open(args.stamp, 'w') as f:
                f.write(f"Formatted {len(formatted_files)} files\n")
        
        return len(failed_files)
    
    def _format_rust(self, file_path: Path, args) -> int:
        """Format Rust code using rustfmt"""
        rustfmt = shutil.which('rustfmt')
        if not rustfmt:
            print("rustfmt not found")
            return 1
        
        cmd = [rustfmt]
        
        # Add configuration
        if args.rust_config:
            cmd.extend(['--config-path', args.rust_config])
        
        # Add edition
        if args.rust_edition:
            cmd.extend(['--edition', args.rust_edition])
        
        # Check mode or format in place
        if args.check:
            cmd.append('--check')
        
        cmd.append(str(file_path))
        
        try:
            result = subprocess.run(cmd, capture_output=True, text=True)
            if result.returncode != 0 and args.verbose:
                print(f"rustfmt output: {result.stderr}")
            return result.returncode
        except Exception as e:
            print(f"Error running rustfmt: {e}")
            return 1
    
    def _format_c(self, file_path: Path, args) -> int:
        """Format C code using clang-format"""
        clang_format = shutil.which('clang-format')
        if not clang_format:
            print("clang-format not found")
            return 1
        
        cmd = [clang_format]
        
        # Add style
        style = args.c_style or 'Google'
        cmd.extend(['-style', style])
        
        # Format in place or check
        if args.check:
            cmd.append('--dry-run')
            cmd.append('--Werror')
        else:
            cmd.append('-i')
        
        cmd.append(str(file_path))
        
        try:
            result = subprocess.run(cmd, capture_output=True, text=True)
            return result.returncode
        except Exception as e:
            print(f"Error running clang-format: {e}")
            return 1
    
    def _format_cpp(self, file_path: Path, args) -> int:
        """Format C++ code using clang-format"""
        return self._format_c(file_path, args)
    
    def _format_python(self, file_path: Path, args) -> int:
        """Format Python code using black"""
        black = shutil.which('black')
        if not black:
            # Try autopep8 as fallback
            autopep8 = shutil.which('autopep8')
            if autopep8:
                return self._format_python_autopep8(file_path, args)
            print("black or autopep8 not found")
            return 1
        
        cmd = [black]
        
        # Add line length
        if args.python_line_length:
            cmd.extend(['--line-length', str(args.python_line_length)])
        
        # Check mode
        if args.check:
            cmd.append('--check')
            cmd.append('--diff')
        
        cmd.append(str(file_path))
        
        try:
            result = subprocess.run(cmd, capture_output=True, text=True)
            return result.returncode
        except Exception as e:
            print(f"Error running black: {e}")
            return 1
    
    def _format_python_autopep8(self, file_path: Path, args) -> int:
        """Format Python code using autopep8"""
        cmd = ['autopep8']
        
        # Add options
        cmd.extend(['--aggressive', '--aggressive'])
        
        if not args.check:
            cmd.append('--in-place')
        
        cmd.append(str(file_path))
        
        try:
            result = subprocess.run(cmd, capture_output=True, text=True)
            return result.returncode
        except Exception as e:
            print(f"Error running autopep8: {e}")
            return 1
    
    def _format_shell(self, file_path: Path, args) -> int:
        """Format shell scripts using shfmt"""
        shfmt = shutil.which('shfmt')
        if not shfmt:
            print("shfmt not found (install with: go install mvdan.cc/sh/v3/cmd/shfmt@latest)")
            return 0  # Don't fail if shfmt is not available
        
        cmd = [shfmt]
        
        # Add options
        cmd.extend(['-i', '2'])  # 2-space indentation
        cmd.extend(['-ci'])      # Switch cases indent
        
        if not args.check:
            cmd.append('-w')
        else:
            cmd.append('-d')
        
        cmd.append(str(file_path))
        
        try:
            result = subprocess.run(cmd, capture_output=True, text=True)
            return result.returncode
        except Exception as e:
            print(f"Error running shfmt: {e}")
            return 1
    
    def _format_markdown(self, file_path: Path, args) -> int:
        """Format Markdown using prettier"""
        prettier = shutil.which('prettier')
        if not prettier:
            return 0  # Don't fail if prettier is not available
        
        cmd = [prettier]
        
        if not args.check:
            cmd.append('--write')
        else:
            cmd.append('--check')
        
        cmd.append(str(file_path))
        
        try:
            result = subprocess.run(cmd, capture_output=True, text=True)
            return result.returncode
        except Exception as e:
            print(f"Error running prettier: {e}")
            return 1
    
    def _format_json(self, file_path: Path, args) -> int:
        """Format JSON files"""
        try:
            import json
            
            with open(file_path, 'r') as f:
                data = json.load(f)
            
            if not args.check:
                with open(file_path, 'w') as f:
                    json.dump(data, f, indent=2, sort_keys=True)
                    f.write('\n')
            
            return 0
        except Exception as e:
            print(f"Error formatting JSON {file_path}: {e}")
            return 1
    
    def _format_yaml(self, file_path: Path, args) -> int:
        """Format YAML files"""
        try:
            import yaml
            
            with open(file_path, 'r') as f:
                data = yaml.safe_load(f)
            
            if not args.check:
                with open(file_path, 'w') as f:
                    yaml.dump(data, f, default_flow_style=False, indent=2)
            
            return 0
        except ImportError:
            print("PyYAML not available for YAML formatting")
            return 0
        except Exception as e:
            print(f"Error formatting YAML {file_path}: {e}")
            return 1
    
    def check_formatting(self, args) -> int:
        """Check if files are properly formatted"""
        args.check = True
        return self.format_project(args)
    
    def setup_git_hooks(self, args) -> int:
        """Set up Git pre-commit hooks for formatting"""
        git_dir = Path('.git')
        if not git_dir.exists():
            print("Not a Git repository")
            return 1
        
        hooks_dir = git_dir / 'hooks'
        hooks_dir.mkdir(exist_ok=True)
        
        pre_commit_hook = hooks_dir / 'pre-commit'
        
        hook_content = '''#!/bin/bash
# RustixOS pre-commit formatting hook

echo "Running code formatting checks..."

# Get list of staged files
STAGED_FILES=$(git diff --cached --name-only --diff-filter=ACM)

# Check if any source files are staged
RUST_FILES=$(echo "$STAGED_FILES" | grep -E '\\.rs$' || true)
C_FILES=$(echo "$STAGED_FILES" | grep -E '\\.(c|h|cpp|hpp)$' || true)
PYTHON_FILES=$(echo "$STAGED_FILES" | grep -E '\\.py$' || true)

# Format Rust files
if [ -n "$RUST_FILES" ]; then
    echo "Formatting Rust files..."
    echo "$RUST_FILES" | xargs rustfmt --check
    if [ $? -ne 0 ]; then
        echo "Rust formatting check failed. Run 'make format' to fix."
        exit 1
    fi
fi

# Format C/C++ files
if [ -n "$C_FILES" ]; then
    echo "Formatting C/C++ files..."
    echo "$C_FILES" | xargs clang-format --dry-run --Werror
    if [ $? -ne 0 ]; then
        echo "C/C++ formatting check failed. Run 'make format' to fix."
        exit 1
    fi
fi

# Format Python files
if [ -n "$PYTHON_FILES" ]; then
    echo "Formatting Python files..."
    if command -v black >/dev/null 2>&1; then
        echo "$PYTHON_FILES" | xargs black --check
        if [ $? -ne 0 ]; then
            echo "Python formatting check failed. Run 'make format' to fix."
            exit 1
        fi
    fi
fi

echo "All formatting checks passed!"
'''
        
        try:
            with open(pre_commit_hook, 'w') as f:
                f.write(hook_content)
            
            # Make executable
            os.chmod(pre_commit_hook, 0o755)
            
            print(f"Git pre-commit hook installed: {pre_commit_hook}")
            return 0
        except Exception as e:
            print(f"Error setting up Git hooks: {e}")
            return 1

def main():
    parser = argparse.ArgumentParser(description='RustixOS Code Formatting Script')
    parser.add_argument('--source-dir', default='.', help='Source directory to format')
    parser.add_argument('--languages', nargs='+', 
                       choices=['rust', 'c', 'cpp', 'python', 'shell', 'markdown', 'json', 'yaml'],
                       help='Languages to format')
    parser.add_argument('--exclude', nargs='+', help='Paths to exclude')
    parser.add_argument('--check', action='store_true', help='Check formatting without modifying files')
    parser.add_argument('--verbose', action='store_true', help='Verbose output')
    parser.add_argument('--stamp', help='Create stamp file on success')
    
    # Rust-specific options
    parser.add_argument('--rust-config', help='Rust formatting configuration file')
    parser.add_argument('--rust-edition', default='2021', help='Rust edition')
    
    # C/C++-specific options
    parser.add_argument('--c-style', default='Google', help='C/C++ formatting style')
    
    # Python-specific options
    parser.add_argument('--python-line-length', type=int, default=88, help='Python line length')
    
    # Git hooks
    parser.add_argument('--setup-git-hooks', action='store_true', help='Set up Git pre-commit hooks')
    
    args = parser.parse_args()
    
    formatter = CodeFormatter()
    
    if args.setup_git_hooks:
        return formatter.setup_git_hooks(args)
    
    return formatter.format_project(args)

if __name__ == '__main__':
    sys.exit(main())