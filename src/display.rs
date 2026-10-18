//! The window, the camera, and keeping a 1280x720 game looking right at
//! whatever size the window (or browser tab) actually is.

use bevy::camera::ScalingMode;
use bevy::prelude::*;
use bevy::window::{MonitorSelection, PrimaryWindow, WindowMode, WindowResized};

use crate::save::Settings;
use crate::{VIEW_HEIGHT, VIEW_WIDTH};

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_camera)
        .add_systems(Update, (scale_ui, apply_fullscreen));
}

pub fn primary_window() -> Window {
    Window {
        title: "Fly Away, Fly!".into(),
        resolution: (VIEW_WIDTH as u32, VIEW_HEIGHT as u32).into(),
        // On the web, draw into the page's <canvas id="game"> and follow its
        // size instead of making a fixed 1280x720 one.
        canvas: Some("#game".into()),
        fit_canvas_to_parent: true,
        ..default()
    }
}

fn spawn_camera(mut commands: Commands) {
    // AutoMin keeps the whole 1280x720 play area on screen at any window
    // size. If the window is a different shape, you just see a bit more
    // forest (or the clear colour) around the edges.
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: VIEW_WIDTH,
                min_height: VIEW_HEIGHT,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}

/// The camera handles the world, but UI is laid out in pixels, so scale it
/// by the same amount. All the UI is built inside a 1280x720 "stage" node
/// (see `hud::stage`), which this makes fill the window exactly.
fn scale_ui(
    mut resized: MessageReader<WindowResized>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut ui_scale: ResMut<UiScale>,
) {
    // Only bother when the window changes, plus once at startup.
    if resized.read().last().is_none() && !ui_scale.is_added() {
        return;
    }
    let Ok(window) = windows.single() else { return };
    let scale = (window.width() / VIEW_WIDTH).min(window.height() / VIEW_HEIGHT);
    if scale > 0.0 && (ui_scale.0 - scale).abs() > f32::EPSILON {
        ui_scale.0 = scale;
    }
}

fn apply_fullscreen(settings: Res<Settings>, mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    if !settings.is_changed() {
        return;
    }
    let Ok(mut window) = windows.single_mut() else {
        return;
    };
    let wanted = if settings.fullscreen {
        WindowMode::BorderlessFullscreen(MonitorSelection::Current)
    } else {
        WindowMode::Windowed
    };
    if window.mode != wanted {
        window.mode = wanted;
    }
}
