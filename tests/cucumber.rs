//! Functional (BDD) test suite — Gherkin scenarios run by cucumber-rs against
//! the `game` + `viewmodel` layers, never a front end. See
//! `docs/functional-tests.md` for the layout and conventions.

mod steps;

use cucumber::World;

use steps::world::GameWorld;

fn main() {
    // The game is fully synchronous; `pollster` is just a minimal `block_on`
    // so we don't pull in a full async runtime. `run_and_exit` turns a failed
    // scenario into a non-zero exit so `cargo test` and prek catch it.
    pollster::block_on(GameWorld::cucumber().run_and_exit("tests/features"));
}
