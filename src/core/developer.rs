mod path_debug;

use bevy::{pbr::wireframe::WireframeConfig, prelude::*};
use bevy_xpbd_3d::prelude::*;
use oxidized_navigation::debug_draw::DrawNavMesh;

use super::settings::{Settings, SettingsApply};
use path_debug::PathDebugPlugin;

/// Propagates developer settings changes into resources.
pub(super) struct DeveloperPlugin;

impl Plugin for DeveloperPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PathDebugPlugin)
            .add_systems(
                Startup,
                (
                    Self::debug_collisions_toggle_system,
                    Self::wireframe_toggle_system,
                    Self::debug_paths_system,
                ),
            )
            .add_systems(
                PostUpdate,
                (
                    Self::debug_collisions_toggle_system,
                    Self::wireframe_toggle_system,
                    Self::debug_paths_system,
                )
                    .run_if(on_event::<SettingsApply>()),
            );
    }
}

impl DeveloperPlugin {
    fn debug_collisions_toggle_system(
        mut config_store: ResMut<GizmoConfigStore>,
        settings: Res<Settings>,
    ) {
        config_store.config_mut::<PhysicsGizmos>().0.enabled = settings.developer.debug_collisions;
    }

    fn wireframe_toggle_system(
        settings: Res<Settings>,
        mut wireframe_config: ResMut<WireframeConfig>,
    ) {
        wireframe_config.global = settings.developer.wireframe;
    }

    fn debug_paths_system(settings: Res<Settings>, mut draw_nav_mesh: ResMut<DrawNavMesh>) {
        draw_nav_mesh.0 = settings.developer.debug_paths;
    }
}
