// Copyright (c) 2010-2013, Susumu Yata
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:
//
// - Redistributions of source code must retain the above copyright notice, this
//   list of conditions and the following disclaimer.
// - Redistributions in binary form must reproduce the above copyright notice,
//   this list of conditions and the following disclaimer in the documentation
//   and/or other materials provided with the distribution.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
// AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
// IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
// ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE
// LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
// CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
// SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
// INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
// CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
// ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
// POSSIBILITY OF SUCH DAMAGE.

/// Min/max values, flags and masks for dictionary settings are defined below.
/// Please note that unspecified settings will be replaced with the default
/// settings. For example, 0 is equivalent to (NumTries::default() |
/// CacheLevel::default() | TailMode::default() | NodeOrder::default()).

/// A dictionary consists of 3 tries by default. Usually more tries make a
/// dictionary space-efficient but time-inefficient.
#[derive(Copy, Clone)]
pub struct NumTries { num_: u32 }
pub const MIN_NUM_TRIES: u32 = 0x00001;
pub const MAX_NUM_TRIES: u32 = 0x0007F;
impl NumTries {
    fn new(num: u32) -> NumTries {
        assert!(num >= MIN_NUM_TRIES && num <= MAX_NUM_TRIES);
        NumTries { num_: num }
    }
    fn get(&self) -> u32 {
        self.num_
    }
}
impl Default for NumTries {
    fn default() -> NumTries {
        NumTries::new(3)
    }
}

/// This library uses a cache technique to accelerate search functions. The
/// following enumerated type gives a list of available cache size options. A
/// larger cache enables faster search but takes a more space.
#[derive(Copy, Clone)]
pub enum CacheLevel {
    Huge   = 0x00080,
    Large  = 0x00100,
    Normal = 0x00200,
    Small  = 0x00400,
    Tiny   = 0x00800,
}
impl Default for CacheLevel {
    fn default() -> CacheLevel {
        CacheLevel::Normal
    }
}

/// This library provides 2 kinds of TAIL implementations.
#[derive(Copy, Clone)]
pub enum TailMode {
    /// Merge last labels as zero-terminated strings. Available if and only if
    /// last labels do not contain a null character.
    ///
    /// If TailMode::Text is specified and a null character exists in the last
    /// labels, the setting is automatically switched to TailMode::Binary.
    Text = 0x01000,

    /// TailMode::Binary also merges last labels but as byte sequences. It uses
    /// a bit vector to detect the end of a sequence, instead of null
    /// characters. So, TailMode::Binary requires a larger space if the average
    /// length of labels is greater than 8.
    Binary = 0x02000,
}
impl Default for TailMode {
    fn default() -> TailMode {
        TailMode::Text
    }
}

/// The arrangement of nodes affects the time cost of matching and the order of
/// predictive search.
#[derive(Copy, Clone)]
pub enum NodeOrder {
    /// Arrange nodes in ascending label order. Useful if an application needs
    /// to predict keys in label order.
    Label = 0x10000,
  
    /// Arrange nodes in descending weight order. Generally the better choice,
    /// because it enables faster matching.
    Weight = 0x20000,
}
impl Default for NodeOrder {
    fn default() -> NodeOrder {
        NodeOrder::Weight
    }
}

/// Config masks
const NUM_TRIES_MASK   : u32 = 0x0007F;
const CACHE_LEVEL_MASK : u32 = 0x00F80;
const TAIL_MODE_MASK   : u32 = 0x0F000;
const NODE_ORDER_MASK  : u32 = 0xF0000;
const CONFIG_MASK      : u32 = 0xFFFFF;

#[derive(Copy, Clone)]
pub struct Config {
    num_tries_: NumTries,
    cache_level_: CacheLevel,
    tail_mode_: TailMode,
    node_order_: NodeOrder,
}

impl Config {
    pub fn new() -> Config {
        Config {
            num_tries_: Default::default(),
            cache_level_: Default::default(),
            tail_mode_: Default::default(),
            node_order_: Default::default(),
        }
    }

    pub fn parse(config_flags: u32) -> Config {
        assert!((config_flags & !CONFIG_MASK) == 0, "MARISA_CODE_ERROR");

        let mut out = Config::new();
        out.parse_num_tries(config_flags);
        out.parse_cache_level(config_flags);
        out.parse_tail_mode(config_flags);
        out.parse_node_order(config_flags);
        out
    }

    pub fn flags(&self) -> u32 {
        self.num_tries_.get()
        | (self.tail_mode_ as u32)
        | (self.node_order_ as u32)
    }

    pub fn num_tries(&self) -> NumTries {
        self.num_tries_
    }
    pub fn cache_level(&self) -> CacheLevel {
        self.cache_level_
    }
    pub fn tail_mode(&self) -> TailMode {
        self.tail_mode_
    }
    pub fn node_order(&self) -> NodeOrder {
        self.node_order_
    }

    pub fn clear(&mut self) {
        *self = Config::new();
    }

    fn parse_num_tries(&mut self, config_flags: u32) {
        let num_tries: u32 = config_flags & NUM_TRIES_MASK;
        if num_tries != 0 {
            self.num_tries_ = NumTries::new(num_tries);
        }
    }

    fn parse_cache_level(&mut self, config_flags: u32) {
        self.cache_level_ = match config_flags & CACHE_LEVEL_MASK {
            0 => Default::default(),
            x if x == CacheLevel::Huge as u32 => CacheLevel::Huge,
            x if x == CacheLevel::Large as u32 => CacheLevel::Large,
            x if x == CacheLevel::Normal as u32 => CacheLevel::Normal,
            x if x == CacheLevel::Small as u32 => CacheLevel::Small,
            x if x == CacheLevel::Tiny as u32 => CacheLevel::Tiny,
            _ => panic!("MARISA_CODE_ERROR: undefined cache level"),
        }
    }

    fn parse_tail_mode(&mut self, config_flags: u32) {
        self.tail_mode_ = match config_flags & TAIL_MODE_MASK {
            0 => Default::default(),
            x if x == TailMode::Text as u32 => TailMode::Text,
            x if x == TailMode::Binary as u32 => TailMode::Binary,
            _ => panic!("MARISA_CODE_ERROR: undefined tail mode"),
        }
    }

    fn parse_node_order(&mut self, config_flags: u32) {
        self.node_order_ = match config_flags & NODE_ORDER_MASK {
            0 => Default::default(),
            x if x == NodeOrder::Label as u32 => NodeOrder::Label,
            x if x == NodeOrder::Weight as u32 => NodeOrder::Weight,
            _ => panic!("MARISA_CODE_ERROR: undefined node order"),
        }
    }
}

