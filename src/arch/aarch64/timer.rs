// AArch64 Generic Timer Implementation
// Professional timer management with high precision and advanced features

use core::arch::asm;
use crate::println;

/// Generic Timer registers and functionality
pub struct GenericTimer {
    frequency: u64,
    enabled: bool,
}

/// Timer types available in AArch64
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerType {
    Physical,      // EL1 Physical Timer
    Virtual,       // EL1 Virtual Timer
    Hypervisor,    // EL2 Physical Timer
    SecurePhysical, // EL3 Physical Timer
}

/// Timer interrupt configuration
#[derive(Debug, Clone, Copy)]
pub struct TimerConfig {
    pub timer_type: TimerType,
    pub interrupt_enabled: bool,
    pub auto_reload: bool,
    pub initial_count: u64,
}

impl Default for TimerConfig {
    fn default() -> Self {
        TimerConfig {
            timer_type: TimerType::Virtual,
            interrupt_enabled: true,
            auto_reload: false,
            initial_count: 0,
        }
    }
}

static mut GENERIC_TIMER: Option<GenericTimer> = None;

impl GenericTimer {
    pub fn new() -> Self {
        GenericTimer {
            frequency: 0,
            enabled: false,
        }
    }
    
    pub fn init(&mut self) -> Result<(), &'static str> {
        // Read timer frequency from CNTFRQ_EL0
        self.frequency = self.read_frequency();
        
        if self.frequency == 0 {
            return Err("Timer frequency is zero");
        }
        
        println!("Generic Timer frequency: {} Hz", self.frequency);
        
        // Initialize virtual timer as default
        self.init_virtual_timer()?;
        
        self.enabled = true;
        println!("Generic Timer initialized successfully");
        
