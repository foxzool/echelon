use bevy::prelude::*;
use evershard::EverShardPlugin;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EverShardPlugin)
        .run()
}
