//! Browser builds retain internal Edit Contents without desktop editor sessions.

use serde_json::Value;

use crate::LienzoApp;

/// No external editor sessions exist in the browser.
#[derive(Default)]
pub(crate) struct State {}

impl State {
    /// The editor-choice dialog is never shown in the browser.
    pub(crate) fn modal_open(&self) -> bool {
        false
    }
}

/// Leave every command to normal dispatch, so Edit Contents uses the internal editor.
pub(crate) fn intercept(_app: &mut LienzoApp, _id: &str, _params: &Value) -> Option<Result<Value, String>> {
    None
}

/// No sessions to poll.
pub(crate) fn poll(_app: &mut LienzoApp, _ctx: &egui::Context) {}

/// No dialog to draw.
pub(crate) fn show(_app: &mut LienzoApp, _ctx: &egui::Context) {}
