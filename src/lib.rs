use crate::{character::CharacterPlugin, scene::ScenePlugin};
use avian3d::PhysicsPlugins;
use bevy::prelude::*;

mod character;
mod scene;

/// Echelon main plugin
pub struct EchelonPlugin;

impl Plugin for EchelonPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ScenePlugin, CharacterPlugin, PhysicsPlugins::default()));

        if cfg!(feature = "debug") {
            app.add_plugins(bevy_inspector_egui::bevy_egui::EguiPlugin {
                enable_multipass_for_primary_context: true,
            })
            .add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new());
        }
    }
}
