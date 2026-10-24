//! Desktop window for `libra ui`.
//!
//! GPUI owns the process main thread. [`run`] is called from `main` before
//! the CLI worker thread starts. It is not safe to call from `cli::parse`.

use std::sync::{Arc, Mutex};

use anyhow::Context;
mod model;
mod view;

use ely_gpui_component::forms::TextInput;
use gpui::{App, AppContext, Bounds, TitlebarOptions, WindowBounds, WindowOptions, px, size};
use view::SessionsWindow;

/// Open one window and run until the user closes it.
///
/// # Errors
///
/// Returns an error when Ely cannot load its fonts or the window fails to
/// open. The message names that step.
pub fn run() -> anyhow::Result<()> {
    let startup_error = Arc::new(Mutex::new(None));
    let reported = Arc::clone(&startup_error);
    gpui_platform::application()
        .with_assets(ely_gpui_component::Assets)
        .run(move |cx: &mut App| {
            if let Err(error) = open_window(cx) {
                match reported.lock() {
                    Ok(mut slot) => *slot = Some(error),
                    Err(poisoned) => *poisoned.into_inner() = Some(error),
                }
                cx.quit();
            }
        });
    match startup_error.lock() {
        Ok(mut slot) => slot.take().map_or(Ok(()), Err),
        Err(poisoned) => poisoned.into_inner().take().map_or(Ok(()), Err),
    }
}

fn open_window(cx: &mut App) -> anyhow::Result<()> {
    ely_gpui_component::init(cx).context("Ely failed to start")?;
    let bounds = Bounds::centered(None, size(px(1280.0), px(800.0)), cx);
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: Some(TitlebarOptions {
            title: Some("Libra Sessions".into()),
            ..Default::default()
        }),
        ..Default::default()
    };
    cx.open_window(options, |window, cx| {
        cx.new(|cx| {
            let focus = cx.focus_handle();
            window.focus(&focus, cx);
            let search = cx.new(|cx| {
                TextInput::new(window, cx)
                    .placeholder("Search redacted content and readable reasoning")
            });
            SessionsWindow::new(focus, search, cx)
        })
    })
    .context("failed to open the Libra window")?;
    cx.activate(true);
    Ok(())
}
