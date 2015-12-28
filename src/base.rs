use std;

#[cfg(target_pointer_width = "32")]
pub const WORD_SIZE: usize = 32;

#[cfg(target_pointer_width = "64")]
pub const WORD_SIZE: usize = 64;

pub const INVALID_LINK_ID: u32 = std::u32::MAX;
pub const INVALID_KEY_ID: u32 = std::u32::MAX;

