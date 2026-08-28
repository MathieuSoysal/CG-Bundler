//! High-level command handlers for the `cg-bundler` binary.
//!
//! Each subcommand is implemented in its own focused module, keeping
//! orchestration logic out of `main.rs`.

pub mod bundle;
pub mod info;
pub mod validate;
pub mod watch;

use cg_bundler::BundlerError;

use crate::cli::Cli;

/// Dispatch to the correct command handler based on the parsed CLI flags.
///
/// # Errors
/// Propagates any [`BundlerError`] raised by the selected command.
pub fn dispatch(cli: &Cli) -> Result<(), BundlerError> {
    if cli.validate {
        validate::run(cli)
    } else if cli.info {
        info::run(&cli.get_project_path())
    } else if cli.watch {
        watch::run(cli)
    } else {
        bundle::run(cli)
    }
}
