//! Step definitions and the shared `GameWorld`. This is a subdirectory of
//! `tests/`, so Cargo does not compile it as its own test binary — only
//! `tests/cucumber.rs` pulls it in.

pub mod world;

mod common;
mod craft;
mod disassemble;
mod equipment_storage;
mod experiment;
mod hunting;
mod quests;
