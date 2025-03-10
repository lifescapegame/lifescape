pub mod button;
pub mod checkbox;
pub mod dialog;
pub mod label;
pub mod popup;
pub mod progress_bar;
pub mod text_edit;
pub mod theme;

use bevy::prelude::*;

use button::ButtonPlugin;
use checkbox::CheckboxPlugin;
use dialog::DialogPlugin;
use label::LabelPlugin;
use popup::PopupPlugin;
use progress_bar::ProgressBarPlugin;
use text_edit::TextEditPlugin;
use theme::ThemePlugin;

pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ButtonPlugin,
            DialogPlugin,
            LabelPlugin,
            CheckboxPlugin,
            PopupPlugin,
            ProgressBarPlugin,
            TextEditPlugin,
            ThemePlugin,
        ));
    }
}
