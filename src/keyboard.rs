use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use spin::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
        Mutex::new(Keyboard::new(ScancodeSet1::new(), layouts::Us104Key, HandleControl::Ignore));
}

pub fn handle_keyboard_interrupt() {
    use x86_64::instructions::port::Port;

    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };

    crate::task::keyboard::add_scancode(scancode);
}

pub fn process_scancode(scancode: u8) -> Option<DecodedKey> {
    let mut keyboard = KEYBOARD.lock();
    
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        keyboard.process_keyevent(key_event)
    } else {
        None
    }
}

pub struct KeyboardManager {
    buffer: [u8; 256],
    read_pos: usize,
    write_pos: usize,
}

impl KeyboardManager {
    pub fn new() -> Self {
        KeyboardManager {
            buffer: [0; 256],
            read_pos: 0,
            write_pos: 0,
        }
    }

    pub fn add_key(&mut self, key: u8) {
        let next_write = (self.write_pos + 1) % self.buffer.len();
        if next_write != self.read_pos {
            self.buffer[self.write_pos] = key;
            self.write_pos = next_write;
        }
    }

    pub fn read_key(&mut self) -> Option<u8> {
        if self.read_pos == self.write_pos {
            None
        } else {
            let key = self.buffer[self.read_pos];
            self.read_pos = (self.read_pos + 1) % self.buffer.len();
            Some(key)
        }
    }

    pub fn has_key(&self) -> bool {
        self.read_pos != self.write_pos
    }
}