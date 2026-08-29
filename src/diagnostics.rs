//! User-facing diagnostics that complement error reporting,
//! such as the bug-report banner shown after a failure.

use colored::Colorize;

use cg_bundler::BundlerError;

/// Report `error` on stderr, followed by the footer it warrants.
pub fn report(error: &BundlerError) {
    eprintln!("{} {error}", "Error:".red().bold());

    if is_user_error(error) {
        // The message already says what to change, so the loud "found a bug?"
        // banner would misattribute the problem. Keep the link reachable, but
        // frame it as feedback rather than a defect report.
        display_feedback_link();
        return;
    }

    display_bug_report_info();
}

/// Whether `error` describes something in the user's project rather than a
/// defect in the bundler.
const fn is_user_error(error: &BundlerError) -> bool {
    matches!(
        error,
        BundlerError::CargoMetadata { .. }
            | BundlerError::Parsing { .. }
            | BundlerError::ProjectStructure { .. }
            | BundlerError::MultipleBinaryTargets { .. }
            | BundlerError::NoBinaryTarget
            | BundlerError::MultipleLibraryTargets { .. }
    )
}

/// Point at the issue tracker without claiming the tool misbehaved.
pub fn display_feedback_link() {
    eprintln!();
    eprintln!(
        "{} {}",
        "  Questions or feedback:".cyan(),
        "https://github.com/MathieuSoysal/CG-Bundler/issues/new".blue()
    );
}

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

    /// A problem in the user's own project is not a bug in the bundler.
    #[test]
    fn project_problems_are_not_reported_as_bundler_bugs() {
        assert!(is_user_error(&BundlerError::NoBinaryTarget));
        assert!(is_user_error(&BundlerError::ProjectStructure {
            message: "is a workspace".to_string(),
        }));
        assert!(is_user_error(&BundlerError::Parsing {
            message: "expected an expression".to_string(),
            file_path: None,
        }));
        assert!(is_user_error(&BundlerError::CargoMetadata {
            message: "no manifest".to_string(),
            source: None,
        }));
    }

    /// An unexpected IO failure is worth reporting upstream.
    #[test]
    fn io_failures_still_invite_a_bug_report() {
        assert!(!is_user_error(&BundlerError::Io {
            source: std::io::Error::other("disk on fire"),
            path: None,
        }));
    }

    #[test]
    fn report_does_not_panic_for_either_class() {
        report(&BundlerError::NoBinaryTarget);
        report(&BundlerError::Io {
            source: std::io::Error::other("boom"),
            path: None,
        });
    }
}
