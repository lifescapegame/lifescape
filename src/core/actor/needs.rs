use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::core::game_world::GameWorld;

pub(super) struct NeedsPlugin;

impl Plugin for NeedsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Hunger>()
            .register_type::<Social>()
            .register_type::<Hygiene>()
            .register_type::<Fun>()
            .register_type::<Energy>()
            .register_type::<Bladder>()
            .register_type::<Need>()
            .replicate::<Hunger>()
            .replicate::<Social>()
            .replicate::<Hygiene>()
            .replicate::<Fun>()
            .replicate::<Energy>()
            .replicate::<Bladder>()
            .replicate::<Need>()
            .add_systems(
                PreUpdate,
                (
                    Self::init_hunger,
                    Self::init_social,
                    Self::init_hygiene,
                    Self::init_fun,
                    Self::init_energy,
                    Self::init_bladder,
                )
                    .after(ClientSet::Receive)
                    .run_if(resource_exists::<GameWorld>),
            )
            .add_systems(
                Update,
                Self::update_values
                    .run_if(on_timer(Duration::from_secs(1)))
                    .run_if(has_authority),
            );
    }
}

impl NeedsPlugin {
    fn init_hunger(mut commands: Commands, needs: Query<Entity, Added<Hunger>>) {
        for entity in &needs {
            commands
                .entity(entity)
                .insert((Name::new("Hunger"), NeedGlyph("🍴"), NeedRate(-0.4)))
                .dont_replicate::<Name>();
        }
    }

    fn init_social(mut commands: Commands, needs: Query<Entity, Added<Social>>) {
        for entity in &needs {
            commands
                .entity(entity)
                .insert((Name::new("Social"), NeedGlyph("💬"), NeedRate(-0.1)))
                .dont_replicate::<Name>();
        }
    }

    fn init_hygiene(mut commands: Commands, needs: Query<Entity, Added<Hygiene>>) {
        for entity in &needs {
            commands
                .entity(entity)
                .insert((Name::new("Hygiene"), NeedGlyph("🚿"), NeedRate(-0.3)))
                .dont_replicate::<Name>();
        }
    }

    fn init_fun(mut commands: Commands, needs: Query<Entity, Added<Fun>>) {
        for entity in &needs {
            commands
                .entity(entity)
                .insert((Name::new("Fun"), NeedGlyph("🎉"), NeedRate(-0.1)))
                .dont_replicate::<Name>();
        }
    }

    fn init_energy(mut commands: Commands, needs: Query<Entity, Added<Energy>>) {
        for entity in &needs {
            commands
                .entity(entity)
                .insert((Name::new("Energy"), NeedGlyph("🔋"), NeedRate(-0.2)))
                .dont_replicate::<Name>();
        }
    }

    fn init_bladder(mut commands: Commands, needs: Query<Entity, Added<Bladder>>) {
        for entity in &needs {
            commands
                .entity(entity)
                .insert((Name::new("Bladder"), NeedGlyph("🚽"), NeedRate(-0.5)))
                .dont_replicate::<Name>();
        }
    }

    fn update_values(mut needs: Query<(&mut Need, &NeedRate)>) {
        for (mut need, rate) in &mut needs {
            if need.0 > rate.0 {
                need.0 += rate.0;
            } else {
                need.0 = 0.0;
            }
        }
    }
}

#[derive(Bundle)]
pub(crate) struct NeedBundle<T: Component> {
    need: Need,
    marker: T,
    parent_sync: ParentSync,
    replication: Replication,
}

impl<T: Component + Default> Default for NeedBundle<T> {
    fn default() -> Self {
        Self {
            need: Default::default(),
            marker: T::default(),
            parent_sync: Default::default(),
            replication: Replication,
        }
    }
}

#[derive(Component, Default, Deserialize, Reflect, Serialize)]
#[reflect(Component)]
pub(crate) struct Hunger;

#[derive(Component, Default, Deserialize, Reflect, Serialize)]
#[reflect(Component)]
pub(crate) struct Social;

#[derive(Component, Default, Deserialize, Reflect, Serialize)]
#[reflect(Component)]
pub(crate) struct Hygiene;

#[derive(Component, Default, Deserialize, Reflect, Serialize)]
#[reflect(Component)]
pub(crate) struct Fun;

#[derive(Component, Default, Deserialize, Reflect, Serialize)]
#[reflect(Component)]
pub(crate) struct Energy;

#[derive(Component, Default, Deserialize, Reflect, Serialize)]
#[reflect(Component)]
pub(crate) struct Bladder;

#[derive(Component, Deserialize, Reflect, Serialize)]
#[reflect(Component)]
pub(crate) struct Need(pub(crate) f32);

impl Default for Need {
    fn default() -> Self {
        Self(100.0)
    }
}

#[derive(Component)]
struct NeedRate(f32);

#[derive(Component)]
pub(crate) struct NeedGlyph(pub(crate) &'static str);
