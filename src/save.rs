//! Saving and loading.
//!
//! There's not much to keep: the best distance, the grape count and the
//! volume. On desktop it's a small JSON file in the usual per-user data
//! folder. In the browser the same JSON goes into localStorage.
//!
//! Anything that wants a save sends `SaveRequested`; the actual write
//! happens once at the end of the frame however many asked for one.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::run::Progress;

pub fn plugin(app: &mut App) {
    let data = storage::load().unwrap_or_default();
    app.add_message::<SaveRequested>()
        .insert_resource(Settings {
            volume: data.music_volume.clamp(0.0, 1.0),
            fullscreen: false,
        })
        .insert_resource(Progress {
            best: data.high_score,
            grapes: data.grapes,
            ..default()
        })
        .add_systems(Last, write_save.run_if(on_message::<SaveRequested>));
}

#[derive(Message)]
pub struct SaveRequested;

/// Player-tweakable settings. Only the volume is saved; starting up
/// fullscreen without being asked to tends to annoy people.
#[derive(Resource)]
pub struct Settings {
    /// 0 to 1, straight from the slider.
    pub volume: f32,
    pub fullscreen: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
struct SaveData {
    high_score: u32,
    grapes: u32,
    music_volume: f32,
}

impl Default for SaveData {
    fn default() -> Self {
        SaveData {
            high_score: 0,
            grapes: 0,
            music_volume: 0.25,
        }
    }
}

fn write_save(progress: Res<Progress>, settings: Res<Settings>) {
    let data = SaveData {
        high_score: progress.best,
        grapes: progress.grapes,
        music_volume: settings.volume,
    };
    match serde_json::to_string_pretty(&data) {
        Ok(json) => storage::store(&json),
        Err(err) => warn!("couldn't serialise the save: {err}"),
    }
}

fn parse(json: &str) -> Option<SaveData> {
    serde_json::from_str(json)
        .inspect_err(|err| warn!("ignoring a broken save file: {err}"))
        .ok()
}

#[cfg(not(target_arch = "wasm32"))]
mod storage {
    use std::fs;
    use std::path::PathBuf;

    use bevy::log::{info, warn};

    /// ~/.local/share/flyaway on Linux, Application Support on macOS and
    /// %APPDATA% on Windows. Falls back to the working directory if none of
    /// those are set, which mostly only happens in odd CI setups.
    fn path() -> PathBuf {
        let base = if cfg!(windows) {
            std::env::var_os("APPDATA").map(PathBuf::from)
        } else if cfg!(target_os = "macos") {
            std::env::var_os("HOME")
                .map(|home| PathBuf::from(home).join("Library/Application Support"))
        } else {
            std::env::var_os("XDG_DATA_HOME")
                .map(PathBuf::from)
                .or_else(|| {
                    std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share"))
                })
        };
        base.unwrap_or_default().join("flyaway").join("save.json")
    }

    pub fn load() -> Option<super::SaveData> {
        // A missing file just means a first run, so that's not worth a warning.
        let json = fs::read_to_string(path()).ok()?;
        super::parse(&json)
    }

    pub fn store(json: &str) {
        let path = path();
        let result = path
            .parent()
            .map_or(Ok(()), fs::create_dir_all)
            .and_then(|_| fs::write(&path, json));
        match result {
            Ok(()) => info!("saved to {}", path.display()),
            Err(err) => warn!("couldn't save to {}: {err}", path.display()),
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod storage {
    use bevy::log::warn;

    const KEY: &str = "flyaway.save";

    // localStorage can be missing entirely (some privacy modes) or throw on
    // write (quota, disabled storage). Either way the game should carry on;
    // it just won't remember anything.
    fn local_storage() -> Option<web_sys::Storage> {
        web_sys::window()?.local_storage().ok().flatten()
    }

    pub fn load() -> Option<super::SaveData> {
        let json = local_storage()?.get_item(KEY).ok().flatten()?;
        super::parse(&json)
    }

    pub fn store(json: &str) {
        let saved = local_storage().is_some_and(|storage| storage.set_item(KEY, json).is_ok());
        if !saved {
            warn!("couldn't save to localStorage");
        }
    }
}
