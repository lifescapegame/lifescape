use std::{fs, path::Path};

use anyhow::{Context, Result};
use bevy::prelude::*;
use iyes_loopless::prelude::*;
use leafwing_input_manager::prelude::*;
use serde::{Deserialize, Serialize};

use super::{control_action::ControlAction, errors::log_err_system, game_paths::GamePaths};

pub(super) struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        let game_paths = app.world.resource::<GamePaths>();

        app.insert_resource(Settings::read(&game_paths.settings).unwrap_or_default())
            .add_event::<SettingsApplied>()
            .add_system(
                Self::write_system
                    .chain(log_err_system)
                    .run_on_event::<SettingsApplied>(),
            );
    }
}

impl SettingsPlugin {
    fn write_system(settings: Res<Settings>, game_paths: Res<GamePaths>) -> Result<()> {
        settings.write(&game_paths.settings)
    }
}

/// An event that applies the specified settings in the [`Settings`] resource.
#[derive(Default)]
pub(crate) struct SettingsApplied;

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub(crate) struct Settings {
    pub(crate) video: VideoSettings,
    // TODO: TOML implementations have issues with [`HashSet`]:
    // https://github.com/alexcrichton/toml-rs/issues/469 and https://github.com/ordian/toml_edit/issues/319
    #[serde(skip)]
    pub(crate) controls: ControlsSettings,
    pub(crate) developer: DeveloperSettings,
}

impl Settings {
    /// Creates [`Settings`] from the application settings file.
    /// Will be initialed with defaults if the file does not exist.
    fn read(file_name: &Path) -> Result<Settings> {
        match fs::read_to_string(file_name) {
            Ok(content) => toml::from_str::<Settings>(&content)
                .with_context(|| format!("Unable to read settings from {file_name:?}")),
            Err(_) => Ok(Settings::default()),
        }
    }

    /// Saves settings on disk under.
    ///
    /// Automatically creates all parent folders.
    fn write(&self, file_name: &Path) -> Result<()> {
        let content = toml::to_string_pretty(&self).context("Unable to serialize settings")?;

        let parent_folder = file_name
            .parent()
            .unwrap_or_else(|| panic!("Unable to get settings directory from {file_name:?}"));

        fs::create_dir_all(&parent_folder)
            .with_context(|| format!("Unable to create {parent_folder:?}"))?;

        fs::write(file_name, content)
            .with_context(|| format!("Unable to write settings to {file_name:?}"))
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub(crate) struct VideoSettings {
    pub(crate) msaa: u32,
    pub(crate) perf_stats: bool,
}

impl Default for VideoSettings {
    fn default() -> Self {
        Self {
            msaa: 1,
            perf_stats: false,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub(crate) struct ControlsSettings {
    pub(crate) mappings: InputMap<ControlAction>,
}

impl Default for ControlsSettings {
    fn default() -> Self {
        let mut input = InputMap::default();
        input
            .insert(KeyCode::W, ControlAction::CameraForward)
            .insert(KeyCode::S, ControlAction::CameraBackward)
            .insert(KeyCode::A, ControlAction::CameraLeft)
            .insert(KeyCode::D, ControlAction::CameraRight)
            .insert(KeyCode::Up, ControlAction::CameraForward)
            .insert(KeyCode::Down, ControlAction::CameraBackward)
            .insert(KeyCode::Left, ControlAction::CameraLeft)
            .insert(KeyCode::Right, ControlAction::CameraRight)
            .insert(MouseButton::Right, ControlAction::RotateCamera)
            .insert(SingleAxis::mouse_wheel_y(), ControlAction::ZoomCamera);

        Self { mappings: input }
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub(crate) struct DeveloperSettings {
    pub(crate) world_inspector: bool,
    pub(crate) debug_collisions: bool,
}

#[cfg(test)]
mod tests {
    use bevy::ecs::event::Events;

    use super::*;

    #[test]
    fn loading() {
        let mut app = App::new();
        app.init_resource::<GamePaths>().add_plugin(SettingsPlugin);

        let game_paths = app.world.resource::<GamePaths>();
        assert!(!game_paths.settings.exists());

        let settings = app.world.resource::<Settings>();
        assert_eq!(*settings, Settings::default());
    }

    #[test]
    fn applying() -> Result<()> {
        let mut app = App::new();
        app.init_resource::<GamePaths>().add_plugin(SettingsPlugin);

        // Modify settings
        let mut settings = app.world.resource_mut::<Settings>();
        settings.video.msaa += 1;

        let mut apply_events = app.world.resource_mut::<Events<SettingsApplied>>();
        apply_events.send_default();

        app.update();

        let game_paths = app.world.resource::<GamePaths>();
        assert!(game_paths.settings.exists());

        let loaded_settings = Settings::read(&game_paths.settings)?;
        let settings = app.world.resource::<Settings>();
        assert_eq!(*settings, loaded_settings);

        Ok(())
    }
}
