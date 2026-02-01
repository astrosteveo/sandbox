// SPDX-FileCopyrightText: 2026 the Sandbox contributors
// SPDX-License-Identifier: GPL-3.0-or-later

//! Editor UI modules.

pub mod animation_editor;
pub mod asset_browser;
pub mod file_menu;
pub mod hierarchy;
pub mod inspector;

pub use animation_editor::{animation_editor_window, AnimationEditorState};
pub use asset_browser::asset_browser_panel;
pub use file_menu::{menu_bar, status_messages};
pub use hierarchy::*;
pub use inspector::*;