        Ok(())
    }
    
    pub fn read_frequency(&self) -> u64 {
        let freq: u64;
        unsafe {
            asm!("mrs {}, CNTFRQ_EL0", out(reg) freq);
        }
        freq
    }
    
    pub fn read_physical_counter(&self) -> u64 {
        let count: u64;
        unsafe {
            asm!("mrs {}, CNTPCT_EL0", out(reg) count);
        }
        count
    }
    
    pub fn read_virtual_counter(&self) -> u64 {
        let count: u64;
        unsafe {
            asm!("mrs {}, CNTVCT_EL0", out(reg) count);
        }
        count
    }
    
    pub fn read_virtual_offset(&self) -> u64 {
        let offset: u64;
        unsafe {
            asm!("mrs {}, CNTVOFF_EL2", out(reg) offset);
        }
        offset
    }
    
    pub fn set_virtual_offset(&self, offset: u64) {
        unsafe {
            asm!("msr CNTVOFF_EL2, {}", in(reg) offset);
        }
    }
    
    pub fn init_virtual_timer(&mut self) -> Result<(), &'static str> {
        // Disable virtual timer first
        self.disable_virtual_timer();
        
        // Clear any pending interrupts
        self.clear_virtual_timer_interrupt();
        
        println!("Virtual timer initialized");
        Ok(())
    }
    
    pub fn init_physical_timer(&mut self) -> Result<(), &'static str> {
        // Disable physical timer first
        self.disable_physical_timer();
        
        // Clear any pending interrupts
        self.clear_physical_timer_interrupt();
        
        println!("Physical timer initialized");
        Ok(())
    }
    
    pub fn set_virtual_timer(&self, ticks: u64, enable: bool) {
        unsafe {
            // Set compare value
            asm!("msr CNTV_CVAL_EL0, {}", in(reg) ticks);
            
            // Enable/disable timer
            let ctl = if enable { 1u64 } else { 0u64 };
            asm!("msr CNTV_CTL_EL0, {}", in(reg) ctl);
        }
    }
    
    pub fn set_physical_timer(&self, ticks: u64, enable: bool) {
        unsafe {
            // Set compare value
            asm!("msr CNTP_CVAL_EL0, {}", in(reg) ticks);
            
            // Enable/disable timer
            let ctl = if enable { 1u64 } else { 0u64 };
            asm!("msr CNTP_CTL_EL0, {}", in(reg) ctl);
        }
    }
    
    pub fn set_virtual_timer_tval(&self, ticks: u32, enable: bool) {
        unsafe {
            // Set timer value (countdown)
            asm!("msr CNTV_TVAL_EL0, {}", in(reg) ticks);
            
            // Enable/disable timer
            let ctl = if enable { 1u64 } else { 0u64 };
            asm!("msr CNTV_CTL_EL0, {}", in(reg) ctl);
        }
    }
    
    pub fn set_physical_timer_tval(&self, ticks: u32, enable: bool) {
        unsafe {
            // Set timer value (countdown)
            asm!("msr CNTP_TVAL_EL0, {}", in(reg) ticks);
            
            // Enable/disable timer
            let ctl = if enable { 1u64 } else { 0u64 };
            asm!("msr CNTP_CTL_EL0, {}", in(reg) ctl);
        }
    }
    
    pub fn enable_virtual_timer(&self) {
        unsafe {
            asm!("msr CNTV_CTL_EL0, {}", in(reg) 1u64);
        }
    }
    
    pub fn disable_virtual_timer(&self) {
        unsafe {
            asm!("msr CNTV_CTL_EL0, {}", in(reg) 0u64);
        }
    }
    
    pub fn enable_physical_timer(&self) {
        unsafe {
            asm!("msr CNTP_CTL_EL0, {}", in(reg) 1u64);
        }
    }
    
    pub fn disable_physical_timer(&self) {
        unsafe {
            asm!("msr CNTP_CTL_EL0, {}", in(reg) 0u64);
        }
    }
    
    pub fn is_virtual_timer_pending(&self) -> bool {
        let ctl: u64;
        unsafe {
            asm!("mrs {}, CNTV_CTL_EL0", out(reg) ctl);
        }
        (ctl & 0x4) != 0  // ISTATUS bit
    }
    
    pub fn is_physical_timer_pending(&self) -> bool {
        let ctl: u64;
        unsafe {
            asm!("mrs {}, CNTP_CTL_EL0", out(reg) ctl);
        }
        (ctl & 0x4) != 0  // ISTATUS bit
    }
    
    pub fn clear_virtual_timer_interrupt(&self) {
        // Interrupt is cleared by writing to CNTV_CTL_EL0 or updating CNTV_CVAL_EL0
        let current_count = self.read_virtual_counter();
        self.set_virtual_timer(current_count + self.frequency, false);
    }
    
    pub fn clear_physical_timer_interrupt(&self) {
        // Interrupt is cleared by writing to CNTP_CTL_EL0 or updating CNTP_CVAL_EL0
        let current_count = self.read_physical_counter();
        self.set_physical_timer(current_count + self.frequency, false);
    }
    
    pub fn get_virtual_timer_value(&self) -> u32 {
        let tval: u32;
        unsafe {
            asm!("mrs {}, CNTV_TVAL_EL0", out(reg) tval);
        }
        tval
    }
    
    pub fn get_physical_timer_value(&self) -> u32 {
        let tval: u32;
        unsafe {
            asm!("mrs {}, CNTP_TVAL_EL0", out(reg) tval);
        }
        tval
    }
    
    pub fn get_virtual_compare_value(&self) -> u64 {
        let cval: u64;
        unsafe {
            asm!("mrs {}, CNTV_CVAL_EL0", out(reg) cval);
        }
        cval
    }
    
    pub fn get_physical_compare_value(&self) -> u64 {
        let cval: u64;
        unsafe {
            asm!("mrs {}, CNTP_CVAL_EL0", out(reg) cval);
        }
        cval
    }
    
    pub fn schedule_timeout_ms(&self, milliseconds: u64, timer_type: TimerType) {
        let ticks = (milliseconds * self.frequency) / 1000;
        let current_count = match timer_type {
            TimerType::Virtual => self.read_virtual_counter(),
            TimerType::Physical => self.read_physical_counter(),
            _ => self.read_virtual_counter(),
        };
        
        let target_count = current_count + ticks;
        
        match timer_type {
            TimerType::Virtual => self.set_virtual_timer(target_count, true),
            TimerType::Physical => self.set_physical_timer(target_count, true),
            _ => self.set_virtual_timer(target_count, true),
        }
    }
    
    pub fn schedule_timeout_us(&self, microseconds: u64, timer_type: TimerType) {
        let ticks = (microseconds * self.frequency) / 1_000_000;
        let current_count = match timer_type {
            TimerType::Virtual => self.read_virtual_counter(),
            TimerType::Physical => self.read_physical_counter(),
            _ => self.read_virtual_counter(),
        };
        
        let target_count = current_count + ticks;
        
        match timer_type {
            TimerType::Virtual => self.set_virtual_timer(target_count, true),
            TimerType::Physical => self.set_physical_timer(target_count, true),
            _ => self.set_virtual_timer(target_count, true),
        }
    }
    
    pub fn delay_ms(&self, milliseconds: u64) {
        let start_count = self.read_virtual_counter();
        let delay_ticks = (milliseconds * self.frequency) / 1000;
        let end_count = start_count + delay_ticks;
        
        while self.read_virtual_counter() < end_count {
            // Busy wait
            core::hint::spin_loop();
        }
    }
    
    pub fn delay_us(&self, microseconds: u64) {
        let start_count = self.read_virtual_counter();
        let delay_ticks = (microseconds * self.frequency) / 1_000_000;
        let end_count = start_count + delay_ticks;
        
        while self.read_virtual_counter() < end_count {
            // Busy wait
            core::hint::spin_loop();
        }
    }
    
    pub fn get_uptime_ms(&self) -> u64 {
        let current_count = self.read_virtual_counter();
        (current_count * 1000) / self.frequency
    }
    
    pub fn get_uptime_us(&self) -> u64 {
        let current_count = self.read_virtual_counter();
        (current_count * 1_000_000) / self.frequency
    }
    
    pub fn get_uptime_ns(&self) -> u64 {
        let current_count = self.read_virtual_counter();
        (current_count * 1_000_000_000) / self.frequency
    }
    
    pub fn ticks_to_ms(&self, ticks: u64) -> u64 {
        (ticks * 1000) / self.frequency
    }
    
    pub fn ticks_to_us(&self, ticks: u64) -> u64 {
        (ticks * 1_000_000) / self.frequency
    }
    
    pub fn ticks_to_ns(&self, ticks: u64) -> u64 {
        (ticks * 1_000_000_000) / self.frequency
    }
    
    pub fn ms_to_ticks(&self, ms: u64) -> u64 {
        (ms * self.frequency) / 1000
    }
    
    pub fn us_to_ticks(&self, us: u64) -> u64 {
        (us * self.frequency) / 1_000_000
    }
    
    pub fn ns_to_ticks(&self, ns: u64) -> u64 {
        (ns * self.frequency) / 1_000_000_000
    }
    
    pub fn get_frequency(&self) -> u64 {
        self.frequency
    }
    
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// High-resolution timer for precise measurements
pub struct HighResTimer {
    start_count: u64,
    frequency: u64,
}

