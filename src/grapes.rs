//! Grapes: the one thing in the forest that's good to fly into.
//!
//! They drift in a little slower than the obstacles, and every one eaten adds
//! to a grape count that sticks around between runs.

use bevy::prelude::*;

use crate::Sim;
use crate::animation::{Anim, Animator, first_frame};
use crate::assets::GameAssets;
use crate::audio::play_effect;
use crate::collision::circles_touch;
use crate::obstacles::Scroll;
use crate::player::{BODY_RADIUS, Crow};
use crate::run::{Progress, RunEntity, StartRun};
use crate::save::{SaveRequested, Settings};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, reset_timer.run_if(on_message::<StartRun>))
        .add_systems(Update, spawn_grapes.in_set(Sim::Spawn))
        .add_systems(Update, eat_grapes.in_set(Sim::Collide));
}

const SPAWN_X: f32 = 896.0;
const SPEED: f32 = 400.0;
const SCALE: f32 = 3.0;
const RADIUS: f32 = 8.0 * SCALE;
const MAX_Y: f32 = 250.0;
/// Grapes turn up about twice as often as obstacles.
const MAX_GAP: f32 = 4.0;

#[derive(Component)]
pub struct Grape;

#[derive(Resource)]
struct GrapeTimer(Timer);

fn random_gap() -> Timer {
    Timer::from_seconds(0.5 + fastrand::f32() * (MAX_GAP - 0.5), TimerMode::Once)
}

fn reset_timer(mut commands: Commands) {
    commands.insert_resource(GrapeTimer(random_gap()));
}

fn spawn_grapes(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<GameAssets>,
    timer: Option<ResMut<GrapeTimer>>,
) {
    let Some(mut timer) = timer else { return };
    if !timer.0.tick(time.delta()).is_finished() {
        return;
    }
    timer.0 = random_gap();

    let y = (fastrand::f32() * 2.0 - 1.0) * MAX_Y;
    commands.spawn((
        Name::new("Grapes"),
        Grape,
        Scroll { speed: SPEED },
        first_frame(&assets, Anim::Grapes),
        Animator::new(Anim::Grapes),
        Transform::from_xyz(SPAWN_X, y, 6.0).with_scale(Vec3::splat(SCALE)),
        RunEntity,
    ));
}

fn eat_grapes(
    mut commands: Commands,
    assets: Res<GameAssets>,
    settings: Res<Settings>,
    mut progress: ResMut<Progress>,
    mut save: MessageWriter<SaveRequested>,
    crows: Query<(&Crow, &Transform)>,
    grapes: Query<(Entity, &Transform), With<Grape>>,
) {
    let Ok((crow, crow_transform)) = crows.single() else {
        return;
    };
    if crow.dead {
        return;
    }
    let at = crow_transform.translation.truncate();

    for (entity, grape) in &grapes {
        if circles_touch(at, BODY_RADIUS, grape.translation.truncate(), RADIUS) {
            commands.entity(entity).despawn();
            progress.grapes += 1;
            play_effect(&mut commands, assets.munch.clone(), &settings);
            // The grape count is the one thing worth keeping even if the
            // player closes the tab mid-run, so save it straight away.
            save.write(SaveRequested);
        }
    }
}
