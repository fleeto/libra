//! `libra ui` — open the desktop window.
//!
//! The window itself runs on the process main thread (`gui::run`, feature
//! `gui`). This module is the clap surface and the refusal used when that
//! path is not taken: a build without the feature, or a library embedder
//! that reached the dispatcher instead of the process entry point.

use clap::Parser;

#[cfg(not(feature = "gui"))]
use crate::utils::error::StableErrorCode;
use crate::utils::error::{CliError, CliResult};

pub const UI_EXAMPLES: &str = "\
EXAMPLES:
    libra ui                               Open the desktop window
    cargo build --features gui && libra ui  Build the window into this binary, then open it";

/// Open the Libra desktop window.
#[derive(Parser, Debug)]
#[command(after_help = UI_EXAMPLES)]
pub struct UiArgs {}

/// Why `libra ui` did not open a window from the dispatcher.
///
/// The process entry point handles a bare `libra ui` before this runs.
/// Reaching here means either this binary was built without `gui`, or the
/// caller is not the process entry point (for example `libra::exec`).
pub fn unavailable_error() -> CliError {
    #[cfg(feature = "gui")]
    {
        CliError::fatal(
            "`libra ui` opens its window on the process main thread. Run the `libra` executable directly (`libra ui`). The library dispatcher cannot open it.",
        )
    }
    #[cfg(not(feature = "gui"))]
    {
        CliError::fatal(
            "this build of libra has no desktop UI. Rebuild with `cargo build --features gui`, then run `libra ui`.",
        )
        .with_stable_code(StableErrorCode::Unsupported)
    }
}

/// Dispatcher entry. Never opens a window; see [`unavailable_error`].
pub fn execute(_args: UiArgs) -> CliResult<()> {
    Err(unavailable_error())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unavailable_error_explains_the_next_step() {
        let error = unavailable_error();
        let message = error.render();
        if cfg!(feature = "gui") {
            assert!(
                message.contains("main thread"),
                "embedded GUI build should say why the dispatcher cannot open a window, got: {message}"
            );
        } else {
            assert!(
                message.contains("--features gui"),
                "feature-off build should name the rebuild, got: {message}"
            );
            assert_eq!(error.stable_code().as_str(), "LBR-UNSUPPORTED-001");
        }
    }
}
