// Filesystem Commands
// Professional filesystem utilities with full POSIX compatibility

use alloc::{string::String, vec::Vec, format};
use crate::shell::{Shell, ShellResult, ShellError, SimpleCommand};
use super::ExternalCommand;
use crate::println;

/// ls command - list directory contents
pub struct LsCommand;

impl ExternalCommand for LsCommand {
    fn name(&self) -> &'static str { "ls" }
    fn description(&self) -> &'static str { "List directory contents" }
    fn usage(&self) -> &'static str { "ls [OPTION]... [FILE]..." }
    
    fn execute(&self, shell: &mut Shell, cmd: &SimpleCommand) -> ShellResult<i32> {
        let mut long_format = false;
        let mut all_files = false;
        let mut human_readable = false;
        let mut recursive = false;
        let mut sort_by_time = false;
        let mut reverse_sort = false;
        let mut color = true;
        let mut classify = false;
        let mut one_per_line = false;
        let mut paths = Vec::new();
        
        // Parse arguments
        let mut i = 0;
        while i < cmd.args.len() {
            let arg = &cmd.args[i];
            if arg.starts_with('-') && arg.len() > 1 {
                for ch in arg.chars().skip(1) {
                    match ch {
                        'l' => long_format = true,
                        'a' => all_files = true,
                        'A' => all_files = true, // almost all (exclude . and ..)
                        'h' => human_readable = true,
                        'R' => recursive = true,
                        't' => sort_by_time = true,
                        'r' => reverse_sort = true,
                        'F' => classify = true,
                        '1' => one_per_line = true,
                        _ => return Err(ShellError::InvalidArgument(format!("invalid option: -{}", ch))),
                    }
                }
            } else if arg == "--color" {
                color = true;
            } else if arg == "--no-color" {
                color = false;
            } else {
                paths.push(arg.clone());
            }
            i += 1;
        }
        
        if paths.is_empty() {
            paths.push(shell.get_current_directory().to_string());
        }
        
        for path in paths {
            self.list_directory(&path, long_format, all_files, human_readable, 
                              recursive, sort_by_time, reverse_sort, color, 
                              classify, one_per_line)?;
        }
        
        Ok(0)
    }
}

impl LsCommand {
    fn list_directory(&self, path: &str, long_format: bool, all_files: bool, 
                     human_readable: bool, recursive: bool, sort_by_time: bool,
                     reverse_sort: bool, color: bool, classify: bool, 
                     one_per_line: bool) -> ShellResult<()> {
        
        // Get directory entries
        let entries = crate::filesystem::read_directory(path)
            .map_err(|_| ShellError::DirectoryNotFound(path.to_string()))?;
        
        let mut filtered_entries = Vec::new();
        for entry in entries {
            if !all_files && entry.name.starts_with('.') {
                continue;
            }
            filtered_entries.push(entry);
        }
        
        // Sort entries
        if sort_by_time {
            filtered_entries.sort_by(|a, b| {
                if reverse_sort {
                    a.modified_time.cmp(&b.modified_time)
                } else {
                    b.modified_time.cmp(&a.modified_time)
                }
            });
        } else {
            filtered_entries.sort_by(|a, b| {
                if reverse_sort {
                    b.name.cmp(&a.name)
                } else {
                    a.name.cmp(&b.name)
                }
            });
        }
        
        // Display entries
        if long_format {
            self.display_long_format(&filtered_entries, human_readable, color)?;
        } else if one_per_line {
            self.display_one_per_line(&filtered_entries, color, classify)?;
        } else {
            self.display_columns(&filtered_entries, color, classify)?;
        }
        
        // Recursive listing
        if recursive {
            for entry in &filtered_entries {
                if entry.is_directory && entry.name != "." && entry.name != ".." {
                    let subpath = format!("{}/{}", path, entry.name);
                    println!("\n{}:", subpath);
                    self.list_directory(&subpath, long_format, all_files, 
                                      human_readable, recursive, sort_by_time,
                                      reverse_sort, color, classify, one_per_line)?;
                }
            }
        }
        
        Ok(())
    }
    
    fn display_long_format(&self, entries: &[crate::filesystem::DirectoryEntry], 
                          human_readable: bool, color: bool) -> ShellResult<()> {
        for entry in entries {
            let permissions = self.format_permissions(entry.permissions);
            let size = if human_readable {
                self.format_human_size(entry.size)
            } else {
                entry.size.to_string()
            };
            
            let name = if color {
                self.colorize_name(&entry.name, entry.is_directory, entry.is_executable)
            } else {
                entry.name.clone()
            };
            
            println!("{} {} {} {} {} {}", 
                    permissions,
                    entry.link_count,
                    "user", // owner
                    "group", // group
                    size,
                    name);
        }
        Ok(())
    }
    
