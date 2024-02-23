pub(super) mod human;
mod movement_animation;
pub(crate) mod needs;
pub(crate) mod task;

use bevy::{
    prelude::*,
    scene::{self, SceneInstanceReady},
};
use bevy_mod_outline::{InheritOutlineBundle, OutlineBundle};
use bevy_replicon::prelude::*;
use bevy_xpbd_3d::prelude::*;
use num_enum::IntoPrimitive;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

use super::{
    asset::collection::{AssetCollection, Collection},
    game_state::GameState,
    game_world::WorldName,
    highlighting::OutlineHighlightingExt,
};
use crate::core::{
    animation_state::AnimationState, cursor_hover::CursorHoverable, navigation::NavigationBundle,
};
use human::HumanPlugin;
use movement_animation::MovementAnimationPlugin;
use needs::NeedsPlugin;
use task::TaskPlugin;

pub(super) struct ActorPlugin;

impl Plugin for ActorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Collection<ActorAnimation>>()
            .add_plugins((
                MovementAnimationPlugin,
                NeedsPlugin,
                HumanPlugin,
                TaskPlugin,
            ))
            .register_type::<Actor>()
            .register_type::<FirstName>()
            .register_type::<Sex>()
            .register_type::<LastName>()
            .replicate::<Actor>()
            .replicate::<FirstName>()
            .replicate::<Sex>()
            .replicate::<LastName>()
            .add_systems(OnExit(GameState::Family), Self::deactivation_system)
            .add_systems(
                PreUpdate,
                Self::init_system
                    .after(ClientSet::Receive)
                    .run_if(resource_exists::<WorldName>),
            )
            .add_systems(
                Update,
                Self::name_update_system.run_if(resource_exists::<WorldName>),
            )
            .add_systems(
                SpawnScene,
                Self::scene_init_system
                    .run_if(resource_exists::<WorldName>)
                    .after(scene::scene_spawner_system),
            )
            .add_systems(PostUpdate, Self::exclusive_system);
    }
}

impl ActorPlugin {
    fn init_system(
        mut commands: Commands,
        actor_animations: Res<Collection<ActorAnimation>>,
        actors: Query<Entity, Added<Actor>>,
    ) {
        for entity in &actors {
            const HEIGHT: f32 = 1.2;
            const RADIUS: f32 = 0.3;
            commands
                .entity(entity)
                .insert((
                    AnimationState::new(actor_animations.handle(ActorAnimation::Idle)),
                    VisibilityBundle::default(),
                    GlobalTransform::default(),
                    OutlineBundle::highlighting(),
                    NavigationBundle::default(), // TODO: Serialize it as part of actor bundle.
                    CursorHoverable,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        SpatialBundle::from_transform(Transform::from_translation(
                            Vec3::Y * (HEIGHT / 2.0 + RADIUS),
                        )),
                        Collider::capsule(HEIGHT, RADIUS),
                    ));
                });
        }
    }

    fn scene_init_system(
        mut commands: Commands,
        mut ready_events: EventReader<SceneInstanceReady>,
        actors: Query<Entity, With<Actor>>,
        chidlren: Query<&Children>,
    ) {
        for actor_entity in actors.iter_many(ready_events.read().map(|event| event.parent)) {
            for child_entity in chidlren.iter_descendants(actor_entity) {
                commands
                    .entity(child_entity)
                    .insert(InheritOutlineBundle::default());
            }
        }
    }

    fn name_update_system(
        mut commands: Commands,
        mut changed_names: Query<
            (Entity, Ref<FirstName>, Ref<LastName>),
            Or<(Changed<FirstName>, Changed<LastName>)>,
        >,
    ) {
        for (entity, first_name, last_name) in &mut changed_names {
            let mut entity = commands.entity(entity);
            entity.insert(Name::new(format!("{} {}", first_name.0, last_name.0)));
            if first_name.is_added() && last_name.is_added() {
                entity.dont_replicate::<Name>();
            }
        }
    }

    fn deactivation_system(mut commands: Commands, actors: Query<Entity, With<ActiveActor>>) {
        if let Ok(entity) = actors.get_single() {
            commands.entity(entity).remove::<ActiveActor>();
        }
    }

    fn exclusive_system(
        mut commands: Commands,
        activated_actors: Query<Entity, Added<ActiveActor>>,
        actors: Query<Entity, With<ActiveActor>>,
    ) {
        if let Some(activated_entity) = activated_actors.iter().last() {
            for actor_entity in actors.iter().filter(|&entity| entity != activated_entity) {
                commands.entity(actor_entity).remove::<ActiveActor>();
            }
        }
    }
}

#[derive(Clone, Component, Default, Deref, Deserialize, Reflect, Serialize)]
#[reflect(Component)]
pub(crate) struct FirstName(pub(crate) String);

#[derive(Clone, Component, Default, Deref, Deserialize, Reflect, Serialize)]
#[reflect(Component)]
pub(crate) struct LastName(pub(crate) String);

#[derive(
    Display, Clone, EnumIter, Component, Copy, Default, Deserialize, PartialEq, Reflect, Serialize,
)]
#[reflect(Component)]
pub(crate) enum Sex {
    #[default]
    Male,
    Female,
}

/// Indicates locally controlled actor.
#[derive(Component)]
pub(crate) struct ActiveActor;

/// Marks entity as an actor.
#[derive(Component, Default, Deserialize, Reflect, Serialize)]
#[reflect(Component)]
pub(crate) struct Actor;

#[reflect_trait]
pub(crate) trait ActorBundle: Reflect {
    fn glyph(&self) -> &'static str;
}

#[derive(Clone, Copy, EnumIter, IntoPrimitive)]
#[repr(usize)]
pub(super) enum ActorAnimation {
    Idle,
    MaleWalk,
    FemaleWalk,
    MaleRun,
    FemaleRun,
    TellSecret,
    ThoughtfulNod,
}

impl AssetCollection for ActorAnimation {
    type AssetType = AnimationClip;

    fn asset_path(&self) -> &'static str {
        match self {
            ActorAnimation::Idle => "base/actors/animations/idle.gltf#Animation0",
            ActorAnimation::MaleWalk => "base/actors/animations/male_walk.gltf#Animation0",
            ActorAnimation::FemaleWalk => "base/actors/animations/female_walk.gltf#Animation0",
            ActorAnimation::MaleRun => "base/actors/animations/male_run.gltf#Animation0",
            ActorAnimation::FemaleRun => "base/actors/animations/female_run.gltf#Animation0",
            ActorAnimation::TellSecret => "base/actors/animations/tell_secret.gltf#Animation0",
            ActorAnimation::ThoughtfulNod => {
                "base/actors/animations/thoughtful_nod.gltf#Animation0"
            }
        }
    }
}
