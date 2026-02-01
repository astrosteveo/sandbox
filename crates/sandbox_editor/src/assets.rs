// SPDX-FileCopyrightText: 2026 the Sandbox contributors
// SPDX-License-Identifier: GPL-3.0-or-later

//! Asset browser resource and directory scanning.

use bevy::prelude::*;
use std::path::PathBuf;

/// Represents a file or directory entry in the asset browser.
#[derive(Clone, Debug)]
pub struct AssetEntry {
    /// Display name of the entry.
    pub name: String,
    /// Full path relative to assets directory.
    pub path: String,
    /// Whether this entry is a directory.
    pub is_directory: bool,
    /// Child entries (if directory).
    pub children: Vec<AssetEntry>,
    /// Whether this directory is expanded in the UI.
    pub expanded: bool,
}

impl AssetEntry {
    /// Creates a new file entry.
    pub fn file(name: String, path: String) -> Self {
        Self {
            name,
            path,
            is_directory: false,
            children: Vec::new(),
            expanded: false,
        }
    }

    /// Creates a new directory entry.
    pub fn directory(name: String, path: String) -> Self {
        Self {
            name,
            path,
            is_directory: true,
            children: Vec::new(),
            expanded: false,
        }
    }
}

/// Type of asset file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetType {
    /// Image files (.png, .jpg, .jpeg)
    Image,
    /// Audio files (.ogg, .wav, .mp3)
    Audio,
    /// Scene files (.scn.ron)
    Scene,
    /// Unknown file type
    Unknown,
}

impl AssetType {
    /// Determines asset type from file extension.
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "png" | "jpg" | "jpeg" | "bmp" | "gif" => AssetType::Image,
            "ogg" | "wav" | "mp3" | "flac" => AssetType::Audio,
            "ron" => AssetType::Scene,
            _ => AssetType::Unknown,
        }
    }

    /// Returns icon character for this asset type.
    pub fn icon(&self) -> &'static str {
        match self {
            AssetType::Image => "🖼",
            AssetType::Audio => "🔊",
            AssetType::Scene => "📄",
            AssetType::Unknown => "📁",
        }
    }
}

/// Resource tracking the state of the asset browser.
#[derive(Resource, Default)]
pub struct AssetBrowser {
    /// Root entries of the asset tree.
    pub files: Vec<AssetEntry>,
    /// Currently selected file path.
    pub selected_path: Option<String>,
    /// Cached preview handles for images.
    pub preview_handles: std::collections::HashMap<String, Handle<Image>>,
    /// Whether the browser needs to be rescanned.
    pub needs_rescan: bool,
    /// Currently playing audio entity (if any).
    pub audio_playback_entity: Option<Entity>,
    /// Path of currently playing audio.
    pub playing_audio_path: Option<String>,
}

impl AssetBrowser {
    /// Creates a new asset browser and performs initial scan.
    pub fn new() -> Self {
        let mut browser = Self {
            files: Vec::new(),
            selected_path: None,
            preview_handles: std::collections::HashMap::new(),
            needs_rescan: false,
            audio_playback_entity: None,
            playing_audio_path: None,
        };
        browser.scan_assets_directory();
        browser
    }

    /// Scans the assets directory and populates the file tree.
    pub fn scan_assets_directory(&mut self) {
        self.files.clear();

        let assets_path = PathBuf::from("assets");
        if !assets_path.exists() {
            // Create assets directory if it doesn't exist
            let _ = std::fs::create_dir_all(&assets_path);
        }

        if let Ok(entries) = Self::scan_directory(&assets_path, "") {
            self.files = entries;
        }

        self.needs_rescan = false;
    }

    /// Recursively scans a directory.
    fn scan_directory(path: &PathBuf, relative_base: &str) -> std::io::Result<Vec<AssetEntry>> {
        let mut entries = Vec::new();

        let mut dir_entries: Vec<_> = std::fs::read_dir(path)?.filter_map(|e| e.ok()).collect();

        // Sort: directories first, then alphabetically
        dir_entries.sort_by(|a, b| {
            let a_is_dir = a.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
            let b_is_dir = b.file_type().map(|ft| ft.is_dir()).unwrap_or(false);

            match (a_is_dir, b_is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.file_name().cmp(&b.file_name()),
            }
        });

        for dir_entry in dir_entries {
            let file_name = dir_entry.file_name().to_string_lossy().to_string();

            // Skip hidden files and .gitkeep
            if file_name.starts_with('.') || file_name == ".gitkeep" {
                continue;
            }

            let relative_path = if relative_base.is_empty() {
                file_name.clone()
            } else {
                format!("{}/{}", relative_base, file_name)
            };

            if let Ok(file_type) = dir_entry.file_type() {
                if file_type.is_dir() {
                    let mut asset_entry = AssetEntry::directory(file_name, relative_path.clone());

                    // Recursively scan subdirectory
                    let full_path = dir_entry.path();
                    if let Ok(children) = Self::scan_directory(&full_path, &relative_path) {
                        asset_entry.children = children;
                    }

                    entries.push(asset_entry);
                } else if file_type.is_file() {
                    entries.push(AssetEntry::file(file_name, relative_path));
                }
            }
        }

        Ok(entries)
    }

    /// Gets the asset type for a file path.
    pub fn get_asset_type(path: &str) -> AssetType {
        path.rsplit('.')
            .next()
            .map(AssetType::from_extension)
            .unwrap_or(AssetType::Unknown)
    }

    /// Toggles expansion state of a directory at the given path.
    pub fn toggle_directory(&mut self, path: &str) {
        Self::toggle_directory_recursive(&mut self.files, path);
    }

    fn toggle_directory_recursive(entries: &mut [AssetEntry], path: &str) {
        for entry in entries.iter_mut() {
            if entry.path == path && entry.is_directory {
                entry.expanded = !entry.expanded;
                return;
            }
            if entry.is_directory && path.starts_with(&entry.path) {
                Self::toggle_directory_recursive(&mut entry.children, path);
            }
        }
    }

    /// Checks if a specific audio file is playing.
    pub fn is_playing(&self, path: &str) -> bool {
        self.playing_audio_path.as_ref() == Some(&path.to_string())
    }
}

/// Marker component for audio preview playback.
#[derive(Component)]
pub struct AudioPreviewMarker;

/// Plugin that sets up the asset browser.
pub struct AssetBrowserPlugin;

impl Plugin for AssetBrowserPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AssetBrowser::new());
    }
}
