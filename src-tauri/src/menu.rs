//! The macOS menu bar, in place of the default one Tauri installs.
//!
//! A Mac app is expected to answer from its menus: Settings… under the app
//! name, View and Go menus for what it does, and every key binding listed
//! beside its item — which is how Mac users find shortcuts at all. Windows and
//! Linux get none of this. Setting a menu there adds a menu bar across the top
//! of the window, over a toolbar that already does the job.
//!
//! The app's own items carry no logic. Each one forwards its id to the webview
//! as a `menu` event, and `+layout.svelte` runs it through the same code its key
//! bindings use. The predefined ones (Copy, Hide, Enter Full Screen, …) are
//! handled by AppKit itself.

use tauri::menu::{
    AboutMetadata, Menu, MenuEvent, MenuItem, MenuItemBuilder, SubmenuBuilder, WINDOW_SUBMENU_ID,
};
use tauri::{AppHandle, Emitter, Runtime};

pub fn build<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    let item = |id: &str, text: &str, accelerator: Option<&str>| -> tauri::Result<MenuItem<R>> {
        let mut builder = MenuItemBuilder::with_id(id, text);
        if let Some(accelerator) = accelerator {
            builder = builder.accelerator(accelerator);
        }
        builder.build(app)
    };

    let package = app.package_info();
    let about = AboutMetadata {
        name: Some(package.name.clone()),
        version: Some(package.version.to_string()),
        ..Default::default()
    };

    let app_menu = SubmenuBuilder::new(app, &package.name)
        .about(Some(about))
        .separator()
        .item(&item("settings", "Settings…", Some("Cmd+,"))?)
        .separator()
        .services()
        .separator()
        .hide()
        .hide_others()
        .show_all()
        .separator()
        .quit()
        .build()?;

    // Without Undo, Cut, Copy and Paste here, their keys do nothing in a text
    // field: on macOS it is the menu that carries them, not the webview.
    let edit_menu = SubmenuBuilder::new(app, "Edit")
        .undo()
        .redo()
        .separator()
        .cut()
        .copy()
        .paste()
        .select_all()
        .separator()
        .item(&item("find", "Find…", Some("Cmd+F"))?)
        .build()?;

    // Bare-letter bindings (m, t, f, n, p, r) stay off the menu: as key
    // equivalents they would fire while typing in the search box.
    let view_menu = SubmenuBuilder::new(app, "View")
        .item(&item(
            "toggle-sidebar",
            "Toggle Sidebar",
            Some("Ctrl+Cmd+S"),
        )?)
        .separator()
        .item(&item("reading-mode", "Toggle Mushaf Page View", None)?)
        .item(&item("verse-cards", "Toggle Verse Cards", None)?)
        .item(&item("focus-mode", "Toggle Focus Mode", None)?)
        .separator()
        .item(&item("zoom-in", "Zoom In", Some("Cmd+="))?)
        .item(&item("zoom-out", "Zoom Out", Some("Cmd+-"))?)
        .item(&item("zoom-reset", "Actual Size", Some("Cmd+0"))?)
        .separator()
        .fullscreen()
        .build()?;

    let go_menu = SubmenuBuilder::new(app, "Go")
        .item(&item("go-to", "Go To…", Some("Cmd+K"))?)
        .separator()
        .item(&item("next", "Next", None)?)
        .item(&item("previous", "Previous", None)?)
        .separator()
        .item(&item("routines", "Routines…", None)?)
        .build()?;

    // The id makes AppKit treat it as the Window menu, and list open windows in
    // it.
    let window_menu = SubmenuBuilder::with_id(app, WINDOW_SUBMENU_ID, "Window")
        .minimize()
        .maximize()
        .separator()
        .close_window()
        .build()?;

    Menu::with_items(
        app,
        &[&app_menu, &edit_menu, &view_menu, &go_menu, &window_menu],
    )
}

/// Hand a menu item's id to the webview.
pub fn forward<R: Runtime>(app: &AppHandle<R>, event: MenuEvent) {
    if let Err(err) = app.emit("menu", event.id().as_ref()) {
        log::error!("Failed to forward menu item {:?}: {err}", event.id());
    }
}
