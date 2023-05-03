use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{egui::Pos2, EguiContexts};
use bevy_inspector_egui::egui::{Align, Layout};

use crate::core::{
    actor::ActiveActor,
    family::FamilyMode,
    game_state::GameState,
    task::{TaskList, TaskRequest},
};

pub(super) struct TaskMenuPlugin;

impl Plugin for TaskMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            Self::menu_system
                .in_set(OnUpdate(GameState::Family))
                .in_set(OnUpdate(FamilyMode::Life)),
        );
    }
}

impl TaskMenuPlugin {
    fn menu_system(
        mut position: Local<Pos2>,
        mut commands: Commands,
        mut egui: EguiContexts,
        mut task_events: EventWriter<TaskRequest>,
        windows: Query<&Window, With<PrimaryWindow>>,
        mut task_lists: Query<(Entity, &Name, &mut TaskList)>,
        active_actors: Query<Entity, With<ActiveActor>>,
    ) {
        let Ok((entity, name, mut task_list)) = task_lists.get_single_mut() else {
            return;
        };

        if task_list.is_added() {
            // Recalculate window position.
            let primary_window = windows.single();
            let cursor_position = primary_window.cursor_position().unwrap_or_default();
            position.x = cursor_position.x;
            position.y = primary_window.height() - cursor_position.y;
        }

        let mut open = true;
        bevy_egui::egui::Window::new(name.as_str())
            .resizable(false)
            .collapsible(false)
            .fixed_pos(*position)
            .default_width(130.0)
            .open(&mut open)
            .show(egui.ctx_mut(), |ui| {
                ui.with_layout(Layout::top_down_justified(Align::Min), |ui| {
                    let mut clicked_index = None;
                    for (index, task) in task_list.iter().enumerate() {
                        if ui.button(task.name()).clicked() {
                            clicked_index = Some(index);
                        }
                    }
                    if let Some(index) = clicked_index {
                        let task = task_list.swap_remove(index);
                        task_events.send(TaskRequest {
                            entity: active_actors.single(),
                            task,
                        });
                        commands.entity(entity).remove::<TaskList>();
                    }
                });
            });

        if !open {
            commands.entity(entity).remove::<TaskList>();
        }
    }
}