impl HighResTimer {
    pub fn new() -> Self {
        let timer = get_timer();
        HighResTimer {
            start_count: timer.read_virtual_counter(),
            frequency: timer.get_frequency(),
        }
    }
    
    pub fn start(&mut self) {
        let timer = get_timer();
        self.start_count = timer.read_virtual_counter();
    }
    
    pub fn elapsed_ns(&self) -> u64 {
        let timer = get_timer();
        let current_count = timer.read_virtual_counter();
        let elapsed_ticks = current_count - self.start_count;
        timer.ticks_to_ns(elapsed_ticks)
    }
    
    pub fn elapsed_us(&self) -> u64 {
        let timer = get_timer();
        let current_count = timer.read_virtual_counter();
        let elapsed_ticks = current_count - self.start_count;
        timer.ticks_to_us(elapsed_ticks)
    }
    
    pub fn elapsed_ms(&self) -> u64 {
        let timer = get_timer();
        let current_count = timer.read_virtual_counter();
        let elapsed_ticks = current_count - self.start_count;
        timer.ticks_to_ms(elapsed_ticks)
    }
}

/// Periodic timer for regular intervals
pub struct PeriodicTimer {
    interval_ticks: u64,
    next_trigger: u64,
    timer_type: TimerType,
}

impl PeriodicTimer {
    pub fn new(interval_ms: u64, timer_type: TimerType) -> Self {
        let timer = get_timer();
        let interval_ticks = timer.ms_to_ticks(interval_ms);
        let current_count = match timer_type {
            TimerType::Virtual => timer.read_virtual_counter(),
            TimerType::Physical => timer.read_physical_counter(),
            _ => timer.read_virtual_counter(),
        };
        
        PeriodicTimer {
            interval_ticks,
            next_trigger: current_count + interval_ticks,
            timer_type,
        }
    }
    
