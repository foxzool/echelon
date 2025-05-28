use crate::{character::CharacterPlugin, scene::ScenePlugin};
use avian3d::PhysicsPlugins;
use bevy::prelude::*;

mod character;
mod scene;

/// EverShard main plugin
pub struct EverShardPlugin;

impl Plugin for EverShardPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ScenePlugin, CharacterPlugin, PhysicsPlugins::default()));
    }
}
