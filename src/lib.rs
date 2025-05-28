use crate::{character::CharacterPlugin, scene::ScenePlugin};
use bevy::prelude::*;

mod character;
mod scene;

/// EverShard main plugin
pub struct EverShardPlugin;

impl Plugin for EverShardPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ScenePlugin).add_plugins(CharacterPlugin);
    }
}