    pub fn start(&mut self) {
        let timer = get_timer();
        match self.timer_type {
            TimerType::Virtual => timer.set_virtual_timer(self.next_trigger, true),
            TimerType::Physical => timer.set_physical_timer(self.next_trigger, true),
            _ => timer.set_virtual_timer(self.next_trigger, true),
        }
    }
    
    pub fn handle_interrupt(&mut self) {
        // Update next trigger time
        self.next_trigger += self.interval_ticks;
        
        // Schedule next interrupt
        let timer = get_timer();
        match self.timer_type {
            TimerType::Virtual => timer.set_virtual_timer(self.next_trigger, true),
            TimerType::Physical => timer.set_physical_timer(self.next_trigger, true),
            _ => timer.set_virtual_timer(self.next_trigger, true),
        }
    }
    
    pub fn stop(&self) {
        let timer = get_timer();
        match self.timer_type {
            TimerType::Virtual => timer.disable_virtual_timer(),
            TimerType::Physical => timer.disable_physical_timer(),
            _ => timer.disable_virtual_timer(),
        }
    }
}

/// Watchdog timer functionality
pub struct WatchdogTimer {
    timeout_ticks: u64,
    enabled: bool,
}

impl WatchdogTimer {
    pub fn new(timeout_ms: u64) -> Self {
        let timer = get_timer();
        let timeout_ticks = timer.ms_to_ticks(timeout_ms);
        
        WatchdogTimer {
            timeout_ticks,
            enabled: false,
        }
    }
    
    pub fn start(&mut self) {
        let timer = get_timer();
        let current_count = timer.read_virtual_counter();
        let timeout_count = current_count + self.timeout_ticks;
        
        timer.set_virtual_timer(timeout_count, true);
        self.enabled = true;
    }
    
    pub fn kick(&mut self) {
        if self.enabled {
            let timer = get_timer();
            let current_count = timer.read_virtual_counter();
            let timeout_count = current_count + self.timeout_ticks;
            
            timer.set_virtual_timer(timeout_count, true);
        }
    }
    
    pub fn stop(&mut self) {
        let timer = get_timer();
        timer.disable_virtual_timer();
        self.enabled = false;
    }
    
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// Timer interrupt handler
pub fn handle_timer_interrupt() {
    let timer = get_timer();
    
    // Check which timer triggered the interrupt
    if timer.is_virtual_timer_pending() {
        handle_virtual_timer_interrupt();
    }
    
    if timer.is_physical_timer_pending() {
        handle_physical_timer_interrupt();
    }
}

fn handle_virtual_timer_interrupt() {
    let timer = get_timer();
    
    // Clear the interrupt
    timer.clear_virtual_timer_interrupt();
    
    // Handle timer tick (would call scheduler, update system time, etc.)
    crate::task::scheduler::timer_tick();
}

fn handle_physical_timer_interrupt() {
    let timer = get_timer();
    
    // Clear the interrupt
    timer.clear_physical_timer_interrupt();
    
    // Handle physical timer interrupt
    println!("Physical timer interrupt");
}

/// Initialize timer subsystem
pub fn init() -> Result<(), &'static str> {
    println!("Initializing AArch64 Generic Timer...");
    
