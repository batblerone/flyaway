//! Fly Away, Fly!
//!
//! A crow, a forest, and a lot of things to fly into. Hold space (or the mouse
//! button, or a finger on a touch screen) to climb, let go to drop, and grab
//! grapes on the way past.
//!
//! The whole game is laid out for a 1280x720 screen with the origin in the
//! middle. The camera and UI scale from that, so the numbers you see in the
//! code are "screen pixels at 720p".

mod animation;
mod assets;
mod audio;
mod background;
mod collision;
mod display;
mod grapes;
mod hud;
mod menu;
mod obstacles;
mod player;
mod run;
mod save;

use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;

pub const VIEW_WIDTH: f32 = 1280.0;
pub const VIEW_HEIGHT: f32 = 720.0;

/// Where the game is at. The menus are tracked in their own state
/// (`menu::MenuScreen`) because the same menu shows up in several of these.
#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameState {
    /// The title screen.
    #[default]
    Title,
    /// Actually playing.
    Flying,
    /// Escape was pressed. The world is frozen with the menu over it.
    Paused,
    /// Hit something. Same as paused, except the crow gets to play its
    /// death animation and the only way out is a restart.
    Crashed,
}

/// The order gameplay runs in each frame. Everything in here only runs while
/// we're actually flying, which is what makes pausing work.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Sim {
    /// Timers that drop new owls, branches and grapes into the world.
    Spawn,
    /// The crow, plus everything sliding in from the right.
    Move,
    /// Hit tests, once everything has settled for the frame.
    Collide,
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.06, 0.08, 0.06)))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(display::primary_window()),
                    ..default()
                })
                // It's all pixel art, scaled up 3-4x. Linear filtering would
                // smear it.
                .set(ImagePlugin::default_nearest())
                // We don't ship .meta files. On the web every one of those
                // lookups would be a wasted request and a 404 in the console.
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                }),
        )
        .init_state::<GameState>()
        .configure_sets(
            Update,
            (Sim::Spawn, Sim::Move, Sim::Collide)
                .chain()
                .run_if(in_state(GameState::Flying)),
        )
        .add_plugins((
            assets::plugin,
            save::plugin,
            audio::plugin,
            display::plugin,
            run::plugin,
            menu::plugin,
            hud::plugin,
            background::plugin,
            player::plugin,
            obstacles::plugin,
            grapes::plugin,
            animation::plugin,
        ))
        .run();
}
