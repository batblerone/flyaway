//! Flipbook animation for the sprite sheets in `assets.rs`.
//!
//! An `Animator` remembers which animation is playing and how far into it we
//! are. Each frame it picks the right image and atlas index for the entity's
//! `Sprite`. Switching animations is just `animator.play(Anim::Whatever)`.

use bevy::prelude::*;

use crate::GameState;
use crate::assets::{GameAssets, Sheet};
use crate::player::Crow;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, animate.after(crate::Sim::Move));
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anim {
    CrowWalk,
    CrowFly,
    CrowFall,
    CrowLand,
    CrowDie,
    Owl,
    Grapes,
}

impl Anim {
    fn fps(self) -> f32 {
        match self {
            Anim::CrowFall => 5.0,
            Anim::Grapes => 7.0,
            Anim::Owl => 12.0,
            _ => 10.0,
        }
    }

    /// Landing and dying play once and then hold on their last frame.
    fn loops(self) -> bool {
        !matches!(self, Anim::CrowLand | Anim::CrowDie)
    }

    pub fn sheet(self, assets: &GameAssets) -> &Sheet {
        match self {
            Anim::CrowWalk => &assets.crow_walk,
            Anim::CrowFly => &assets.crow_fly,
            Anim::CrowFall => &assets.crow_fall,
            Anim::CrowLand => &assets.crow_land,
            Anim::CrowDie => &assets.crow_die,
            Anim::Owl => &assets.owl,
            Anim::Grapes => &assets.grapes,
        }
    }
}

#[derive(Component)]
pub struct Animator {
    anim: Anim,
    frame: usize,
    /// Time spent on the current frame.
    elapsed: f32,
    /// Set once a non-looping animation has run off its last frame.
    finished: bool,
}

impl Animator {
    pub fn new(anim: Anim) -> Self {
        Animator {
            anim,
            frame: 0,
            elapsed: 0.0,
            finished: false,
        }
    }

    /// Switch to `anim`. Asking for the one that's already playing does
    /// nothing, so it's fine to call this every frame.
    pub fn play(&mut self, anim: Anim) {
        if self.anim != anim {
            *self = Animator::new(anim);
        }
    }

    pub fn current(&self) -> Anim {
        self.anim
    }

    pub fn finished(&self) -> bool {
        self.finished
    }

    fn advance(&mut self, dt: f32, frame_count: usize) {
        let frame_time = 1.0 / self.anim.fps();
        self.elapsed += dt;
        while self.elapsed >= frame_time && !self.finished {
            self.elapsed -= frame_time;
            if self.frame + 1 < frame_count {
                self.frame += 1;
            } else if self.anim.loops() {
                self.frame = 0;
            } else {
                self.finished = true;
            }
        }
    }
}

/// The sprite to spawn an animated entity with, showing the first frame.
pub fn first_frame(assets: &GameAssets, anim: Anim) -> Sprite {
    anim.sheet(assets).sprite(0)
}

fn animate(
    time: Res<Time>,
    state: Res<State<GameState>>,
    assets: Res<GameAssets>,
    mut animated: Query<(&mut Animator, &mut Sprite, Has<Crow>)>,
) {
    for (mut animator, mut sprite, is_crow) in &mut animated {
        // Pausing freezes everything. A crash freezes everything except the
        // crow, who still gets to tumble over.
        let running = match state.get() {
            GameState::Title | GameState::Flying => true,
            GameState::Crashed => is_crow,
            GameState::Paused => false,
        };

        let sheet = animator.anim.sheet(&assets);
        if running {
            animator.advance(time.delta_secs(), sheet.frames);
        }

        // Only write to the sprite when the frame actually changed. Any
        // mutable access counts as a change in Bevy, and a changed sprite
        // gets re-extracted for rendering.
        let index = sheet.first + animator.frame;
        let up_to_date = sprite.image == sheet.image
            && sprite
                .texture_atlas
                .as_ref()
                .is_some_and(|atlas| atlas.layout == sheet.layout && atlas.index == index);
        if !up_to_date {
            sprite.image = sheet.image.clone();
            sprite.texture_atlas = Some(TextureAtlas {
                layout: sheet.layout.clone(),
                index,
            });
        }
    }
}
