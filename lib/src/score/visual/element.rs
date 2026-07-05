use crate::app_defaults::AppDefaults;
use crate::layout::Layout;
use crate::user_layout::UserLayout;

pub trait ScoreElement {
    fn children(&mut self) -> Vec<&mut dyn ScoreElement> {
        vec![]
    }

    fn apply_layout(
        &mut self,
        layout: &Layout,
        user_layout: &UserLayout,
        app_defaults: &AppDefaults,
    ) {
        self._apply_layout(layout, user_layout, app_defaults);

        for child in self.children() {
            child.apply_layout(layout, user_layout, app_defaults);
        }
    }

    fn _apply_layout(
        &mut self,
        _layout: &Layout,
        _user_layout: &UserLayout,
        _app_defaults: &AppDefaults,
    ) {
    }
}