    let mut timer = GenericTimer::new();
    timer.init()?;
    
    unsafe {
        GENERIC_TIMER = Some(timer);
    }
    
    // Enable timer interrupt in GIC
    super::gic::enable_interrupt(27); // Virtual timer interrupt
    super::gic::enable_interrupt(30); // Physical timer interrupt
    
    println!("AArch64 Generic Timer initialized successfully");
    Ok(())
}

/// Get global timer instance
pub fn get_timer() -> &'static GenericTimer {
    unsafe {
        GENERIC_TIMER.as_ref().expect("Timer not initialized")
    }
}

/// Get current system time in milliseconds
pub fn get_system_time_ms() -> u64 {
    get_timer().get_uptime_ms()
}

/// Get current system time in microseconds
pub fn get_system_time_us() -> u64 {
    get_timer().get_uptime_us()
}

/// Get current system time in nanoseconds
pub fn get_system_time_ns() -> u64 {
    get_timer().get_uptime_ns()
}

/// Sleep for specified milliseconds
pub fn sleep_ms(milliseconds: u64) {
    get_timer().delay_ms(milliseconds);
}

/// Sleep for specified microseconds
pub fn sleep_us(microseconds: u64) {
    get_timer().delay_us(microseconds);
}

/// Schedule a one-shot timer
pub fn schedule_timer_ms(milliseconds: u64, timer_type: TimerType) {
    get_timer().schedule_timeout_ms(milliseconds, timer_type);
}

/// Schedule a one-shot timer in microseconds
pub fn schedule_timer_us(microseconds: u64, timer_type: TimerType) {
    get_timer().schedule_timeout_us(microseconds, timer_type);
}

/// Timer calibration for accurate timing
pub fn calibrate_timer() -> Result<(), &'static str> {
    let timer = get_timer();
    
    // Perform calibration by measuring against a known reference
    // This is a simplified version - real calibration would be more complex
    
    let start_count = timer.read_virtual_counter();
    
    // Wait for a known period (e.g., using a delay loop)
    for _ in 0..1000000 {
        core::hint::spin_loop();
    }
    
    let end_count = timer.read_virtual_counter();
    let measured_ticks = end_count - start_count;
    
    println!("Timer calibration: {} ticks for reference period", measured_ticks);
    
    Ok(())
}

/// Performance counter access
pub fn read_performance_counter() -> u64 {
    let count: u64;
    unsafe {
        asm!("mrs {}, PMCCNTR_EL0", out(reg) count);
    }
    count
}

pub fn enable_performance_counters() {
    unsafe {
        // Enable user access to performance counters
        asm!("msr PMUSERENR_EL0, {}", in(reg) 0x0Fu64);
        
        // Enable cycle counter
        asm!("msr PMCNTENSET_EL0, {}", in(reg) 0x80000000u64);
        
        // Enable performance monitoring
        asm!("msr PMCR_EL0, {}", in(reg) 0x1u64);
    }
}

/// Timer statistics and debugging
pub struct TimerStats {
    pub virtual_timer_interrupts: u64,
    pub physical_timer_interrupts: u64,
    pub timer_overruns: u64,
    pub max_interrupt_latency_ns: u64,
    pub avg_interrupt_latency_ns: u64,
}

static mut TIMER_STATS: TimerStats = TimerStats {
    virtual_timer_interrupts: 0,
    physical_timer_interrupts: 0,
    timer_overruns: 0,
    max_interrupt_latency_ns: 0,
    avg_interrupt_latency_ns: 0,
};

pub fn get_timer_stats() -> &'static TimerStats {
    unsafe { &TIMER_STATS }
}

pub fn reset_timer_stats() {
    unsafe {
        TIMER_STATS = TimerStats {
            virtual_timer_interrupts: 0,
            physical_timer_interrupts: 0,
            timer_overruns: 0,
            max_interrupt_latency_ns: 0,
            avg_interrupt_latency_ns: 0,
        };
    }
}