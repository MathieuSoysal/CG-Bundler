//! `cg-bundler` binary entry point.
//!
//! Responsibilities are intentionally narrow: parse CLI arguments, dispatch
//! to the appropriate command handler, and translate any returned error
//! into a user-facing message plus a non-zero exit code.

mod cli;
mod commands;
mod diagnostics;
mod minify;
mod rustfmt_fmt;

#[cfg(test)]
mod test_support;

use clap::Parser;
use std::process;

use cli::Cli;

fn main() {
    let cli = Cli::parse();

    if let Err(e) = commands::dispatch(&cli) {
        diagnostics::report(&e);
        process::exit(1);
    }
}
