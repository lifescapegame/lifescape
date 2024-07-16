use bevy::prelude::*;
use bevy_mod_outline::{OutlineBundle, OutlineVolume};

use crate::{core::GameState, game_world::hover::Hovered};

pub(super) struct HighlightingPlugin;

impl Plugin for HighlightingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (Self::enable, Self::disable)
                .run_if(in_state(GameState::City).or_else(in_state(GameState::Family))),
        );
    }
}

impl HighlightingPlugin {
    fn enable(mut hovered: Query<&mut OutlineVolume, Added<Hovered>>) {
        if let Ok(mut outline) = hovered.get_single_mut() {
            debug!("highlighting enabled");
            outline.visible = true;
        }
    }

    fn disable(mut unhovered: RemovedComponents<Hovered>, mut hovered: Query<&mut OutlineVolume>) {
        for entity in unhovered.read() {
            if let Ok(mut outline) = hovered.get_mut(entity) {
                debug!("highlighting disabled");
                outline.visible = false;
            }
        }
    }
}

pub(crate) trait OutlineHighlightingExt {
    fn highlighting() -> Self;
}

impl OutlineHighlightingExt for OutlineBundle {
    fn highlighting() -> Self {
        Self {
            outline: OutlineVolume {
                visible: false,
                colour: Color::rgba(1.0, 1.0, 1.0, 0.3),
                width: 3.0,
            },
            ..Default::default()
        }
    }
}
