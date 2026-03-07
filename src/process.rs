#![cfg(target_arch = "x86_64")]
use alloc::{
    collections::{BTreeMap, VecDeque},
    string::String,
    vec::Vec,
};
use spin::Mutex;
use lazy_static::lazy_static;
use x86_64::VirtAddr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProcessId(pub u64);

impl ProcessId {
    pub fn new() -> Self {
        use core::sync::atomic::{AtomicU64, Ordering};
        static NEXT_PID: AtomicU64 = AtomicU64::new(1);
        ProcessId(NEXT_PID.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Ready,
    Running,
    Blocked,
    Terminated,
}

#[derive(Debug)]
pub struct Process {
    pub id: ProcessId,
    pub name: String,
    pub state: ProcessState,
    pub stack_pointer: VirtAddr,
    pub instruction_pointer: VirtAddr,
    pub page_table: Option<VirtAddr>,
    pub parent_id: Option<ProcessId>,
    pub children: Vec<ProcessId>,
    pub priority: u8,
    pub cpu_time: u64,
}

impl Process {
    pub fn new(name: String, entry_point: VirtAddr) -> Self {
        Process {
            id: ProcessId::new(),
            name,
            state: ProcessState::Ready,
            stack_pointer: VirtAddr::new(0),
            instruction_pointer: entry_point,
            page_table: None,
            parent_id: None,
            children: Vec::new(),
            priority: 10,
            cpu_time: 0,
        }
    }

    pub fn kernel_process(name: String) -> Self {
        Process {
            id: ProcessId::new(),
            name,
            state: ProcessState::Running,
            stack_pointer: VirtAddr::new(0),
            instruction_pointer: VirtAddr::new(0),
            page_table: None,
            parent_id: None,
            children: Vec::new(),
            priority: 0,
            cpu_time: 0,
        }
    }
}

pub struct ProcessManager {
    processes: BTreeMap<ProcessId, Process>,
    ready_queue: VecDeque<ProcessId>,
    current_process: Option<ProcessId>,
    scheduler_ticks: u64,
}

impl ProcessManager {
    pub fn new() -> Self {
        let mut pm = ProcessManager {
            processes: BTreeMap::new(),
            ready_queue: VecDeque::new(),
            current_process: None,
            scheduler_ticks: 0,
        };

        // Create kernel idle process
        let idle_process = Process::kernel_process("idle".into());
        let idle_pid = idle_process.id;
        pm.processes.insert(idle_pid, idle_process);
        pm.current_process = Some(idle_pid);

        pm
    }

    pub fn create_process(&mut self, name: String, entry_point: VirtAddr) -> ProcessId {
        let process = Process::new(name, entry_point);
        let pid = process.id;
        
        self.processes.insert(pid, process);
        self.ready_queue.push_back(pid);
        
        pid
    }

    pub fn get_process(&self, pid: ProcessId) -> Option<&Process> {
        self.processes.get(&pid)
    }

    pub fn get_process_mut(&mut self, pid: ProcessId) -> Option<&mut Process> {
        self.processes.get_mut(&pid)
    }

    pub fn terminate_process(&mut self, pid: ProcessId) -> Result<(), &'static str> {
        if let Some(process) = self.processes.get_mut(&pid) {
            process.state = ProcessState::Terminated;
            
            // Remove from ready queue if present
            self.ready_queue.retain(|&p| p != pid);
            
            // If this is the current process, schedule next
            if self.current_process == Some(pid) {
                self.current_process = None;
                self.schedule();
            }
            
            Ok(())
        } else {
            Err("Process not found")
        }
    }

    pub fn schedule(&mut self) -> Option<ProcessId> {
        self.scheduler_ticks += 1;

        // Simple round-robin scheduler
        if let Some(current_pid) = self.current_process {
            if let Some(current_process) = self.processes.get_mut(&current_pid) {
                if current_process.state == ProcessState::Running {
                    current_process.state = ProcessState::Ready;
                    self.ready_queue.push_back(current_pid);
                }
            }
        }

        // Get next ready process
        while let Some(next_pid) = self.ready_queue.pop_front() {
            if let Some(next_process) = self.processes.get_mut(&next_pid) {
                if next_process.state == ProcessState::Ready {
                    next_process.state = ProcessState::Running;
                    self.current_process = Some(next_pid);
                    return Some(next_pid);
                }
            }
        }

        // No ready processes, return to idle
        if let Some(idle_process) = self.processes.values_mut().find(|p| p.name == "idle") {
            idle_process.state = ProcessState::Running;
            self.current_process = Some(idle_process.id);
            Some(idle_process.id)
        } else {
            None
        }
    }

    pub fn get_current_process(&self) -> Option<ProcessId> {
        self.current_process
    }

    pub fn block_process(&mut self, pid: ProcessId) -> Result<(), &'static str> {
        if let Some(process) = self.processes.get_mut(&pid) {
            process.state = ProcessState::Blocked;
            self.ready_queue.retain(|&p| p != pid);
            
            if self.current_process == Some(pid) {
                self.current_process = None;
                self.schedule();
            }
            
            Ok(())
        } else {
            Err("Process not found")
        }
    }

    pub fn unblock_process(&mut self, pid: ProcessId) -> Result<(), &'static str> {
        if let Some(process) = self.processes.get_mut(&pid) {
            if process.state == ProcessState::Blocked {
                process.state = ProcessState::Ready;
                self.ready_queue.push_back(pid);
            }
            Ok(())
        } else {
            Err("Process not found")
        }
    }

    pub fn list_processes(&self) -> Vec<&Process> {
        self.processes.values().collect()
    }

    pub fn get_scheduler_ticks(&self) -> u64 {
        self.scheduler_ticks
    }
}

lazy_static! {
    pub static ref PROCESS_MANAGER: Mutex<ProcessManager> = Mutex::new(ProcessManager::new());
}

pub fn init() {
    crate::println!("Process management initialized");
}

pub fn create_process(name: String, entry_point: VirtAddr) -> ProcessId {
    PROCESS_MANAGER.lock().create_process(name, entry_point)
}

pub fn schedule() -> Option<ProcessId> {
    PROCESS_MANAGER.lock().schedule()
}

pub fn get_current_process() -> Option<ProcessId> {
    PROCESS_MANAGER.lock().get_current_process()
}

pub fn terminate_process(pid: ProcessId) -> Result<(), &'static str> {
    PROCESS_MANAGER.lock().terminate_process(pid)
}

pub fn block_current_process() -> Result<(), &'static str> {
    if let Some(current_pid) = get_current_process() {
        PROCESS_MANAGER.lock().block_process(current_pid)
    } else {
        Err("No current process")
    }
}

pub fn unblock_process(pid: ProcessId) -> Result<(), &'static str> {
    PROCESS_MANAGER.lock().unblock_process(pid)
}

// Context switching structure
#[repr(C)]
pub struct Context {
    pub rsp: u64,
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub rbx: u64,
    pub rbp: u64,
}

impl Context {
    pub fn new() -> Self {
        Context {
            rsp: 0,
            r15: 0,
            r14: 0,
            r13: 0,
            r12: 0,
            rbx: 0,
            rbp: 0,
        }
    }
}

// Assembly function for context switching (would be implemented in assembly)
extern "C" {
    fn switch_context(old_context: *mut Context, new_context: *const Context);
}

pub fn context_switch(old_pid: ProcessId, new_pid: ProcessId) {
    // This would perform actual context switching in a real implementation
    crate::println!("Context switch from {:?} to {:?}", old_pid, new_pid);
}