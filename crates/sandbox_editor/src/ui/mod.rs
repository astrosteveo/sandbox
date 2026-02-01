// SPDX-FileCopyrightText: 2026 the Sandbox contributors
// SPDX-License-Identifier: GPL-3.0-or-later

//! Editor UI modules.

pub mod file_menu;
pub mod hierarchy;
pub mod inspector;

pub use file_menu::{menu_bar, status_messages};
pub use hierarchy::*;
pub use inspector::*;
