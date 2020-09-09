mod components;
mod entities;
mod frame_cnt;
mod frame_limiter;
mod game_events;
mod inventory;
mod keyboard;
mod levels;
mod systems;
mod resources;

use bevy::prelude::*;
use bevy::sprite::TextureAtlas;
use bevy::window;
use structopt::StructOpt;
// use bevy::render::pass::ClearColor;
use frame_cnt::FrameCntPlugin;
use frame_limiter::FrameLimiterPlugin;
use game_events::GameEvent;
use inventory::Inventory;
use keyboard::KeyboardPlugin;
use levels::{LevelInfo, LevelSet, LevelSetLoader};
use resources::DamageMap;
use systems::*;

mod consts {
    pub const MAX_WIDTH: i32 = 31;
    pub const MAX_HEIGHT: i32 = 16;
    pub const FPS: f32 = 30.0;
    pub const KEYFRAME_INTERVAL: usize = 4;
}

mod sounds {
    pub const AMMO: &str = "assets/sounds/ammo.ogg";
    pub const KEY: &str = "assets/sounds/key.ogg";
    pub const SCREW: &str = "assets/sounds/screw.ogg";
    pub const BOMB: &str = "assets/sounds/bomb.ogg";
    pub const WALK: &str = "assets/sounds/walk.ogg";
    pub const TELEPORT: &str = "assets/sounds/teleport.ogg";
    pub const SHOT: &str = "assets/sounds/shot.ogg";
    pub const SPAWN: &str = "assets/sounds/spawn.ogg";
    pub const DOOR: &str = "assets/sounds/door.ogg";
    pub const BURN: &str = "assets/sounds/burn.ogg";
}

use consts::*;

use bevy::asset::AddAsset;

pub struct TextureAtlasHandle(pub Option<Handle<TextureAtlas>>);

#[derive(StructOpt, Debug, Default, Clone)]
#[structopt(name = "basic")]
pub struct Opts {
    #[structopt(short, long)]
    pub benchmark_mode: bool,

    #[structopt(long)]
    pub no_render: bool,

    #[structopt(long)]
    pub no_audio: bool,

    #[structopt(short, long, default_value = "0")]
    pub level: usize,

    #[structopt(long, default_value = "assets/original.txt")]
    pub levelset_path: std::path::PathBuf,

    #[structopt(long, default_value = "1.5")]
    pub zoom: f32,
}

fn main() {
    let opts = Opts::from_args();

    let mut builder = App::build();
    builder
        .add_resource(WindowDescriptor {
            title: "Robbo".to_string(),
            width: ((32 * MAX_WIDTH) as f32 * opts.zoom) as u32,
            height: ((32 * MAX_HEIGHT) as f32 * opts.zoom) as u32,
            vsync: !opts.benchmark_mode,
            resizable: false,
            mode: window::WindowMode::Windowed,
            ..Default::default()
        })
        .add_resource(bevy::render::pass::ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .add_resource(TextureAtlasHandle(None))
        .add_resource(Inventory::default())
        .add_resource(LevelInfo::default())
        .add_resource(DamageMap::default())
        .add_resource(Events::<GameEvent>::default())
        .add_resource(opts.clone())
        .add_system_to_stage(stage::EVENT_UPDATE, update_game_events.system())
        .add_default_plugins()
        .add_asset::<LevelSet>()
        .add_asset_loader::<LevelSet, LevelSetLoader>()
        .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
        // .add_plugin(bevy::diagnostic::PrintDiagnosticsPlugin::default())
        .add_plugin(FrameCntPlugin)
        .add_plugin(KeyboardPlugin)
        .add_startup_system(render_setup.system())
        .add_startup_system(level_setup.system())
        .add_stage_before(stage::UPDATE, "move")
        .add_stage_before(stage::UPDATE, "move_robbo")
        .add_stage_before(stage::POST_UPDATE, "reload_level")
        .add_stage_before(stage::POST_UPDATE, "shots")
        .add_stage_before(stage::POST_UPDATE, "process_damage")
        .add_stage_before(stage::POST_UPDATE, "game_events")
        .add_stage_before(stage::POST_UPDATE, "create_sprites")
        .add_stage_before(stage::POST_UPDATE, "prepare_render")
        .add_stage_after("keyboard", "magnetic_field")
        .add_stage_after("frame_cnt", "tick")
        .add_system_to_stage("magnetic_field", magnetic_field_system.system())
        .add_system_to_stage(stage::EVENT_UPDATE, asset_events.system())
        .add_system_to_stage("process_damage", process_damage.system())
        .add_system_to_stage("move", move_laser_head.system())
        .add_system_to_stage("move", move_bear.system())
        .add_system_to_stage("move", move_bird.system())
        .add_system_to_stage("move", move_box.system())
        .add_system_to_stage("move", move_bullet.system())
        .add_system_to_stage("move", move_blaster_head.system())
        .add_system_to_stage("move", eyes_system.system())
        .add_system_to_stage("move", force_field_system.system())
        .add_system_to_stage("move_robbo", move_robbo.system())
        .add_system_to_stage("shots", shot_system.system())
        .add_system_to_stage("game_events", game_event_system.system())
        .add_system_to_stage("tick", activate_capsule_system.system())
        .add_system_to_stage("tick", tick_system.system())
        .add_system_to_stage("tick", damage_system.system());

    if !opts.benchmark_mode {
        builder
            .add_system_to_stage("reload_level", reload_level.system())
            .add_system_to_stage("create_sprites", create_sprites.system())
            .add_system_to_stage("prepare_render", prepare_render.system())
            .add_plugin(FrameLimiterPlugin { fps: FPS });
    } else {
        builder.add_system_to_stage("reload_level", benchmark_reload_level.system());
        if !opts.no_render {
            builder
                .add_system_to_stage("create_sprites", create_sprites.system())
                .add_system_to_stage("prepare_render", prepare_render.system());
        }
    }
    builder.run();
}
