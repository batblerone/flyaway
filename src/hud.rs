//! The in-flight HUD: best distance, current distance and the grape count,
//! up in the top-left corner.
//!
//! Also home to `stage`, the 1280x720 box every bit of UI is laid out in.

use bevy::prelude::*;

use crate::assets::GameAssets;
use crate::run::{Progress, RunEntity, StartRun};
use crate::{VIEW_HEIGHT, VIEW_WIDTH};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, spawn_hud.run_if(on_message::<StartRun>))
        .add_systems(Update, update_hud.run_if(resource_changed::<Progress>));
}

/// A full-window node that centres a 1280x720 one inside it, with
/// `children` (a `children![...]` list) going in the inner one. `display.rs`
/// scales the UI so that inner box exactly fits the window, which means
/// everything can be positioned in plain 720p pixels and it'll line up with
/// the game world underneath.
pub fn stage(children: impl Bundle) -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        Pickable::IGNORE,
        children![(
            Node {
                width: px(VIEW_WIDTH),
                height: px(VIEW_HEIGHT),
                flex_shrink: 0.0,
                ..default()
            },
            Pickable::IGNORE,
            children,
        )],
    )
}

#[derive(Component)]
enum Readout {
    Best,
    Distance,
    Grapes,
}

const FONT_SIZE: f32 = 30.0;

/// A text node pinned at (x, y) from the stage's top-left corner.
fn label(assets: &GameAssets, text: &str, x: f32, y: f32) -> impl Bundle {
    (
        Text::new(text),
        TextFont {
            font: assets.font.clone().into(),
            font_size: FontSize::Px(FONT_SIZE),
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            left: px(x),
            top: px(y),
            ..default()
        },
        Pickable::IGNORE,
    )
}

fn spawn_hud(mut commands: Commands, assets: Res<GameAssets>, progress: Res<Progress>) {
    let a = &*assets;
    commands.spawn((
        Name::new("HUD"),
        RunEntity,
        stage(children![
            label(a, "Furthest:", 128.0, 104.0),
            (
                label(a, &format!("{} m", progress.best), 240.0, 104.0),
                Readout::Best
            ),
            label(a, "Distance:", 128.0, 144.0),
            (label(a, "0 m", 240.0, 144.0), Readout::Distance),
            (
                ImageNode::new(a.grape_icon.clone()),
                Node {
                    position_type: PositionType::Absolute,
                    left: px(328),
                    top: px(120),
                    width: px(45),
                    height: px(48),
                    ..default()
                },
                Pickable::IGNORE,
            ),
            (
                label(a, &progress.grapes.to_string(), 384.0, 128.0),
                Readout::Grapes
            ),
        ]),
    ));
}

fn update_hud(progress: Res<Progress>, mut readouts: Query<(&Readout, &mut Text)>) {
    for (readout, mut text) in &mut readouts {
        let value = match readout {
            Readout::Best => format!("{} m", progress.best),
            Readout::Distance => format!("{} m", progress.metres()),
            Readout::Grapes => progress.grapes.to_string(),
        };
        // Comparing first avoids re-laying-out text that hasn't changed,
        // which is most frames for most of these.
        if text.0 != value {
            text.0 = value;
        }
    }
}
