//! Menus: the title screen, pause, game over, settings and credits.
//!
//! Which panel is showing is its own little state machine, separate from
//! `GameState`, since the same main menu appears on the title screen and when
//! paused. Each panel is built when its state is entered and torn down
//! automatically when it's left (that's what `DespawnOnExit` is for).
//!
//! The controls are Bevy's built-in headless widgets: `Button`, `Checkbox`
//! and `Slider` handle the clicking, dragging and keyboard side, and send
//! events when something happens. All this file adds is the look, plus what
//! each control actually does.

use bevy::app::AppExit;
use bevy::ecs::spawn::SpawnIter;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::{Checked, Pressed};
use bevy::ui_widgets::{
    Activate, Button, Checkbox, Slider, SliderPrecision, SliderRange, SliderStep, SliderThumb,
    SliderValue, TrackClick, ValueChange, observe,
};

use crate::GameState;
use crate::assets::GameAssets;
use crate::hud::stage;
use crate::run::StartRun;
use crate::save::{SaveRequested, Settings};

pub fn plugin(app: &mut App) {
    app.init_state::<MenuScreen>()
        .add_systems(OnEnter(GameState::Title), show(MenuScreen::Main))
        .add_systems(OnEnter(GameState::Paused), show(MenuScreen::Main))
        .add_systems(OnEnter(GameState::Crashed), show(MenuScreen::GameOver))
        .add_systems(OnEnter(GameState::Flying), show(MenuScreen::Hidden))
        .add_systems(OnEnter(MenuScreen::Main), main_panel)
        .add_systems(OnEnter(MenuScreen::GameOver), game_over_panel)
        .add_systems(OnEnter(MenuScreen::Settings), settings_panel)
        .add_systems(OnEnter(MenuScreen::Credits), credits_panel)
        .add_systems(Update, (button_colours, refresh_settings_widgets))
        .add_observer(on_activate);
}

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MenuScreen {
    #[default]
    Hidden,
    Main,
    GameOver,
    Settings,
    Credits,
}

/// A system that switches to `screen`, for wiring up to `OnEnter`.
fn show(screen: MenuScreen) -> impl FnMut(ResMut<NextState<MenuScreen>>) {
    move |mut next: ResMut<NextState<MenuScreen>>| next.set(screen)
}

/// What a menu button does when it's clicked.
#[derive(Component, Clone, Copy)]
enum Action {
    Play,
    Settings,
    Credits,
    Back,
    Quit,
}

#[derive(Component)]
struct FullscreenToggle;
#[derive(Component)]
struct FullscreenTick;
#[derive(Component)]
struct VolumeSlider;
#[derive(Component)]
struct VolumeFill;

const TITLE_SIZE: f32 = 140.0;
const TEXT_SIZE: f32 = 40.0;
const BUTTON_WIDTH: f32 = 353.0;
const BUTTON_HEIGHT: f32 = 58.0;
const THUMB_WIDTH: f32 = 14.0;

const TEXT: Color = Color::srgb(0.875, 0.875, 0.875);
// Smoky, see-through buttons so the forest still shows behind the menu.
const BUTTON: Color = Color::srgba(0.1, 0.1, 0.1, 0.6);
const BUTTON_HOVER: Color = Color::srgba(0.225, 0.225, 0.225, 0.6);
const BUTTON_PRESSED: Color = Color::srgba(0.0, 0.0, 0.0, 0.6);
const TRACK: Color = Color::srgba(0.1, 0.1, 0.1, 0.6);
const TICK: Color = Color::srgba(0.875, 0.875, 0.875, 0.5);

fn text(assets: &GameAssets, value: &str, size: f32, colour: Color) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            font: assets.font.clone().into(),
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(colour),
        TextLayout::justify(Justify::Center),
        Pickable::IGNORE,
    )
}

