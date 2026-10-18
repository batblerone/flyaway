//! The crow.
//!
//! Holding "up" eases the crow toward a fixed climb speed; letting go hands
//! it over to gravity. It can't leave the screen: there's a floor it walks
//! along and a ceiling it bumps against.

use bevy::prelude::*;

use crate::Sim;
use crate::animation::{Anim, Animator, first_frame};
use crate::assets::GameAssets;
use crate::run::{RunEntity, StartRun};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, spawn.run_if(on_message::<StartRun>))
        .add_systems(Update, fly.in_set(Sim::Move));
}

const GRAVITY: f32 = 980.0;
/// Top climbing speed, in pixels per second.
const CLIMB_SPEED: f32 = 600.0;
/// How hard the crow pulls toward `CLIMB_SPEED`. Higher feels snappier;
/// lower feels like a heavier bird.
const CLIMB_ACCEL: f32 = 1000.0;

const FLOOR_Y: f32 = -290.0;
const CEILING_Y: f32 = 290.0;
const START: Vec2 = Vec2::new(-384.0, -256.0);

/// Radius of the crow's body for bumping into owls and eating grapes.
pub const BODY_RADIUS: f32 = 32.0;
/// Branches use a slightly bigger circle. It's barely different, but it
/// means clipping a leaf with a wingtip still counts.
pub const HURT_RADIUS: f32 = 33.0;

/// Frame hitches (tab switches, window drags) can hand us a huge delta. Past
/// this we'd rather the game slow down than have the crow teleport.
const MAX_STEP: f32 = 1.0 / 20.0;

#[derive(Component, Default)]
pub struct Crow {
    /// Vertical speed, positive is up.
    pub velocity: f32,
    pub dead: bool,
    on_floor: bool,
}

fn spawn(mut commands: Commands, assets: Res<GameAssets>) {
    commands.spawn((
        Name::new("Crow"),
        Crow::default(),
        first_frame(&assets, Anim::CrowWalk),
        Animator::new(Anim::CrowWalk),
        Transform::from_translation(START.extend(10.0)).with_scale(Vec3::splat(4.0)),
        RunEntity,
    ));
}

/// Space, the left mouse button, or any finger on the screen.
pub fn holding_up(
    keys: &ButtonInput<KeyCode>,
    mouse: &ButtonInput<MouseButton>,
    touches: &Touches,
) -> bool {
    keys.pressed(KeyCode::Space)
        || mouse.pressed(MouseButton::Left)
        || touches.iter().next().is_some()
}

fn fly(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    mut crows: Query<(&mut Crow, &mut Transform, &mut Animator)>,
) {
    let dt = time.delta_secs().min(MAX_STEP);
    let flapping = holding_up(&keys, &mouse, &touches);

    for (mut crow, mut transform, mut animator) in &mut crows {
        if crow.dead {
            continue;
        }

        if flapping {
            // Close a fraction of the gap to climb speed each frame. Starting
            // from a fall, that turns around quickly but not instantly.
            let pull = (CLIMB_ACCEL * dt / CLIMB_SPEED).min(1.0);
            crow.velocity += (CLIMB_SPEED - crow.velocity) * pull;
        } else {
            crow.velocity -= GRAVITY * dt;
        }

        let was_on_floor = crow.on_floor;
        let mut y = transform.translation.y + crow.velocity * dt;
        crow.on_floor = false;
        if y - BODY_RADIUS <= FLOOR_Y {
            y = FLOOR_Y + BODY_RADIUS;
            crow.velocity = crow.velocity.max(0.0);
            crow.on_floor = true;
        }
        if y + BODY_RADIUS >= CEILING_Y {
            y = CEILING_Y - BODY_RADIUS;
            crow.velocity = crow.velocity.min(0.0);
        }
        transform.translation.y = y;

        let next = if flapping {
            Anim::CrowFly
        } else if !crow.on_floor {
            Anim::CrowFall
        } else if !was_on_floor {
            // Just touched down.
            Anim::CrowLand
        } else if animator.current() == Anim::CrowLand && !animator.finished() {
            // Let the landing finish before breaking into a walk.
            Anim::CrowLand
        } else {
            Anim::CrowWalk
        };
        animator.play(next);
    }
}

/// Marks the crow as dead and swaps in the death animation. Returns false if
/// it was already dead, so callers can avoid crashing twice in one frame.
pub fn kill(crow: &mut Crow, animator: &mut Animator) -> bool {
    if crow.dead {
        return false;
    }
    crow.dead = true;
    animator.play(Anim::CrowDie);
    true
}
