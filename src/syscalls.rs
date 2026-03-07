#![cfg(target_arch = "x86_64")]
use crate::{filesystem::FILESYSTEM, process::PROCESS_MANAGER};
use alloc::{string::String, vec::Vec};
use x86_64::VirtAddr;

#[derive(Debug, Clone, Copy)]
#[repr(u64)]
pub enum SyscallNumber {
    Read = 0,
    Write = 1,
    Open = 2,
    Close = 3,
    Exit = 60,
    Fork = 57,
    Execve = 59,
    Getpid = 39,
    Kill = 62,
    Mkdir = 83,
    Rmdir = 84,
    Chdir = 80,
    Getcwd = 79,
}

impl SyscallNumber {
    pub fn from_u64(n: u64) -> Option<Self> {
        match n {
            0 => Some(Self::Read),
            1 => Some(Self::Write),
            2 => Some(Self::Open),
            3 => Some(Self::Close),
            60 => Some(Self::Exit),
            57 => Some(Self::Fork),
            59 => Some(Self::Execve),
            39 => Some(Self::Getpid),
            62 => Some(Self::Kill),
            83 => Some(Self::Mkdir),
            84 => Some(Self::Rmdir),
            80 => Some(Self::Chdir),
            79 => Some(Self::Getcwd),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct SyscallResult {
    pub value: i64,
    pub error: Option<&'static str>,
}

impl SyscallResult {
    pub fn ok(value: i64) -> Self {
        SyscallResult { value, error: None }
    }

    pub fn err(error: &'static str) -> Self {
        SyscallResult { value: -1, error: Some(error) }
    }
}

pub fn handle_syscall(
    syscall_number: u64,
    arg1: u64,
    arg2: u64,
    arg3: u64,
    _arg4: u64,
    _arg5: u64,
    _arg6: u64,
) -> SyscallResult {
    match SyscallNumber::from_u64(syscall_number) {
        Some(SyscallNumber::Read) => sys_read(arg1 as i32, arg2 as *mut u8, arg3 as usize),
        Some(SyscallNumber::Write) => sys_write(arg1 as i32, arg2 as *const u8, arg3 as usize),
        Some(SyscallNumber::Open) => sys_open(arg1 as *const u8, arg2 as i32),
        Some(SyscallNumber::Close) => sys_close(arg1 as i32),
        Some(SyscallNumber::Exit) => sys_exit(arg1 as i32),
        Some(SyscallNumber::Fork) => sys_fork(),
        Some(SyscallNumber::Execve) => sys_execve(arg1 as *const u8, arg2 as *const *const u8),
        Some(SyscallNumber::Getpid) => sys_getpid(),
        Some(SyscallNumber::Kill) => sys_kill(arg1 as i32, arg2 as i32),
        Some(SyscallNumber::Mkdir) => sys_mkdir(arg1 as *const u8),
        Some(SyscallNumber::Rmdir) => sys_rmdir(arg1 as *const u8),
        Some(SyscallNumber::Chdir) => sys_chdir(arg1 as *const u8),
        Some(SyscallNumber::Getcwd) => sys_getcwd(arg1 as *mut u8, arg2 as usize),
        None => SyscallResult::err("Invalid syscall number"),
    }
}

fn sys_read(fd: i32, buf: *mut u8, count: usize) -> SyscallResult {
    // For now, only support reading from stdin (fd 0)
    if fd != 0 {
        return SyscallResult::err("Invalid file descriptor");
    }

    // In a real implementation, this would read from keyboard buffer
    // For now, return 0 (EOF)
    SyscallResult::ok(0)
}

fn sys_write(fd: i32, buf: *const u8, count: usize) -> SyscallResult {
    if fd != 1 && fd != 2 {
        return SyscallResult::err("Invalid file descriptor");
    }

    unsafe {
        let slice = core::slice::from_raw_parts(buf, count);
        if let Ok(s) = core::str::from_utf8(slice) {
            crate::print!("{}", s);
            SyscallResult::ok(count as i64)
        } else {
            SyscallResult::err("Invalid UTF-8")
        }
    }
}

fn sys_open(pathname: *const u8, _flags: i32) -> SyscallResult {
    unsafe {
        let path_str = cstr_to_string(pathname);
        let fs = FILESYSTEM.lock();
        
        if fs.file_exists(&path_str) {
            // Return a fake file descriptor
            SyscallResult::ok(3)
        } else {
            SyscallResult::err("File not found")
        }
    }
}

fn sys_close(_fd: i32) -> SyscallResult {
    // For now, always succeed
    SyscallResult::ok(0)
}

fn sys_exit(status: i32) -> SyscallResult {
    if let Some(current_pid) = crate::process::get_current_process() {
        crate::println!("Process {:?} exiting with status {}", current_pid, status);
        let _ = crate::process::terminate_process(current_pid);
    }
    SyscallResult::ok(0)
}

fn sys_fork() -> SyscallResult {
    // Fork implementation would be complex - for now return error
    SyscallResult::err("Fork not implemented")
}

fn sys_execve(_filename: *const u8, _argv: *const *const u8) -> SyscallResult {
    // Execve implementation would be complex - for now return error
    SyscallResult::err("Execve not implemented")
}

fn sys_getpid() -> SyscallResult {
    if let Some(current_pid) = crate::process::get_current_process() {
        SyscallResult::ok(current_pid.0 as i64)
    } else {
        SyscallResult::ok(0)
    }
}

fn sys_kill(pid: i32, _signal: i32) -> SyscallResult {
    let target_pid = crate::process::ProcessId(pid as u64);
    match crate::process::terminate_process(target_pid) {
        Ok(()) => SyscallResult::ok(0),
        Err(e) => SyscallResult::err(e),
    }
}

fn sys_mkdir(pathname: *const u8) -> SyscallResult {
    unsafe {
        let path_str = cstr_to_string(pathname);
        let mut fs = FILESYSTEM.lock();
        
        match fs.create_directory(&path_str) {
            Ok(()) => SyscallResult::ok(0),
            Err(e) => SyscallResult::err(e),
        }
    }
}

fn sys_rmdir(pathname: *const u8) -> SyscallResult {
    unsafe {
        let path_str = cstr_to_string(pathname);
        let mut fs = FILESYSTEM.lock();
        
        match fs.delete_file(&path_str) {
            Ok(()) => SyscallResult::ok(0),
            Err(e) => SyscallResult::err(e),
        }
    }
}

fn sys_chdir(path: *const u8) -> SyscallResult {
    unsafe {
        let path_str = cstr_to_string(path);
        let mut fs = FILESYSTEM.lock();
        
        match fs.change_directory(&path_str) {
            Ok(()) => SyscallResult::ok(0),
            Err(e) => SyscallResult::err(e),
        }
    }
}

fn sys_getcwd(buf: *mut u8, size: usize) -> SyscallResult {
    let fs = FILESYSTEM.lock();
    let cwd = fs.get_current_directory();
    
    if cwd.len() >= size {
        return SyscallResult::err("Buffer too small");
    }
    
    unsafe {
        let slice = core::slice::from_raw_parts_mut(buf, size);
        let cwd_bytes = cwd.as_bytes();
        slice[..cwd_bytes.len()].copy_from_slice(cwd_bytes);
        slice[cwd_bytes.len()] = 0; // null terminator
    }
    
    SyscallResult::ok(cwd.len() as i64)
}

unsafe fn cstr_to_string(ptr: *const u8) -> String {
    let mut len = 0;
    while *ptr.add(len) != 0 {
        len += 1;
    }
    
    let slice = core::slice::from_raw_parts(ptr, len);
    String::from_utf8_lossy(slice).into_owned()
}

// System call handler that would be called from assembly
#[no_mangle]
pub extern "C" fn syscall_handler(
    rax: u64, rdi: u64, rsi: u64, rdx: u64, 
    r10: u64, r8: u64, r9: u64
) -> i64 {
    let result = handle_syscall(rax, rdi, rsi, rdx, r10, r8, r9);
    
    if let Some(error) = result.error {
        crate::println!("Syscall error: {}", error);
    }
    
    result.value
}

pub fn init() {
    crate::println!("System calls initialized");
}

// File descriptor table
use alloc::collections::BTreeMap;
use spin::Mutex;
use lazy_static::lazy_static;

#[derive(Debug, Clone)]
pub struct FileDescriptor {
    pub path: String,
    pub offset: usize,
    pub flags: i32,
}

lazy_static! {
    static ref FD_TABLE: Mutex<BTreeMap<i32, FileDescriptor>> = {
        let mut table = BTreeMap::new();
        // Standard file descriptors
        table.insert(0, FileDescriptor { path: "/dev/stdin".into(), offset: 0, flags: 0 });
        table.insert(1, FileDescriptor { path: "/dev/stdout".into(), offset: 0, flags: 1 });
        table.insert(2, FileDescriptor { path: "/dev/stderr".into(), offset: 0, flags: 1 });
        Mutex::new(table)
    };
}

static NEXT_FD: core::sync::atomic::AtomicI32 = core::sync::atomic::AtomicI32::new(3);

pub fn allocate_fd(path: String, flags: i32) -> i32 {
    let fd = NEXT_FD.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    let descriptor = FileDescriptor { path, offset: 0, flags };
    FD_TABLE.lock().insert(fd, descriptor);
    fd
}

pub fn deallocate_fd(fd: i32) -> Result<(), &'static str> {
    match FD_TABLE.lock().remove(&fd) {
        Some(_) => Ok(()),
        None => Err("Invalid file descriptor"),
    }
}