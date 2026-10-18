//! A single run, from pressing Play to hitting a branch.
//!
//! Starting a run is a `StartRun` message. This module clears out whatever
//! was left over from the last one, and every other module listens for the
//! same message to spawn its own pieces: the crow, the backdrop, the HUD,
//! fresh spawn timers. Doing it with a message rather than `OnEnter`
//! matters, because coming back from the pause menu also enters `Flying`
//! and that must not reset anything.

use bevy::prelude::*;

use crate::GameState;
use crate::save::SaveRequested;

pub fn plugin(app: &mut App) {
    app.add_message::<StartRun>()
        .init_resource::<Progress>()
        .add_systems(Update, start_run.run_if(on_message::<StartRun>))
        .add_systems(Update, count_distance.in_set(crate::Sim::Move))
        .add_systems(Update, toggle_pause)
        .add_systems(OnEnter(GameState::Paused), request_save)
        .add_systems(OnEnter(GameState::Crashed), request_save);
}

/// Throw away the current run (if any) and begin a new one.
#[derive(Message, Default)]
pub struct StartRun;

/// Tags everything that belongs to a run, so it can all be cleared at once.
#[derive(Component, Default)]
pub struct RunEntity;

/// How far the crow flies per second. Nothing actually moves the crow
/// forward; the world scrolls past it, so this is purely for the score.
const METRES_PER_SECOND: f32 = 2.0;

#[derive(Resource, Default)]
pub struct Progress {
    /// This run's distance in metres. Kept fractional so the score ticks up
    /// smoothly at any frame rate, and shown rounded down.
    pub distance: f32,
    /// Best distance ever, in whole metres.
    pub best: u32,
    /// Grapes eaten, across every run.
    pub grapes: u32,
}

impl Progress {
    pub fn metres(&self) -> u32 {
        self.distance as u32
    }
}

fn start_run(
    mut commands: Commands,
    old: Query<Entity, With<RunEntity>>,
    mut progress: ResMut<Progress>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for entity in &old {
        commands.entity(entity).despawn();
    }
    progress.distance = 0.0;
    next_state.set(GameState::Flying);
}

fn count_distance(time: Res<Time>, mut progress: ResMut<Progress>) {
    progress.distance += time.delta_secs() * METRES_PER_SECOND;
    // The best score climbs along with the current one rather than waiting
    // for the run to end. It's more fun to watch it get beaten live.
    progress.best = progress.best.max(progress.metres());
}

fn toggle_pause(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }
    match state.get() {
        GameState::Flying => next_state.set(GameState::Paused),
        GameState::Paused => next_state.set(GameState::Flying),
        // Nothing to pause on the title screen, and after a crash the only
        // way forward is Restart.
        GameState::Title | GameState::Crashed => {}
    }
}

fn request_save(mut save: MessageWriter<SaveRequested>) {
    save.write(SaveRequested);
}
