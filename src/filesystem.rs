#![cfg(target_arch = "x86_64")]
use alloc::{
    collections::BTreeMap,
    string::{String, ToString},
    vec::Vec,
    format,
};
use spin::Mutex;
use lazy_static::lazy_static;

#[derive(Debug, Clone)]
pub struct File {
    pub name: String,
    pub content: Vec<u8>,
    pub size: usize,
    pub is_directory: bool,
}

impl File {
    pub fn new_file(name: String, content: Vec<u8>) -> Self {
        let size = content.len();
        File {
            name,
            content,
            size,
            is_directory: false,
        }
    }

    pub fn new_directory(name: String) -> Self {
        File {
            name,
            content: Vec::new(),
            size: 0,
            is_directory: true,
        }
    }
}

pub struct FileSystem {
    files: BTreeMap<String, File>,
    current_directory: String,
}

impl FileSystem {
    pub fn new() -> Self {
        let mut fs = FileSystem {
            files: BTreeMap::new(),
            current_directory: "/".to_string(),
        };
        
        // Create root directory
        fs.files.insert("/".to_string(), File::new_directory("/".to_string()));
        
        // Create some initial files and directories
        fs.create_file("/hello.txt", b"Hello, RustixOS!".to_vec()).ok();
        fs.create_file("/readme.md", b"# RustixOS\n\nA Rust-based operating system kernel.".to_vec()).ok();
        fs.create_directory("/bin").ok();
        fs.create_directory("/etc").ok();
        fs.create_directory("/home").ok();
        fs.create_directory("/tmp").ok();
        
        fs
    }

    pub fn create_file(&mut self, path: &str, content: Vec<u8>) -> Result<(), &'static str> {
        let path = self.normalize_path(path);
        if self.files.contains_key(&path) {
            return Err("File already exists");
        }
        
        let file = File::new_file(path.clone(), content);
        self.files.insert(path, file);
        Ok(())
    }

    pub fn create_directory(&mut self, path: &str) -> Result<(), &'static str> {
        let path = self.normalize_path(path);
        if self.files.contains_key(&path) {
            return Err("Directory already exists");
        }
        
        let dir = File::new_directory(path.clone());
        self.files.insert(path, dir);
        Ok(())
    }

    pub fn read_file(&self, path: &str) -> Result<&Vec<u8>, &'static str> {
        let path = self.normalize_path(path);
        match self.files.get(&path) {
            Some(file) if !file.is_directory => Ok(&file.content),
            Some(_) => Err("Path is a directory"),
            None => Err("File not found"),
        }
    }

    pub fn write_file(&mut self, path: &str, content: Vec<u8>) -> Result<(), &'static str> {
        let path = self.normalize_path(path);
        match self.files.get_mut(&path) {
            Some(file) if !file.is_directory => {
                file.content = content;
                file.size = file.content.len();
                Ok(())
            }
            Some(_) => Err("Path is a directory"),
            None => self.create_file(&path, content),
        }
    }

    pub fn delete_file(&mut self, path: &str) -> Result<(), &'static str> {
        let path = self.normalize_path(path);
        match self.files.remove(&path) {
            Some(_) => Ok(()),
            None => Err("File not found"),
        }
    }

    pub fn list_directory(&self, path: &str) -> Result<Vec<String>, &'static str> {
        let path = self.normalize_path(path);
        
        if !self.files.contains_key(&path) {
            return Err("Directory not found");
        }
        
        if let Some(file) = self.files.get(&path) {
            if !file.is_directory {
                return Err("Path is not a directory");
            }
        }

        let mut entries = Vec::new();
        let prefix = if path == "/" { "/" } else { &format!("{}/", path) };
        
        for file_path in self.files.keys() {
            if file_path.starts_with(prefix) && file_path != &path {
                let relative_path = &file_path[prefix.len()..];
                if !relative_path.contains('/') {
                    entries.push(relative_path.to_string());
                }
            }
        }
        
        Ok(entries)
    }

    pub fn change_directory(&mut self, path: &str) -> Result<(), &'static str> {
        let path = self.normalize_path(path);
        
        match self.files.get(&path) {
            Some(file) if file.is_directory => {
                self.current_directory = path;
                Ok(())
            }
            Some(_) => Err("Path is not a directory"),
            None => Err("Directory not found"),
        }
    }

    pub fn get_current_directory(&self) -> &str {
        &self.current_directory
    }

    fn normalize_path(&self, path: &str) -> String {
        if path.starts_with('/') {
            path.to_string()
        } else {
            if self.current_directory == "/" {
                format!("/{}", path)
            } else {
                format!("{}/{}", self.current_directory, path)
            }
        }
    }

    pub fn file_exists(&self, path: &str) -> bool {
        let path = self.normalize_path(path);
        self.files.contains_key(&path)
    }

    pub fn is_directory(&self, path: &str) -> bool {
        let path = self.normalize_path(path);
        self.files.get(&path)
            .map(|f| f.is_directory)
            .unwrap_or(false)
    }
}

lazy_static! {
    pub static ref FILESYSTEM: Mutex<FileSystem> = Mutex::new(FileSystem::new());
}

pub fn init() {
    crate::println!("Filesystem initialized");
}

// VFS (Virtual File System) interface
pub trait VfsNode {
    fn read(&self, offset: usize, buffer: &mut [u8]) -> Result<usize, &'static str>;
    fn write(&mut self, offset: usize, buffer: &[u8]) -> Result<usize, &'static str>;
    fn get_size(&self) -> usize;
    fn is_directory(&self) -> bool;
}

impl VfsNode for File {
    fn read(&self, offset: usize, buffer: &mut [u8]) -> Result<usize, &'static str> {
        if self.is_directory {
            return Err("Cannot read from directory");
        }
        
        if offset >= self.content.len() {
            return Ok(0);
        }
        
        let end = core::cmp::min(offset + buffer.len(), self.content.len());
        let bytes_to_read = end - offset;
        
        buffer[..bytes_to_read].copy_from_slice(&self.content[offset..end]);
        Ok(bytes_to_read)
    }

    fn write(&mut self, offset: usize, buffer: &[u8]) -> Result<usize, &'static str> {
        if self.is_directory {
            return Err("Cannot write to directory");
        }
        
        if offset + buffer.len() > self.content.len() {
            self.content.resize(offset + buffer.len(), 0);
        }
        
        self.content[offset..offset + buffer.len()].copy_from_slice(buffer);
        self.size = self.content.len();
        Ok(buffer.len())
    }

    fn get_size(&self) -> usize {
        self.size
    }

    fn is_directory(&self) -> bool {
        self.is_directory
    }
}