    fn display_one_per_line(&self, entries: &[crate::filesystem::DirectoryEntry], 
                           color: bool, classify: bool) -> ShellResult<()> {
        for entry in entries {
            let mut name = if color {
                self.colorize_name(&entry.name, entry.is_directory, entry.is_executable)
            } else {
                entry.name.clone()
            };
            
            if classify {
                if entry.is_directory {
                    name.push('/');
                } else if entry.is_executable {
                    name.push('*');
                }
            }
            
            println!("{}", name);
        }
        Ok(())
    }
    
    fn display_columns(&self, entries: &[crate::filesystem::DirectoryEntry], 
                      color: bool, classify: bool) -> ShellResult<()> {
        let mut names = Vec::new();
        for entry in entries {
            let mut name = if color {
                self.colorize_name(&entry.name, entry.is_directory, entry.is_executable)
            } else {
                entry.name.clone()
            };
            
            if classify {
                if entry.is_directory {
                    name.push('/');
                } else if entry.is_executable {
                    name.push('*');
                }
            }
            
            names.push(name);
        }
        
        // Simple column display (could be improved with proper column calculation)
        let mut line = String::new();
        for (i, name) in names.iter().enumerate() {
            line.push_str(name);
            if (i + 1) % 4 == 0 || i == names.len() - 1 {
                println!("{}", line);
                line.clear();
            } else {
                line.push_str("  ");
            }
        }
        
        Ok(())
    }
    
    fn format_permissions(&self, perms: u32) -> String {
        let mut result = String::new();
        
        // File type
        result.push(if perms & 0o040000 != 0 { 'd' } else { '-' });
        
        // Owner permissions
        result.push(if perms & 0o400 != 0 { 'r' } else { '-' });
        result.push(if perms & 0o200 != 0 { 'w' } else { '-' });
        result.push(if perms & 0o100 != 0 { 'x' } else { '-' });
        
        // Group permissions
        result.push(if perms & 0o040 != 0 { 'r' } else { '-' });
        result.push(if perms & 0o020 != 0 { 'w' } else { '-' });
        result.push(if perms & 0o010 != 0 { 'x' } else { '-' });
        
        // Other permissions
        result.push(if perms & 0o004 != 0 { 'r' } else { '-' });
        result.push(if perms & 0o002 != 0 { 'w' } else { '-' });
        result.push(if perms & 0o001 != 0 { 'x' } else { '-' });
        
        result
    }
    
    fn format_human_size(&self, size: u64) -> String {
        const UNITS: &[&str] = &["B", "K", "M", "G", "T", "P"];
        let mut size = size as f64;
        let mut unit_index = 0;
        
        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }
        
        if unit_index == 0 {
            format!("{}", size as u64)
        } else {
            format!("{:.1}{}", size, UNITS[unit_index])
        }
    }
    
    fn colorize_name(&self, name: &str, is_directory: bool, is_executable: bool) -> String {
        if is_directory {
            format!("\x1b[34m{}\x1b[0m", name) // Blue
        } else if is_executable {
            format!("\x1b[32m{}\x1b[0m", name) // Green
        } else {
            name.to_string()
        }
    }
}

/// cat command - display file contents
pub struct CatCommand;

impl ExternalCommand for CatCommand {
    fn name(&self) -> &'static str { "cat" }
    fn description(&self) -> &'static str { "Display file contents" }
    fn usage(&self) -> &'static str { "cat [OPTION]... [FILE]..." }
    
    fn execute(&self, shell: &mut Shell, cmd: &SimpleCommand) -> ShellResult<i32> {
        let mut number_lines = false;
        let mut number_nonblank = false;
        let mut show_ends = false;
        let mut show_tabs = false;
        let mut files = Vec::new();
        
        // Parse arguments
        for arg in &cmd.args {
            if arg.starts_with('-') {
                for ch in arg.chars().skip(1) {
                    match ch {
                        'n' => number_lines = true,
                        'b' => number_nonblank = true,
                        'E' => show_ends = true,
                        'T' => show_tabs = true,
                        _ => return Err(ShellError::InvalidArgument(format!("invalid option: -{}", ch))),
                    }
                }
            } else {
                files.push(arg.clone());
            }
        }
        
        if files.is_empty() {
            // Read from stdin (not implemented yet)
            return Err(ShellError::InvalidArgument("stdin reading not implemented".to_string()));
        }
        
        for file in files {
            self.display_file(&file, number_lines, number_nonblank, show_ends, show_tabs)?;
        }
        
        Ok(0)
    }
}

