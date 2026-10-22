//! Window-menu document switching when Preferences › User Interface › Open Documents As Tabs is off.

use serde_json::json;

use crate::VectorcraftApp;
use crate::menus::Item;

/// Items for Window when the tab strip is hidden and more than one document is open.
pub fn menu_items(app: &VectorcraftApp) -> Vec<Item> {
    if app.session.prefs.open_documents_as_tabs || app.session.documents().len() <= 1 {
        return Vec::new();
    }
    app.session
        .documents()
        .iter()
        .enumerate()
        .map(|(i, d)| {
            let title = format!("{}{}", d.title(), if d.is_dirty() { "*" } else { "" });
            Item::Cmd(crate::shortcut_editor::intern(&title), "document.activate", json!({ "index": i }))
        })
        .collect()
}

/// Insert [`menu_items`] after the Workspace submenu in the static Window menu.
pub fn inject_window_menu(app: &VectorcraftApp, base: Vec<Item>) -> Vec<Item> {
    let extra = menu_items(app);
    if extra.is_empty() {
        return base;
    }
    let mut out = base;
    let at = (after_workspace_index() + 1).min(out.len());
    let mut inject = vec![Item::Sep];
    inject.extend(extra);
    out.splice(at..at, inject);
    out
}

fn after_workspace_index() -> usize {
    3
}
