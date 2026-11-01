use crate::nadk::keyboard::{
    Key, KeyboardState,
};

static NUM_KEYS: usize = 53;

pub struct Modifiers {
    pub shift_active: bool,
    pub alpha_active: bool,
}

pub struct Keyboard {
    // max wait time 255
    timers: [u8; NUM_KEYS],
    state : KeyboardState,
    old   : KeyboardState,
}

impl Keyboard {
    pub fn new() -> Self {
        Self {
            timers: [0; NUM_KEYS],
            state : KeyboardState::default(),
            old   : KeyboardState::default(),
        }
    }

    pub fn scan(&mut self, ms_dt: u32) {
        // scan the keyboard
        let mut new_state: u64 = 0;
        let new_scan = KeyboardState::scan();
        
        // for all keys
        for key in enum_iterator::all::<Key>() {
            if !new_scan.key_down(key) {
                self.timers[key as usize] = 0;
                continue;
            } 

            if self.timers[key as usize] > 0 {
                // decrement the timer
                self.timers[key as usize] = (self.timers[key as usize] as u32).saturating_sub(ms_dt) as u8;
            }
            else {
                // key down taken!
                new_state |= new_scan.0 & (1 << key as usize);
                
                // if key down
                if new_scan.get_just_pressed(self.old).key_down(key) {
                    self.timers[key as usize] = 250;
                }
                else {
                    self.timers[key as usize] = 100;
                }
            }
        }
        
        self.old   = new_scan;
        self.state = KeyboardState(new_state);
    }

    pub fn is_keydown(&self, key: Key) -> bool {
        self.state.key_down(key)
    }

    pub fn any_keydown(&self) -> bool {
        self.state.0 != 0
    }
}