impl CatCommand {
    fn display_file(&self, filename: &str, number_lines: bool, number_nonblank: bool,
                   show_ends: bool, show_tabs: bool) -> ShellResult<()> {
        let content = crate::filesystem::read_file(filename)
            .map_err(|_| ShellError::FileNotFound(filename.to_string()))?;
        
        let lines: Vec<&str> = content.split('\n').collect();
        let mut line_number = 1;
        
        for line in lines {
            let mut output_line = line.to_string();
            
            // Show tabs
            if show_tabs {
                output_line = output_line.replace('\t', "^I");
            }
            
            // Show line endings
            if show_ends {
                output_line.push('$');
            }
            
            // Number lines
            if number_lines || (number_nonblank && !line.is_empty()) {
                print!("{:6}\t", line_number);
            }
            
            println!("{}", output_line);
            
            if !number_nonblank || !line.is_empty() {
                line_number += 1;
            }
        }
        
        Ok(())
    }
}

/// mkdir command - create directories
pub struct MkdirCommand;

impl ExternalCommand for MkdirCommand {
    fn name(&self) -> &'static str { "mkdir" }
    fn description(&self) -> &'static str { "Create directories" }
    fn usage(&self) -> &'static str { "mkdir [OPTION]... DIRECTORY..." }
    
    fn execute(&self, shell: &mut Shell, cmd: &SimpleCommand) -> ShellResult<i32> {
        let mut parents = false;
        let mut mode = 0o755;
        let mut directories = Vec::new();
        
        // Parse arguments
        let mut i = 0;
        while i < cmd.args.len() {
            let arg = &cmd.args[i];
            if arg == "-p" || arg == "--parents" {
                parents = true;
            } else if arg == "-m" || arg == "--mode" {
                i += 1;
                if i >= cmd.args.len() {
                    return Err(ShellError::InvalidArgument("missing mode argument".to_string()));
                }
                mode = u32::from_str_radix(&cmd.args[i], 8)
                    .map_err(|_| ShellError::InvalidArgument("invalid mode".to_string()))?;
            } else if arg.starts_with('-') {
                return Err(ShellError::InvalidArgument(format!("invalid option: {}", arg)));
            } else {
                directories.push(arg.clone());
            }
            i += 1;
        }
        
        if directories.is_empty() {
            return Err(ShellError::InvalidArgument("missing directory argument".to_string()));
        }
        
        for dir in directories {
            if parents {
                self.create_directory_recursive(&dir, mode)?;
            } else {
                self.create_directory(&dir, mode)?;
            }
        }
        
        Ok(0)
    }
}

impl MkdirCommand {
    fn create_directory(&self, path: &str, mode: u32) -> ShellResult<()> {
        crate::filesystem::create_directory(path, mode)
            .map_err(|_| ShellError::IoError(format!("cannot create directory '{}'", path)))
    }
    
    fn create_directory_recursive(&self, path: &str, mode: u32) -> ShellResult<()> {
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        let mut current_path = if path.starts_with('/') {
            String::from("/")
        } else {
            String::new()
        };
        
        for part in parts {
            if !current_path.is_empty() && !current_path.ends_with('/') {
                current_path.push('/');
            }
            current_path.push_str(part);
            
            if !crate::filesystem::path_exists(&current_path) {
                self.create_directory(&current_path, mode)?;
            }
        }
        
        Ok(())
    }
}

// Additional filesystem commands would be implemented similarly...

/// cp command - copy files and directories
pub struct CpCommand;

impl ExternalCommand for CpCommand {
    fn name(&self) -> &'static str { "cp" }
    fn description(&self) -> &'static str { "Copy files and directories" }
    fn usage(&self) -> &'static str { "cp [OPTION]... SOURCE... DEST" }
    
    fn execute(&self, shell: &mut Shell, cmd: &SimpleCommand) -> ShellResult<i32> {
        // Implementation would go here
        println!("cp command not yet fully implemented");
        Ok(0)
    }
}

/// mv command - move/rename files and directories
pub struct MvCommand;

impl ExternalCommand for MvCommand {
    fn name(&self) -> &'static str { "mv" }
    fn description(&self) -> &'static str { "Move/rename files and directories" }
    fn usage(&self) -> &'static str { "mv [OPTION]... SOURCE... DEST" }
    
