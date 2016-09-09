//! Lists the contents of a single directory.
//!
//! Reacts to events for selection of one or more files, de-selection, deletion and
//! double-clicking.

use {
    Borderable,
    color,
    Color,
    Colorable,
    FontSize,
    Labelable,
    Positionable,
    Sizeable,
    Scalar,
    Widget,
};
use event;
use std;
use widget;

/// For viewing, selecting, double-clicking, etc the contents of a directory.
pub struct DirectoryView<'a> {
    common: widget::CommonBuilder,
    /// Unique styling for the widget.
    pub style: Style,
    /// The path of the directory to display.
    pub directory: &'a std::path::Path,
    /// Only display files of the given type.
    pub types: super::Types<'a>,
    // Whether or not hidden files and directories will be shown.
    show_hidden: bool,
}

/// Unique state stored within the widget graph for each `FileNavigator`.
pub struct State {
    /// The absolute path, `Rectangle` and `Text` indices for each file in the directory.
    entries: Vec<Entry>,
    /// The absolute path to the directory.
    directory: std::path::PathBuf,
    /// The `DirectoryView`'s children widgets:
    /// 
    /// - The background color for the directory view.
    /// - The index used to instantiate the `ListSelect` widget.
    ids: Ids,
}

/// Data stored for each `File` in the `State`.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    path: std::path::PathBuf,
    is_selected: bool,
}

widget_ids! {
    struct Ids {
        rectangle,
        list_select,
    }
}

widget_style! {
    /// Unique styling for the widget.
    style Style {
        /// Color of the selected entries.
        - color: Color { theme.shape_color }
        /// The color of the unselected entries.
        - unselected_color: Option<Color> { None }
        /// The color of the directory and file names.
        - text_color: Option<Color> { None }
        /// The font size for the directory and file names.
        - font_size: FontSize { theme.font_size_medium }
    }
}

/// The kinds of `Event`s produced by the `DirectoryView`.
#[derive(Clone)]
pub enum Event {
    /// Some change in the `Selection` occurred. This represents the new full selection.
    Selection(Vec<std::path::PathBuf>),
    /// One or more entries have been double clicked.
    Click(event::Click, Vec<std::path::PathBuf>),
    /// One or more entries have been double clicked.
    DoubleClick(event::DoubleClick, Vec<std::path::PathBuf>),
    /// A `Press` event occurred while the given entries were selected.
    Press(event::Press, Vec<std::path::PathBuf>),
    /// A `Release` event occurred while the given entries were selected.
    Release(event::Release, Vec<std::path::PathBuf>),
}

#[cfg(all(target_os = "windows", not(feature = "windows_metadataext")))]
fn is_file_hidden(_path: &std::path::PathBuf) -> bool {
    false
}
#[cfg(all(target_os = "windows", feature = "windows_metadataext"))]
/// Check if a file is hidden on windows, using the file attributes.
/// To be enabled once windows::fs::MetadataExt is no longer an unstable API.
fn is_file_hidden(path: &std::path::PathBuf) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;

    let metadata = std::fs::metadata(&path).ok();
    if let Some(metadata) = metadata {
        let win_attr: u32 = metadata.file_attributes();
        return (win_attr & FILE_ATTRIBUTE_HIDDEN) != 0;
    }
    false
}

#[cfg(not(target_os = "windows"))]
/// Check if a file is hidden on any other OS than windows, using the dot file namings.
fn is_file_hidden(path: &std::path::PathBuf) -> bool {
    let name = path.file_name();
    if let Some(name) = name {
        return name.to_string_lossy().starts_with(".");
    }
    false
}

/// Returns true if file or directory should be displayed depending on configuration
/// and file status (hidden or not)
fn check_hidden(show_hidden: bool, path: &std::path::PathBuf) -> bool {
    show_hidden | (!is_file_hidden(path))
}

impl<'a> DirectoryView<'a> {

    /// Begin building a `DirectoryNavigator` widget that displays only files of the given types.
    pub fn new(directory: &'a std::path::Path, types: super::Types<'a>) -> Self {
        DirectoryView {
            common: widget::CommonBuilder::new(),
            style: Style::new(),
            directory: directory,
            types: types,
            show_hidden: false,
        }
    }

    /// The color of the unselected entries within each `DirectoryView`.
    pub fn unselected_color(mut self, color: Color) -> Self {
        self.style.unselected_color = Some(Some(color));
        self
    }

    /// The color of the `Text` used to display the file names.
    pub fn text_color(mut self, color: Color) -> Self {
        self.style.text_color = Some(Some(color));
        self
    }

    /// Whether to show hidden files and directories
    pub fn show_hidden_files(mut self, show_hidden: bool) -> Self {
        self.show_hidden = show_hidden;
        self
    }

    builder_methods!{
        pub font_size { style.font_size = Some(FontSize) }
    }

}

impl<'a> Widget for DirectoryView<'a> {
    type State = State;
    type Style = Style;
    type Event = Vec<Event>;

    fn common(&self) -> &widget::CommonBuilder {
        &self.common
    }

    fn common_mut(&mut self) -> &mut widget::CommonBuilder {
        &mut self.common
    }

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State {
            entries: Vec::new(),
            directory: std::path::PathBuf::new(),
            ids: Ids::new(id_gen),
        }
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs { id, state, style, rect, mut ui, .. } = args;
        let DirectoryView { directory, types, .. } = self;

