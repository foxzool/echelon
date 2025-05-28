use crate::scene::ScenePlugin;
use bevy::prelude::*;

mod scene;

/// EverShard main plugin
pub struct EverShardPlugin;

impl Plugin for EverShardPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ScenePlugin);
    }
}