fn button(assets: &GameAssets, label: &str, action: Action) -> impl Bundle {
    (
        Button,
        // Picking only keeps hover state up to date on entities that ask
        // for it, and the colours below need it.
        Hovered::default(),
        action,
        Node {
            width: px(BUTTON_WIDTH),
            height: px(BUTTON_HEIGHT),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(BUTTON),
        children![text(assets, label, TEXT_SIZE, TEXT)],
    )
}

/// Every panel has the same shape: a big heading across the top 256 pixels,
/// and a centred column of whatever `body` is underneath.
fn spawn_panel(
    commands: &mut Commands,
    assets: &GameAssets,
    screen: MenuScreen,
    heading: &str,
    body: impl Bundle,
) {
    commands.spawn((
        Name::new(format!("{screen:?} menu")),
        DespawnOnExit(screen),
        // Sit on top of the HUD.
        GlobalZIndex(10),
        stage(children![(
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            Pickable::IGNORE,
            children![
                (
                    Node {
                        width: percent(100),
                        height: px(256),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    Pickable::IGNORE,
                    children![text(assets, heading, TITLE_SIZE, Color::WHITE)],
                ),
                (
                    Node {
                        flex_grow: 1.0,
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        row_gap: px(10),
                        ..default()
                    },
                    Pickable::IGNORE,
                    body,
                ),
            ],
        )]),
    ));
}

/// Quitting makes no sense in a browser tab, so the web build leaves that
/// button out.
const CAN_QUIT: bool = !cfg!(target_arch = "wasm32");

fn main_panel(mut commands: Commands, assets: Res<GameAssets>) {
    let a = &*assets;
    if CAN_QUIT {
        spawn_panel(
            &mut commands,
            a,
            MenuScreen::Main,
            "Fly Away, Fly!",
            children![
                button(a, "Play", Action::Play),
                button(a, "Settings", Action::Settings),
                button(a, "Credits", Action::Credits),
                button(a, "Exit", Action::Quit),
            ],
        );
    } else {
        spawn_panel(
            &mut commands,
            a,
            MenuScreen::Main,
            "Fly Away, Fly!",
            children![
                button(a, "Play", Action::Play),
                button(a, "Settings", Action::Settings),
                button(a, "Credits", Action::Credits),
            ],
        );
    }
}

fn game_over_panel(mut commands: Commands, assets: Res<GameAssets>) {
    let a = &*assets;
    if CAN_QUIT {
        spawn_panel(
            &mut commands,
            a,
            MenuScreen::GameOver,
            "You crashed, again?",
            children![
                button(a, "Restart?", Action::Play),
                button(a, "Settings", Action::Settings),
                button(a, "Credits", Action::Credits),
                button(a, "Exit", Action::Quit),
            ],
        );
    } else {
        spawn_panel(
            &mut commands,
            a,
            MenuScreen::GameOver,
            "You crashed, again?",
            children![
                button(a, "Restart?", Action::Play),
                button(a, "Settings", Action::Settings),
                button(a, "Credits", Action::Credits),
            ],
        );
    }
}

fn credits_panel(mut commands: Commands, assets: Res<GameAssets>) {
    let a = &*assets;
    spawn_panel(
        &mut commands,
        a,
        MenuScreen::Credits,
        "Fly Away, Fly!",
        children![
            text(a, "A Game by Silly Crow Studios", TEXT_SIZE, Color::WHITE),
            text(
                a,
                "Music by FarBeyond Studio (https://farbeyond-studio.carrd.co/)",
                TEXT_SIZE,
                Color::WHITE
            ),
            // A bit of breathing room before the button.
            (
                Node {
                    height: px(10),
                    ..default()
                },
                Pickable::IGNORE
            ),
            button(a, "Back", Action::Back),
        ],
    );
}

fn settings_panel(mut commands: Commands, assets: Res<GameAssets>, settings: Res<Settings>) {
    let a = &*assets;

    // The checkbox is the whole row, label included, so clicking the word
    // "Fullscreen" works too. The tick inside the little box is just a
    // lighter square that `refresh_settings_widgets` shows or hides.
    let fullscreen = (
        Checkbox,
        FullscreenToggle,
        Hovered::default(),
        Node {
            width: px(BUTTON_WIDTH),
            height: px(BUTTON_HEIGHT),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: px(14),
            ..default()
        },
        BackgroundColor(BUTTON),
        observe(
            |change: On<ValueChange<bool>>, mut settings: ResMut<Settings>| {
                settings.fullscreen = change.value;
            },
        ),
        children![
            (
                Node {
                    width: px(28),
                    height: px(28),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
                Pickable::IGNORE,
                children![(
                    Node {
                        width: px(16),
                        height: px(16),
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                    FullscreenTick,
                    Pickable::IGNORE,
                )],
            ),
            text(a, "Fullscreen", TEXT_SIZE, TEXT),
        ],
    );

    // The volume slider. The outer node is the hit area, taller than the
    // groove so it's easy to grab. Clicking anywhere on it jumps straight
    // there, and dragging follows the pointer even if it wanders off.
    let slider = (
        Slider {
            track_click: TrackClick::Snap,
            ..default()
        },
        VolumeSlider,
        SliderValue(settings.volume),
        SliderRange::new(0.0, 1.0),
        // Arrow keys move it in 5% steps; dragging rounds to 0.1%, which is
        // finer than anyone can hear and keeps the save file tidy.
        SliderStep(0.05),
        SliderPrecision(3),
        Hovered::default(),
        Node {
            width: px(BUTTON_WIDTH),
            height: px(44),
            align_items: AlignItems::Center,
            margin: UiRect::bottom(px(10)),
            ..default()
        },
        observe(
            |change: On<ValueChange<f32>>,
             mut settings: ResMut<Settings>,
             mut save: MessageWriter<SaveRequested>| {
                settings.volume = change.value;
                // `is_final` is set once the mouse button comes back up, which
                // is the moment worth saving. No need to write a file for
                // every pixel of a drag.
                if change.is_final {
                    save.write(SaveRequested);
                }
            },
        ),
        children![
            // The visible groove.
            (
                Node {
                    width: percent(100),
                    height: px(8),
                    ..default()
                },
                BackgroundColor(TRACK),
                Pickable::IGNORE,
            ),
            ticks(),
            // An invisible box, one thumb narrower than the slider, that the
            // thumb and the filled part are positioned in. That way 0% and
            // 100% both leave the whole thumb on the slider, and positions
            // can be plain percentages.
            (
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    right: px(THUMB_WIDTH),
                    top: px(0),
                    bottom: px(0),
                    align_items: AlignItems::Center,
                    ..default()
                },
                Pickable::IGNORE,
                children![
                    (
                        Node {
                            position_type: PositionType::Absolute,
                            width: percent(0),
                            height: px(8),
                            ..default()
                        },
                        BackgroundColor(TEXT),
                        VolumeFill,
                        Pickable::IGNORE,
                    ),
                    (
                        Node {
                            position_type: PositionType::Absolute,
                            left: percent(0),
                            width: px(THUMB_WIDTH),
                            height: px(28),
                            ..default()
                        },
                        BackgroundColor(TEXT),
                        SliderThumb,
                        Pickable::IGNORE,
                    ),
                ],
            ),
        ],
    );

    spawn_panel(
        &mut commands,
        a,
        MenuScreen::Settings,
        "Fly Away, Fly!",
        children![
            fullscreen,
            text(a, "Volume", TEXT_SIZE, Color::WHITE),
            slider,
            button(a, "Back", Action::Back),
        ],
    );
}

/// Ten little marks along the bottom of the slider, lined up with where the
/// middle of the thumb sits at 0%, 11%, ... 100%.
fn ticks() -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            bottom: px(0),
            left: px(THUMB_WIDTH / 2.0 - 1.0),
            right: px(THUMB_WIDTH / 2.0 - 1.0),
            height: px(8),
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        },
        Pickable::IGNORE,
        Children::spawn(SpawnIter((0..10).map(|_| {
            (
                Node {
                    width: px(2),
                    height: percent(100),
                    ..default()
                },
                BackgroundColor(TICK),
                Pickable::IGNORE,
            )
        }))),
    )
}

