//! User-facing diagnostics that complement error reporting,
//! such as the bug-report banner shown after a failure.

use colored::Colorize;

/// Display bug report information to the user on stderr.
pub fn display_bug_report_info() {
    eprintln!();
    eprintln!("{}", "━".repeat(60).bright_yellow());
    eprintln!("{}", "💡 Need help or found a bug?".bright_yellow().bold());
    eprintln!();
    eprintln!(
        "{}",
        "  Please report issues, request features, or get support at:".yellow()
    );
    eprintln!(
        "{}",
        "  🔗 https://github.com/MathieuSoysal/CG-Bundler/issues/new"
            .blue()
            .bold()
    );
    eprintln!();
    eprintln!(
        "{}",
        "  Your feedback helps improve CG-Bundler for everyone!".yellow()
    );
    eprintln!("{}", "━".repeat(60).bright_yellow());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_bug_report_info_does_not_panic() {
        display_bug_report_info();
    }
}
