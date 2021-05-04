#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::must_use_candidate, clippy::missing_panics_doc)]

pub mod cli;

pub use rustmatheval as lib;

fn main() -> ! {
    cli::main()
}
