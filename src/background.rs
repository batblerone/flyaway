//! The scrolling forest behind everything.
//!
//! It's one painting, tiled side by side and slid left. When a copy has gone
//! all the way off the left, it hops to the back of the line. The title
//! screen uses the same painting, drifting much more slowly.

use bevy::prelude::*;

use crate::assets::GameAssets;
use crate::run::{RunEntity, StartRun};
use crate::{GameState, Sim, VIEW_HEIGHT, VIEW_WIDTH};

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Title), spawn_title_backdrop)
        .add_systems(Update, spawn_game_backdrop.run_if(on_message::<StartRun>))
        .add_systems(
            Update,
            scroll_tiles::<TitleBackdrop>.run_if(in_state(GameState::Title)),
        )
        .add_systems(Update, scroll_tiles::<RunEntity>.in_set(Sim::Move));
}

/// One copy of the painting in a looping row of `copies`, each `spacing`
/// pixels apart.
#[derive(Component)]
struct Tile {
    speed: f32,
    spacing: f32,
    copies: f32,
}

#[derive(Component)]
struct TitleBackdrop;

/// How fast the forest goes by while flying. Deliberately slower than the
/// owls and branches (480), so the trees read as being further away.
const GAME_SPEED: f32 = 300.0;

/// In game the painting is drawn a touch smaller than the screen and nudged
/// a few pixels up and right. That puts the path in the art just under the
/// crow's feet.
const GAME_TILE_SIZE: Vec2 = Vec2::new(1158.0, 678.0);
const GAME_TILE_OFFSET: Vec2 = Vec2::new(5.0, 11.0);
const GAME_TILE_SPACING: f32 = 1152.0;

fn spawn_title_backdrop(mut commands: Commands, assets: Res<GameAssets>) {
    for i in 0..2 {
        commands.spawn((
            Name::new("Title backdrop"),
            TitleBackdrop,
            DespawnOnExit(GameState::Title),
            Tile {
                speed: 12.8,
                spacing: VIEW_WIDTH,
                copies: 2.0,
            },
            Sprite {
                image: assets.forest.clone(),
                custom_size: Some(Vec2::new(VIEW_WIDTH, VIEW_HEIGHT)),
                ..default()
            },
            Transform::from_xyz(i as f32 * VIEW_WIDTH, 0.0, 0.0),
        ));
    }
}

fn spawn_game_backdrop(mut commands: Commands, assets: Res<GameAssets>) {
    // Four copies, starting one to the left of centre, is enough to always
    // cover the screen however the row is shifted.
    for i in -1..3 {
        let at = GAME_TILE_OFFSET + Vec2::X * (i as f32 * GAME_TILE_SPACING);
        commands.spawn((
            Name::new("Backdrop"),
            RunEntity,
            Tile {
                speed: GAME_SPEED,
                spacing: GAME_TILE_SPACING,
                copies: 4.0,
            },
            Sprite {
                image: assets.forest.clone(),
                custom_size: Some(GAME_TILE_SIZE),
                ..default()
            },
            Transform::from_translation(at.extend(0.0)),
        ));
    }
}

/// `Marker` picks which set of tiles to move, so the title backdrop and the
/// in-game one can run under different conditions.
fn scroll_tiles<Marker: Component>(
    time: Res<Time>,
    mut tiles: Query<(&Tile, &mut Transform), With<Marker>>,
) {
    for (tile, mut transform) in &mut tiles {
        let mut x = transform.translation.x - tile.speed * time.delta_secs();
        let row_width = tile.spacing * tile.copies;
        if x < -row_width / 2.0 {
            x += row_width;
        }
        transform.translation.x = x;
    }
}
