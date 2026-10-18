//! Every image, sound and font the game uses, loaded once at startup.
//!
//! Most of the art comes as horizontal strips of animation frames, so each one
//! gets a `Sheet`: the image plus a grid layout telling the sprite renderer
//! where each frame lives.

use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    // Done during setup rather than in a startup system, so the handles exist
    // before the title screen (which is built on the very first frame).
    app.init_resource::<GameAssets>();
}

/// One strip of animation frames.
pub struct Sheet {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
    /// Index of the first frame we care about. Usually 0, but the owl's frames
    /// sit in the middle of a much longer strip.
    pub first: usize,
    pub frames: usize,
}

impl Sheet {
    /// A sprite showing frame `index` of this sheet.
    pub fn sprite(&self, index: usize) -> Sprite {
        Sprite::from_atlas_image(
            self.image.clone(),
            TextureAtlas {
                layout: self.layout.clone(),
                index: self.first + index.min(self.frames - 1),
            },
        )
    }
}

#[derive(Resource)]
pub struct GameAssets {
    pub font: Handle<Font>,

    pub crow_walk: Sheet,
    pub crow_fly: Sheet,
    pub crow_fall: Sheet,
    pub crow_land: Sheet,
    pub crow_die: Sheet,
    pub owl: Sheet,
    pub grapes: Sheet,
    /// The two leafy tufts on the ends of a branch.
    pub bush: Sheet,

    pub branch: Handle<Image>,
    pub forest: Handle<Image>,
    pub grape_icon: Handle<Image>,

    pub menu_music: Handle<AudioSource>,
    pub game_music: Handle<AudioSource>,
    pub munch: Handle<AudioSource>,
}

impl FromWorld for GameAssets {
    fn from_world(world: &mut World) -> Self {
        let server = world.resource::<AssetServer>().clone();
        let mut layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();
        // A strip of `columns` frames, `size` pixels each, `gap` pixels apart.
        let mut strip = |path: &'static str,
                         size: UVec2,
                         columns: u32,
                         gap: u32,
                         first: usize,
                         frames: usize| {
            let layout =
                TextureAtlasLayout::from_grid(size, columns, 1, Some(UVec2::new(gap, 0)), None);
            Sheet {
                image: server.load(path),
                layout: layouts.add(layout),
                first,
                frames,
            }
        };

        let crow = UVec2::splat(40);
        GameAssets {
            font: server.load("fonts/EBGaramond-VariableFont_wght.ttf"),

            crow_walk: strip("sprites/crow_walk_strip16.png", crow, 16, 0, 0, 16),
            crow_fly: strip("sprites/crow_fly_strip6.png", crow, 6, 0, 0, 6),
            crow_fall: strip("sprites/crow_fall_strip2.png", crow, 2, 0, 0, 2),
            crow_land: strip("sprites/crow_land_strip3.png", crow, 3, 0, 0, 3),
            // The strip has 16 frames. We stop at the 10th and hold there.
            crow_die: strip("sprites/crow_die_strip16.png", crow, 16, 0, 0, 10),
            // The owl sheet has every animation for the bird in one long row. The
            // loop we use is frames 32 through 42.
            owl: strip("sprites/horned_owl.png", UVec2::splat(64), 49, 0, 32, 11),
            // Grape frames are 15px wide on a 16px grid, hence the 1px gap.
            grapes: strip(
                "sprites/grapes_spritesheet.png",
                UVec2::new(15, 16),
                5,
                1,
                0,
                5,
            ),
            bush: strip("sprites/bush.png", UVec2::splat(16), 2, 0, 0, 2),

            branch: server.load("sprites/branch.png"),
            forest: server.load("sprites/parallax_forest.png"),
            grape_icon: server.load("sprites/grape_icon.png"),

            menu_music: server.load("audio/music/mystic_forest.ogg"),
            game_music: server.load("audio/music/enchanted_woods.ogg"),
            munch: server.load("audio/effects/munch.ogg"),
        }
    }
}