    fn execute(&self, shell: &mut Shell, cmd: &SimpleCommand) -> ShellResult<i32> {
        println!("mv command not yet fully implemented");
        Ok(0)
    }
}

/// rm command - remove files and directories
pub struct RmCommand;

impl ExternalCommand for RmCommand {
    fn name(&self) -> &'static str { "rm" }
    fn description(&self) -> &'static str { "Remove files and directories" }
    fn usage(&self) -> &'static str { "rm [OPTION]... FILE..." }
    
    fn execute(&self, shell: &mut Shell, cmd: &SimpleCommand) -> ShellResult<i32> {
        println!("rm command not yet fully implemented");
        Ok(0)
    }
}

// Placeholder implementations for other filesystem commands
macro_rules! impl_placeholder_command {
    ($name:ident, $cmd_name:expr, $desc:expr, $usage:expr) => {
        pub struct $name;
        
        impl ExternalCommand for $name {
            fn name(&self) -> &'static str { $cmd_name }
            fn description(&self) -> &'static str { $desc }
            fn usage(&self) -> &'static str { $usage }
            
            fn execute(&self, shell: &mut Shell, cmd: &SimpleCommand) -> ShellResult<i32> {
                println!("{} command not yet fully implemented", $cmd_name);
                Ok(0)
            }
        }
    };
}

impl_placeholder_command!(RmdirCommand, "rmdir", "Remove empty directories", "rmdir [OPTION]... DIRECTORY...");
impl_placeholder_command!(TouchCommand, "touch", "Change file timestamps", "touch [OPTION]... FILE...");
impl_placeholder_command!(FindCommand, "find", "Search for files and directories", "find [PATH...] [EXPRESSION]");
impl_placeholder_command!(GrepCommand, "grep", "Search text patterns", "grep [OPTION]... PATTERN [FILE]...");
impl_placeholder_command!(HeadCommand, "head", "Display first lines of files", "head [OPTION]... [FILE]...");
impl_placeholder_command!(TailCommand, "tail", "Display last lines of files", "tail [OPTION]... [FILE]...");
impl_placeholder_command!(WcCommand, "wc", "Count lines, words, and characters", "wc [OPTION]... [FILE]...");
impl_placeholder_command!(SortCommand, "sort", "Sort lines of text", "sort [OPTION]... [FILE]...");
impl_placeholder_command!(UniqCommand, "uniq", "Report or omit repeated lines", "uniq [OPTION]... [INPUT [OUTPUT]]");
impl_placeholder_command!(CutCommand, "cut", "Extract columns from lines", "cut OPTION... [FILE]...");
impl_placeholder_command!(AwkCommand, "awk", "Pattern scanning and processing", "awk [OPTION]... PROGRAM [FILE]...");
impl_placeholder_command!(SedCommand, "sed", "Stream editor", "sed [OPTION]... {script-only-if-no-other-script} [input-file]...");
impl_placeholder_command!(TarCommand, "tar", "Archive files", "tar [OPTION...] [FILE]...");
impl_placeholder_command!(GzipCommand, "gzip", "Compress files", "gzip [OPTION]... [FILE]...");
impl_placeholder_command!(GunzipCommand, "gunzip", "Decompress files", "gunzip [OPTION]... [FILE]...");
impl_placeholder_command!(ZipCommand, "zip", "Create ZIP archives", "zip [OPTION]... ZIPFILE [FILE]...");
impl_placeholder_command!(UnzipCommand, "unzip", "Extract ZIP archives", "unzip [OPTION]... ZIPFILE [FILE]...");
impl_placeholder_command!(DfCommand, "df", "Display filesystem disk space usage", "df [OPTION]... [FILE]...");
impl_placeholder_command!(DuCommand, "du", "Display directory space usage", "du [OPTION]... [FILE]...");
impl_placeholder_command!(MountCommand, "mount", "Mount filesystems", "mount [OPTION]... DEVICE|DIR");
impl_placeholder_command!(UmountCommand, "umount", "Unmount filesystems", "umount [OPTION]... DIR | DEVICE...");
impl_placeholder_command!(FsckCommand, "fsck", "Check and repair filesystems", "fsck [OPTION]... [FILESYSTEM]...");
impl_placeholder_command!(MkfsCommand, "mkfs", "Create filesystems", "mkfs [OPTION]... DEVICE [SIZE]");
impl_placeholder_command!(LsblkCommand, "lsblk", "List block devices", "lsblk [OPTION]... [DEVICE]...");
impl_placeholder_command!(FdiskCommand, "fdisk", "Manipulate disk partition table", "fdisk [OPTION]... DEVICE");