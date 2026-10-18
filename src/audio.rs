//! Music and sound effects.
//!
//! There are two music tracks: a calmer one for menus and a busier one for
//! flying. Whenever the game state changes we work out which track should be
//! on and swap if it's different.

use bevy::audio::Volume;
use bevy::prelude::*;

use crate::GameState;
use crate::assets::GameAssets;
use crate::save::Settings;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (pick_track, follow_volume));
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Track {
    Menu,
    Game,
}

#[derive(Component)]
struct Music(Track);

/// The music sits well under the sound effects. Squaring the slider value
/// does that, and also happens to make the slider feel more even, since
/// hearing is closer to logarithmic than linear.
fn music_volume(settings: &Settings) -> Volume {
    Volume::Linear(settings.volume * settings.volume)
}

fn pick_track(
    mut commands: Commands,
    state: Res<State<GameState>>,
    assets: Res<GameAssets>,
    settings: Res<Settings>,
    playing: Query<(Entity, &Music)>,
) {
    // The first frame counts as a change too, which is what starts the
    // menu music on launch.
    if !state.is_changed() {
        return;
    }
    let wanted = match state.get() {
        GameState::Flying => Track::Game,
        GameState::Title | GameState::Paused | GameState::Crashed => Track::Menu,
    };
    if playing.iter().any(|(_, music)| music.0 == wanted) {
        return;
    }

    for (entity, _) in &playing {
        commands.entity(entity).despawn();
    }
    let source = match wanted {
        Track::Menu => assets.menu_music.clone(),
        Track::Game => assets.game_music.clone(),
    };
    commands.spawn((
        Name::new("Music"),
        Music(wanted),
        AudioPlayer(source),
        PlaybackSettings::LOOP.with_volume(music_volume(&settings)),
    ));
}

/// Keep the playing track in step with the volume slider.
fn follow_volume(settings: Res<Settings>, mut sinks: Query<&mut AudioSink, With<Music>>) {
    if settings.is_changed() {
        for mut sink in &mut sinks {
            sink.set_volume(music_volume(&settings));
        }
    }
}

/// Fire off a one-shot sound. The entity cleans itself up when it's done.
pub fn play_effect(commands: &mut Commands, sound: Handle<AudioSource>, settings: &Settings) {
    if settings.volume > 0.0 {
        commands.spawn((
            AudioPlayer(sound),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(settings.volume)),
        ));
    }
}
