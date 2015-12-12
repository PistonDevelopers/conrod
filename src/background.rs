
use CharacterCache;
use color::{Color, Colorable};
use ui::{self, Ui};

/// A type for drawing a colored window background.
#[derive(Copy, Clone)]
pub struct Background {
    maybe_color: Option<Color>,
}

impl Background {

    /// Construct a background.
    pub fn new() -> Background {
        Background {
            maybe_color: None,
        }
    }

    /// Set the color used clear the background with before drawing widgets.
    pub fn set<C>(&mut self, ui: &mut Ui<C>) where C: CharacterCache {
        let color = self.maybe_color.unwrap_or(ui.theme.background_color);
        ui::clear_with(ui, color);
    }

}

impl Colorable for Background {
    fn color(mut self, color: Color) -> Self {
        self.maybe_color = Some(color);
        self
    }
}

