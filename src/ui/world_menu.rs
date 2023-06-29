use std::{fmt::Display, mem};

use bevy::prelude::*;
use derive_more::Display;
use strum::{EnumIter, IntoEnumIterator};

use super::{
    theme::Theme,
    widget::{
        button::{ExclusiveButton, TabContent, TextButtonBundle, Toggled},
        click::Click,
        text_edit::TextEditBundle,
        ui_root::UiRoot,
        Dialog, DialogBundle, LabelBundle,
    },
};
use crate::core::{
    actor::ActiveActor,
    city::{ActiveCity, City, CityBundle},
    family::{FamilyActors, FamilyDespawn},
    game_state::GameState,
    game_world::WorldName,
};

pub(super) struct WorldMenuPlugin;

impl Plugin for WorldMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(Self::setup_system.in_schedule(OnEnter(GameState::World)))
            .add_systems(
                (
                    Self::family_node_system,
                    Self::city_node_system,
                    Self::family_button_system,
                    Self::city_button_system,
                    Self::create_button_system,
                    Self::city_dialog_button_system,
                )
                    .in_set(OnUpdate(GameState::World)),
            );
    }
}

impl WorldMenuPlugin {
    fn setup_system(
        mut commands: Commands,
        mut tab_commands: Commands,
        theme: Res<Theme>,
        world_name: Res<WorldName>,
        families: Query<(Entity, &Name), With<FamilyActors>>,
        cities: Query<(Entity, &Name), With<City>>,
    ) {
        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        size: Size::all(Val::Percent(100.0)),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::FlexStart,
                        padding: theme.padding.global,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                UiRoot,
            ))
            .with_children(|parent| {
                parent.spawn(LabelBundle::large(&theme, world_name.0.clone()));

                let tabs_entity = parent
                    .spawn(NodeBundle {
                        style: Style {
                            justify_content: JustifyContent::Center,
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .id();

                for (index, tab) in WorldTab::iter().enumerate() {
                    let content_entity = parent
                        .spawn(NodeBundle {
                            style: Style {
                                size: Size::all(Val::Percent(100.0)),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::FlexStart,
                                padding: theme.padding.normal,
                                gap: theme.gap.normal,
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .with_children(|parent| match tab {
                            WorldTab::Families => {
                                for (entity, name) in &families {
                                    setup_entity_node::<FamilyButton>(parent, &theme, entity, name);
                                }
                            }
                            WorldTab::Cities => {
                                for (entity, name) in &cities {
                                    setup_entity_node::<CityButton>(parent, &theme, entity, name);
                                }
                            }
                        })
                        .id();

                    tab_commands
                        .spawn((
                            tab,
                            TabContent(content_entity),
                            ExclusiveButton,
                            Toggled(index == 0),
                            TextButtonBundle::normal(&theme, tab.to_string()),
                        ))
                        .set_parent(tabs_entity);
                }

                parent
                    .spawn(NodeBundle {
                        style: Style {
                            size: Size::new(Val::Percent(100.0), Val::Auto),
                            justify_content: JustifyContent::FlexStart,
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .with_children(|parent| {
                        parent.spawn((
                            CreateEntityButton,
                            TextButtonBundle::normal(&theme, "Create new"),
                        ));
                    });
            });
    }

    fn family_node_system(
        mut commands: Commands,
        theme: Res<Theme>,
        families: Query<(Entity, &Name), Added<FamilyActors>>,
        tabs: Query<(&TabContent, &WorldTab)>,
        buttons: Query<&WorldEntity>,
    ) {
        for (entity, name) in &families {
            let (tab_content, _) = tabs
                .iter()
                .find(|(_, &tab)| tab == WorldTab::Families)
                .expect("tab with families should be spawned on state enter");
            if !buttons.iter().any(|world_entity| world_entity.0 == entity) {
                commands.entity(tab_content.0).with_children(|parent| {
                    setup_entity_node::<FamilyButton>(parent, &theme, entity, name);
                });
            }
        }
    }

    fn city_node_system(
        mut commands: Commands,
        theme: Res<Theme>,
        cities: Query<(Entity, &Name), Added<City>>,
        tabs: Query<(&TabContent, &WorldTab)>,
        buttons: Query<&WorldEntity>,
    ) {
        for (entity, name) in &cities {
            let (tab_content, _) = tabs
                .iter()
                .find(|(_, &tab)| tab == WorldTab::Cities)
                .expect("tab with cities should be spawned on state enter");
            if !buttons.iter().any(|world_entity| world_entity.0 == entity) {
                commands.entity(tab_content.0).with_children(|parent| {
                    setup_entity_node::<CityButton>(parent, &theme, entity, name);
                });
            }
        }
    }

    fn family_button_system(
        mut commands: Commands,
        mut despawn_events: EventWriter<FamilyDespawn>,
        mut click_events: EventReader<Click>,
        mut game_state: ResMut<NextState<GameState>>,
        buttons: Query<(&WorldEntity, &FamilyButton)>,
        families: Query<&FamilyActors>,
    ) {
        for event in &mut click_events {
            if let Ok((world_entity, family_button)) = buttons.get(event.0) {
                match family_button {
                    FamilyButton::Play => {
                        let actors = families
                            .get(world_entity.0)
                            .expect("world entity with family buttons should reference a family");
                        let actor_entity = *actors
                            .first()
                            .expect("family always have at least one member");

                        commands.entity(actor_entity).insert(ActiveActor);
                        game_state.set(GameState::Family);
                    }
                    FamilyButton::Delete => despawn_events.send(FamilyDespawn(world_entity.0)),
                }
            }
        }
    }

    fn city_button_system(
        mut commands: Commands,
        mut click_events: EventReader<Click>,
        mut game_state: ResMut<NextState<GameState>>,
        buttons: Query<(&WorldEntity, &CityButton)>,
    ) {
        for event in &mut click_events {
            if let Ok((world_entity, family_button)) = buttons.get(event.0) {
                // TODO: use event for despawn, otherwise client will despawn the city locally.
                match family_button {
                    CityButton::Edit => {
                        commands.entity(world_entity.0).insert(ActiveCity);
                        game_state.set(GameState::City);
                    }
                    CityButton::Delete => commands.entity(world_entity.0).despawn(),
                }
            }
        }
    }

    fn create_button_system(
        mut commands: Commands,
        mut click_events: EventReader<Click>,
        mut game_state: ResMut<NextState<GameState>>,
        theme: Res<Theme>,
        buttons: Query<(), With<CreateEntityButton>>,
        tabs: Query<(&Toggled, &WorldTab)>,
        roots: Query<Entity, With<UiRoot>>,
    ) {
        for event in &mut click_events {
            if buttons.get(event.0).is_ok() {
                let current_tab = tabs
                    .iter()
                    .find_map(|(toggled, world_tab)| toggled.0.then_some(world_tab))
                    .expect("one tab should always be active");

                match current_tab {
                    WorldTab::Families => game_state.set(GameState::FamilyEditor),
                    WorldTab::Cities => {
                        setup_create_city_dialog(&mut commands, roots.single(), &theme);
                    }
                }
            }
        }
    }

    fn city_dialog_button_system(
        mut commands: Commands,
        mut click_events: EventReader<Click>,
        buttons: Query<&CityDialogButton>,
        mut text_edits: Query<&mut Text, With<CityNameEdit>>,
        dialogs: Query<Entity, With<Dialog>>,
    ) {
        for event in &mut click_events {
            if let Ok(dialog_button) = buttons.get(event.0) {
                if let CityDialogButton::Create = dialog_button {
                    let mut city_name = text_edits.single_mut();
                    commands.spawn(CityBundle::new(
                        mem::take(&mut city_name.sections[0].value).into(),
                    ));
                }
                commands.entity(dialogs.single()).despawn_recursive();
            }
        }
    }
}

fn setup_entity_node<E>(
    parent: &mut ChildBuilder,
    theme: &Theme,
    entity: Entity,
    label: impl Into<String>,
) where
    E: IntoEnumIterator + Clone + Copy + Component + Display,
{
    parent
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(50.0), Val::Percent(25.0)),
                padding: theme.padding.normal,
                ..Default::default()
            },
            background_color: theme.panel_color.into(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::all(Val::Percent(100.0)),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn(LabelBundle::large(theme, label));
                });
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        gap: theme.gap.normal,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with_children(|parent| {
                    for button in E::iter() {
                        parent.spawn((
                            button,
                            WorldEntity(entity),
                            TextButtonBundle::normal(theme, button.to_string()),
                        ));
                    }
                });
        });
}

fn setup_create_city_dialog(commands: &mut Commands, root_entity: Entity, theme: &Theme) {
    commands.entity(root_entity).with_children(|parent| {
        parent
            .spawn(DialogBundle::new(theme))
            .with_children(|parent| {
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            size: Size::new(Val::Percent(50.0), Val::Percent(25.0)),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            padding: theme.padding.normal,
                            gap: theme.gap.normal,
                            ..Default::default()
                        },
                        background_color: theme.panel_color.into(),
                        ..Default::default()
                    })
                    .with_children(|parent| {
                        parent.spawn(LabelBundle::normal(theme, "Create city"));
                        parent.spawn((
                            CityNameEdit,
                            TextEditBundle::new(theme, "New city").active(),
                        ));
                        parent
                            .spawn(NodeBundle {
                                style: Style {
                                    gap: theme.gap.normal,
                                    ..Default::default()
                                },
                                ..Default::default()
                            })
                            .with_children(|parent| {
                                for dialog_button in CityDialogButton::iter() {
                                    parent.spawn((
                                        dialog_button,
                                        TextButtonBundle::normal(theme, dialog_button.to_string()),
                                    ));
                                }
                            });
                    });
            });
    });
}

#[derive(Clone, Component, Copy, Display, EnumIter, PartialEq)]
enum WorldTab {
    Families,
    Cities,
}

#[derive(Component, EnumIter, Clone, Copy, Display)]
enum FamilyButton {
    Play,
    Delete,
}

#[derive(Component, EnumIter, Clone, Copy, Display)]
enum CityButton {
    Edit,
    Delete,
}

/// References family for [`FamilyButton`] or city for [`CityButton`].
#[derive(Component)]
struct WorldEntity(Entity);

/// Button that creates family or city depending on the selected [`WorldTab`].
#[derive(Component)]
struct CreateEntityButton;

#[derive(Component, EnumIter, Clone, Copy, Display)]
enum CityDialogButton {
    Create,
    Cancel,
}

#[derive(Component)]
struct CityNameEdit;
