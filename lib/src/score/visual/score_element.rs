use crate::score::app_defaults::AppDefaults;
use crate::score::score_defaults::ScoreDefaults;
use crate::score::user_layout::UserLayout;

pub trait ScoreElement {
    fn children(&mut self) -> Vec<&mut dyn ScoreElement> {
        vec![]
    }

    /// do not override.
    fn apply_layout(
        &mut self,
        layout: &ScoreDefaults,
        user_layout: &UserLayout,
        app_defaults: &AppDefaults,
    ) {
        self._apply_layout(layout, user_layout, app_defaults);

        for child in self.children() {
            child.apply_layout(layout, user_layout, app_defaults);
        }
    }

    /// do override.
    fn _apply_layout(
        &mut self,
        _layout: &ScoreDefaults,
        _user_layout: &UserLayout,
        _app_defaults: &AppDefaults,
    ) {
    }
}
