//! Owls and branches, the two things that end a run.
//!
//! Owls fly in at whatever height the crow is at when they spawn, so sitting
//! still is never safe. Branches come in at a random height and angle, and
//! a fair few of them spin.
//!
//! This file also has `Scroll`, the "slide left at this speed" component
//! that grapes use too, and the clean-up for anything that's gone off the
//! left edge of the screen.

use std::f32::consts::{FRAC_PI_2, TAU};

use bevy::prelude::*;

use crate::GameState;
use crate::Sim;
use crate::animation::{Anim, Animator, first_frame};
use crate::assets::GameAssets;
use crate::collision::{circle_hits_box, circles_touch};
use crate::player::{BODY_RADIUS, Crow, HURT_RADIUS, kill};
use crate::run::{RunEntity, StartRun};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, reset_timer.run_if(on_message::<StartRun>))
        .add_systems(Update, spawn_obstacles.in_set(Sim::Spawn))
        .add_systems(
            Update,
            (scroll, spin_branches, despawn_offscreen).in_set(Sim::Move),
        )
        .add_systems(Update, hit_obstacles.in_set(Sim::Collide));
}

/// Slides an entity leftward at `speed` pixels per second.
#[derive(Component)]
pub struct Scroll {
    pub speed: f32,
}

/// Past this x everything is well off screen and can go.
const DESPAWN_X: f32 = -1000.0;
/// New obstacles appear this far right, comfortably out of view.
const SPAWN_X: f32 = 1088.0;
/// The longest gap between two obstacles. The actual gap is random, anywhere
/// from half a second up to this.
const MAX_GAP: f32 = 8.0;

const OWL_SPEED: f32 = 480.0;
const OWL_SCALE: f32 = 4.0;
/// The owl's hitbox is a circle roughly the size of its body.
const OWL_RADIUS: f32 = 8.0 * OWL_SCALE;
/// The owl art isn't centred in its frame. Its body sits a little down and to
/// the left of the sprite's middle, so that's where the hitbox goes.
const OWL_HITBOX_OFFSET: Vec2 = Vec2::new(-8.0, -8.0);

const BRANCH_SPEED: f32 = 480.0;
const BRANCH_SCALE: f32 = 4.0;
/// Radians per second for the spinning ones.
const BRANCH_SPIN: f32 = 10.0;
/// The hitbox covers the stick and both leafy ends, in unscaled pixels.
const BRANCH_HALF_SIZE: Vec2 = Vec2::new(39.5, 8.5);
const BRANCH_HITBOX_OFFSET: Vec2 = Vec2::new(0.5, -0.5);
/// Branches stay a bit clear of the floor and ceiling so there's always a gap
/// to get through.
const BRANCH_MAX_Y: f32 = 225.0;

#[derive(Component)]
pub struct Owl;

#[derive(Component)]
pub struct Branch {
    angle: f32,
    spin: f32,
}

#[derive(Resource)]
struct ObstacleTimer(Timer);

fn random_gap() -> Timer {
    Timer::from_seconds(0.5 + fastrand::f32() * (MAX_GAP - 0.5), TimerMode::Once)
}

fn reset_timer(mut commands: Commands) {
    commands.insert_resource(ObstacleTimer(random_gap()));
}

enum Kind {
    Owl,
    Branch,
    SpinningBranch,
}

/// 40% owls, 35% still branches, 25% spinning ones.
fn pick_kind() -> Kind {
    match fastrand::u32(0..100) {
        0..40 => Kind::Owl,
        40..75 => Kind::Branch,
        _ => Kind::SpinningBranch,
    }
}

