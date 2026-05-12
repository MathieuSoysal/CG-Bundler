//! Optional `rustfmt` integration used by the `--pretty` flag.
//!
//! The integration is best-effort: if `rustfmt` is not installed or the
//! input cannot be formatted, the caller falls back to the unformatted code.

use std::io::Write as _;
use std::process::{Command, Stdio};

/// Format `code` by piping it through `rustfmt`.
///
/// Returns `None` if `rustfmt` is not available, the spawn failed, or
/// formatting itself was unsuccessful. When `verbose` is `true`, `rustfmt`
/// stderr is forwarded to the user's terminal to aid debugging.
#[must_use]
pub fn format_with_rustfmt(code: &str, verbose: bool) -> Option<String> {
    let mut child = spawn_rustfmt(verbose)?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(code.as_bytes()).ok()?;
    }

    let output = child.wait_with_output().ok()?;

    if output.status.success() {
        String::from_utf8(output.stdout).ok()
    } else {
        None
    }
}

fn spawn_rustfmt(verbose: bool) -> Option<std::process::Child> {
    Command::new("rustfmt")
        .arg("--emit")
        .arg("stdout")
        .arg("--edition")
        .arg("2021")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(if verbose {
            Stdio::inherit()
        } else {
            Stdio::null()
        })
        .spawn()
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_with_rustfmt_valid_code() {
        let code = "fn main(){let x=5;println!(\"{}\",x);}";
        let result = format_with_rustfmt(code, false);
        if let Some(formatted) = result {
            assert!(formatted.contains("fn main"));
        }
    }

    #[test]
    fn test_format_with_rustfmt_invalid_code() {
        let code = "fn broken( { let";
        let result = format_with_rustfmt(code, false);
        assert!(result.is_none());
    }

    #[test]
    fn test_format_with_rustfmt_verbose() {
        let code = "fn main(){let x=5;println!(\"{}\",x);}";
        let _ = format_with_rustfmt(code, true);
    }
}