        if directory != &state.directory {
            state.update(|state| {
                state.directory = directory.to_path_buf();
                state.entries.clear();
            });

            let show_hidden = self.show_hidden;
            let entries: Vec<_> = match std::fs::read_dir(directory).ok() {
                Some(entries) => {
                    entries.filter_map(|e| e.ok())
                        .filter(|f| check_hidden(show_hidden, &f.path()))
                        .collect()
                }
                None => return Vec::new(),
            };

            // Create an iterator yielding the path for each directory.
            let directory_paths = entries.iter()
                .map(|e| e.path())
                .filter_map(|path| if path.is_dir() { Some(path) } else { None });

            // And now paths for the relevant files.
            let file_paths = entries.iter()
                .map(|e| e.path())
                .filter_map(|path| match types {
                    super::Types::All => Some(path),
                    super::Types::WithExtension(valid_exts) => {
                        // We're only after files.
                        if path.is_dir() {
                            return None;
                        }
                        // Check for valid extensions.
                        let ext = path.extension()
                            .and_then(|ext| ext.to_str())
                            .map(|s| std::ascii::AsciiExt::to_ascii_lowercase(s))
                            .unwrap_or_else(String::new);
                        if valid_exts.iter().any(|&valid_ext| &ext == valid_ext) {
                            Some(path)
                        } else {
                            None
                        }
                    },
                });

            // Chain them in order of directories and then files.
            let entry_paths = directory_paths.chain(file_paths);

            state.update(|state| {
                for entry_path in entry_paths {
                    let entry = Entry {
                        path: entry_path.to_path_buf(),
                        is_selected: false,
                    };
                    state.entries.push(entry);
                }
            });
        }

        let color = style.color(&ui.theme);
        let font_size = style.font_size(&ui.theme);
        let file_h = font_size as Scalar * 2.0;
        let unselected_rect_color = style.unselected_color(&ui.theme)
            .unwrap_or_else(|| color.plain_contrast().plain_contrast());
        let text_color = style.text_color(&ui.theme)
            .unwrap_or_else(|| color.plain_contrast());

        // Color the background of the directory view.
        widget::Rectangle::fill(rect.dim())
            .color(unselected_rect_color)
            .xy(rect.xy())
            .parent(id)
            .graphics_for(id)
            .set(state.ids.rectangle, ui);

        // Collect any events that have occurred.
        let mut events = Vec::new();

        let list_h = rect.h().min(state.entries.len() as Scalar * file_h);
        let (mut list_events, scrollbar) =
            widget::ListSelect::multiple(state.entries.len(), file_h)
                .scrollbar_on_top()
                .w_h(rect.w(), list_h)
                .mid_top_of(id)
                .set(state.ids.list_select, ui);

        // A helper method for collecting all selected entries.
        let collect_selected = |entries: &[Entry]| entries.iter()
            .filter_map(|e| if e.is_selected { Some(e.path.clone()) } else { None })
            .collect();

        while let Some(event) = list_events.next(ui, |i| state.entries[i].is_selected) {
            use widget::list_select;

            match event {

                // Instantiate a `Button` for each item.
                list_select::Event::Item(item) => {
                    let entry = &state.entries[item.i];
                    let is_selected = entry.is_selected;
                    let is_directory = entry.path.is_dir();

                    // Get the file/directory name.
                    let entry_name = state.entries[item.i].path.file_name()
                        .and_then(|name| name.to_str())
                        .map_or_else(String::new, |s| {
                            let mut string = s.to_string();
                            if is_directory {
                                string.push('/');
                            }
                            string
                        });

                    // Determine the color of this file's `Rectangle`.
                    let rect_color = if is_selected {
                        color
                    } else {
                        match ui.widget_input(item.widget_id).mouse() {
                            None => color::TRANSPARENT,
                            Some(_) => unselected_rect_color,
                        }
                    };

                    let button = widget::Button::new()
                        .border(0.0)
                        .color(rect_color)
                        .label(&entry_name)
                        .label_color(text_color)
                        .label_font_size(font_size)
                        .align_label_left();
                    item.set(button, ui);
                },

                // Update the state's selection.
                list_select::Event::Selection(selection) => {
                    match selection {
                        list_select::Selection::Add(indices) =>
                            state.update(|state| for i in indices {
                                state.entries[i].is_selected = true;
                            }),
                        list_select::Selection::Remove(indices) =>
                            state.update(|state| for i in indices {
                                state.entries[i].is_selected = false;
                            }),
                    }
                    events.push(Event::Selection(collect_selected(&state.entries)));
                },

                // Propagate the interaction events.
                list_select::Event::Click(e) =>
                    events.push(Event::Click(e, collect_selected(&state.entries))),
                list_select::Event::DoubleClick(e) =>
                    events.push(Event::DoubleClick(e, collect_selected(&state.entries))),
                list_select::Event::Press(e) =>
                    events.push(Event::Press(e, collect_selected(&state.entries))),
                list_select::Event::Release(e) =>
                    events.push(Event::Release(e, collect_selected(&state.entries))),
            }
        }

        if let Some(s) = scrollbar { s.set(ui); }

        // If the scrollable `Rectangle` was pressed, deselect all entries.
        if ui.widget_input(id).presses().mouse().left().next().is_some() {
            // Deselect all entries.
            state.update(|state| for entry in &mut state.entries {
                entry.is_selected = false;
            });
            events.push(Event::Selection(Vec::new()));
        }

        events
    }
}

impl<'a> Colorable for DirectoryView<'a> {
    builder_method!(color { style.color = Some(Color) });
}