fn spawn_obstacles(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<GameAssets>,
    timer: Option<ResMut<ObstacleTimer>>,
    crows: Query<(&Crow, &Transform)>,
) {
    let Some(mut timer) = timer else { return };
    if !timer.0.tick(time.delta()).is_finished() {
        return;
    }
    timer.0 = random_gap();

    let Ok((crow, crow_transform)) = crows.single() else {
        return;
    };
    if crow.dead {
        return;
    }

    match pick_kind() {
        Kind::Owl => {
            let at = Vec2::new(SPAWN_X, crow_transform.translation.y) - OWL_HITBOX_OFFSET;
            commands.spawn((
                Name::new("Owl"),
                Owl,
                Scroll { speed: OWL_SPEED },
                first_frame(&assets, Anim::Owl),
                Animator::new(Anim::Owl),
                Transform::from_translation(at.extend(5.0)).with_scale(Vec3::splat(OWL_SCALE)),
                RunEntity,
            ));
        }
        kind => {
            let y = (fastrand::f32() * 2.0 - 1.0) * BRANCH_MAX_Y;
            let spin = if matches!(kind, Kind::SpinningBranch) {
                BRANCH_SPIN
            } else {
                0.0
            };
            spawn_branch(
                &mut commands,
                &assets,
                Vec2::new(SPAWN_X, y),
                fastrand::f32() * TAU,
                spin,
            );
        }
    }
}

fn spawn_branch(commands: &mut Commands, assets: &GameAssets, at: Vec2, angle: f32, spin: f32) {
    // The branch is a stick with a leafy tuft stuck on each end. The parent
    // carries position, rotation and scale; the children are laid out in the
    // unscaled 16px-per-tile space the art was drawn in.
    commands.spawn((
        Name::new("Branch"),
        Branch { angle, spin },
        Scroll {
            speed: BRANCH_SPEED,
        },
        Transform::from_translation(at.extend(4.0))
            .with_rotation(Quat::from_rotation_z(angle))
            .with_scale(Vec3::splat(BRANCH_SCALE)),
        Visibility::default(),
        RunEntity,
        children![
            (
                Sprite {
                    image: assets.branch.clone(),
                    // The stick art is square; squash it into a long thin twig.
                    custom_size: Some(Vec2::new(59.0, 36.0)),
                    ..default()
                },
                Transform::from_xyz(2.5, -1.0, 0.1),
            ),
            (
                assets.bush.sprite(0),
                Transform::from_xyz(-31.0, 0.0, 0.2)
                    .with_rotation(Quat::from_rotation_z(FRAC_PI_2)),
            ),
            (
                assets.bush.sprite(1),
                Transform::from_xyz(32.0, -1.0, 0.2)
                    .with_rotation(Quat::from_rotation_z(-FRAC_PI_2)),
            ),
        ],
    ));
}

fn scroll(time: Res<Time>, mut movers: Query<(&Scroll, &mut Transform)>) {
    for (scroll, mut transform) in &mut movers {
        transform.translation.x -= scroll.speed * time.delta_secs();
    }
}

fn spin_branches(time: Res<Time>, mut branches: Query<(&mut Branch, &mut Transform)>) {
    for (mut branch, mut transform) in &mut branches {
        if branch.spin != 0.0 {
            branch.angle = (branch.angle + branch.spin * time.delta_secs()) % TAU;
            transform.rotation = Quat::from_rotation_z(branch.angle);
        }
    }
}

fn despawn_offscreen(mut commands: Commands, movers: Query<(Entity, &Transform), With<Scroll>>) {
    for (entity, transform) in &movers {
        if transform.translation.x < DESPAWN_X {
            commands.entity(entity).despawn();
        }
    }
}

fn hit_obstacles(
    mut crows: Query<(&mut Crow, &Transform, &mut Animator)>,
    owls: Query<&Transform, With<Owl>>,
    branches: Query<(&Branch, &Transform)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (mut crow, crow_transform, mut animator) in &mut crows {
        let at = crow_transform.translation.truncate();

        let hit_owl = owls.iter().any(|owl| {
            let centre = owl.translation.truncate() + OWL_HITBOX_OFFSET;
            circles_touch(at, BODY_RADIUS, centre, OWL_RADIUS)
        });

        let hit_branch = branches.iter().any(|(branch, transform)| {
            let rotation = Mat2::from_angle(branch.angle);
            let centre =
                transform.translation.truncate() + rotation * (BRANCH_HITBOX_OFFSET * BRANCH_SCALE);
            circle_hits_box(
                at,
                HURT_RADIUS,
                centre,
                BRANCH_HALF_SIZE * BRANCH_SCALE,
                branch.angle,
            )
        });

        if (hit_owl || hit_branch) && kill(&mut crow, &mut animator) {
            next_state.set(GameState::Crashed);
        }
    }
}
