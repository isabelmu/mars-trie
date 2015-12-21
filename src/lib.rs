#[macro_use] extern crate log;
#[macro_use] extern crate fallthrough;
extern crate quickcheck;
extern crate rand;

mod base;
mod config;
mod error;
mod iter_util;
mod trie;
mod vector;

#[cfg(test)]
extern crate env_logger;

