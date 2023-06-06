use bevy::prelude::*;
use strum::IntoEnumIterator;

use crate::ui2::ui_state::UiState;

pub(super) struct UiRootPlugin;

impl Plugin for UiRootPlugin {
    fn build(&self, app: &mut App) {
        for state in UiState::iter() {
            app.add_system(Self::cleanup_system.in_schedule(OnExit(state)));
        }
    }
}

impl UiRootPlugin {
    fn cleanup_system(mut commands: Commands, roots: Query<Entity, With<UiRoot>>) {
        commands.entity(roots.single()).despawn_recursive();
    }
}

#[derive(Component)]
pub(crate) struct UiRoot;