/// Buttons (and the fullscreen row) light up under the pointer and darken
/// while held down.
fn button_colours(
    mut buttons: Query<
        (&Hovered, Has<Pressed>, &mut BackgroundColor),
        Or<(With<Button>, With<Checkbox>)>,
    >,
) {
    for (hovered, pressed, mut colour) in &mut buttons {
        let wanted = match (pressed, hovered.get()) {
            (true, _) => BUTTON_PRESSED,
            (false, true) => BUTTON_HOVER,
            (false, false) => BUTTON,
        };
        // Compare first: writing the same colour every frame would still count
        // as a change and make Bevy redo work for nothing.
        if colour.0 != wanted {
            colour.0 = wanted;
        }
    }
}

/// Every menu button ends up here when it's clicked (or, if it has keyboard
/// focus, when Enter or Space is pressed on it).
fn on_activate(
    activate: On<Activate>,
    actions: Query<&Action>,
    state: Res<State<GameState>>,
    mut next_menu: ResMut<NextState<MenuScreen>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut start: MessageWriter<StartRun>,
    mut save: MessageWriter<SaveRequested>,
    mut exit: MessageWriter<AppExit>,
) {
    let Ok(action) = actions.get(activate.entity) else {
        return;
    };

    match action {
        Action::Play => match state.get() {
            // From the pause menu, Play just carries on where you left off.
            GameState::Paused => next_state.set(GameState::Flying),
            _ => {
                start.write(StartRun);
            }
        },
        Action::Settings => next_menu.set(MenuScreen::Settings),
        Action::Credits => next_menu.set(MenuScreen::Credits),
        // Back goes to whichever panel this menu opened on.
        Action::Back => next_menu.set(match state.get() {
            GameState::Crashed => MenuScreen::GameOver,
            _ => MenuScreen::Main,
        }),
        Action::Quit => {
            save.write(SaveRequested);
            exit.write(AppExit::Success);
        }
    }
}

/// The widgets only report what the player did; it's up to us to show it.
/// This keeps the checkbox, its tick, and the slider's value, thumb and fill
/// in line with `Settings`.
#[allow(clippy::type_complexity)]
fn refresh_settings_widgets(
    mut commands: Commands,
    settings: Res<Settings>,
    toggles: Query<(Entity, Has<Checked>), With<FullscreenToggle>>,
    mut ticks: Query<&mut BackgroundColor, With<FullscreenTick>>,
    sliders: Query<(Entity, &SliderValue), With<VolumeSlider>>,
    mut thumbs: Query<&mut Node, (With<SliderThumb>, Without<VolumeFill>)>,
    mut fills: Query<&mut Node, (With<VolumeFill>, Without<SliderThumb>)>,
) {
    for (toggle, checked) in &toggles {
        if checked != settings.fullscreen {
            if settings.fullscreen {
                commands.entity(toggle).insert(Checked);
            } else {
                commands.entity(toggle).remove::<Checked>();
            }
        }
    }
    let tick = if settings.fullscreen {
        Color::srgb(0.95, 0.95, 0.95)
    } else {
        Color::NONE
    };
    for mut colour in &mut ticks {
        if colour.0 != tick {
            colour.0 = tick;
        }
    }

    for (slider, value) in &sliders {
        if value.0 != settings.volume {
            commands.entity(slider).insert(SliderValue(settings.volume));
        }
    }
    let position = percent(settings.volume * 100.0);
    for mut thumb in &mut thumbs {
        if thumb.left != position {
            thumb.left = position;
        }
    }
    for mut fill in &mut fills {
        if fill.width != position {
            fill.width = position;
        }
    }
}
