//! A widget for navigating through through a file system. Generally inspired by Finder.
//!
//! Useful for saving widgets that save/load files.
//!
//! - `DirectoryView`: Lists the contents of a single directory. Reacts to events for selection
//! of one or more files, de-selection, deletion and double-clicking.
//! - `FileView`: Displays some basic information about the file.

use {
    color,
    Color,
    Colorable,
    FontSize,
    NodeIndex,
    Positionable,
    Scalar,
    Sizeable,
    Widget,
};
use event;
use std;
use widget;

pub use self::directory_view::DirectoryView;

/// A widget for navigating and interacting with a file system.
pub struct FileNavigator<'a> {
    common: widget::CommonBuilder,
    /// Unique styling for the widget.
    pub style: Style,
    /// The first directory shown for the `FileNavigator`.
    pub starting_directory: &'a std::path::Path,
    /// Only display files of the given type.
    pub types: Types<'a>,
    /// Whether to show hidden files and directories
    show_hidden: bool,
}

/// A type for specifying the types of files to be shown by a `FileNavigator`.
#[derive(Copy, Clone)]
pub enum Types<'a> {
    /// Indicates that files of all types should be shown.
    All,
    /// A list of types of files that are accepted by the `FileNavigator`.
    ///
    /// i.e. `&["wav", "wave", "aiff"]`.
    WithExtension(&'a [&'a str]),
}

/// Unique state stored within the widget graph for each `FileNavigator`.
#[derive(Debug, PartialEq)]
pub struct State {
    /// A canvas upon which we can scroll the `DirectoryView`s horizontally.
    scrollable_canvas_idx: widget::IndexSlot,
    /// Horizontal scrollbar for manually scrolling the canvas.
    scrollbar_idx: widget::IndexSlot,
    /// The starting directory displayed by the `FileNavigator`.
    starting_directory: std::path::PathBuf,
    /// The stack of currently displayed directories.
    ///
    /// Directories are laid out left to right, where the left-most directory is initially the
    /// `starting_directory`.
    directory_stack: Vec<Directory>,
    /// The first `NodeIndex` is stored for the `DirectoryView` for each directory in the stack.
    ///
    /// The second is for the width-resizing `Rectangle`.
    directory_view_indices: Vec<(NodeIndex, NodeIndex)>,
}

/// Represents the state for a single directory.
#[derive(Debug, PartialEq)]
pub struct Directory {
    /// The path of the directory.
    path: std::path::PathBuf,
    /// The width of the `DirectoryView`.
    column_width: Scalar,
}

widget_style!{
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
        /// The default width of a single directory view.
        ///
        /// The first directory will always be initialised to this size.
        - column_width: Scalar { 250.0 }
        /// The width of the bar that separates each directory in the stack and allows for
        /// re-sizing.
        - resize_handle_width: Scalar { 5.0 }
    }
}

/// The kinds of events that the `FileNavigator` may produce.
#[derive(Clone, Debug)]
pub enum Event {
    /// The directory at the top of the stack has changed.
    ChangeDirectory(std::path::PathBuf),
    /// The selection of files in the top of the stack has changed.
    ChangeSelection(Vec<std::path::PathBuf>),
    /// A file was double clicked.
    DoubleClick(Vec<std::path::PathBuf>),
    /// A key was pressed over a selection of entries.
    KeyPress(Vec<std::path::PathBuf>, event::KeyPress),
}

impl<'a> FileNavigator<'a> {

    /// Begin building a `FileNavigator` widget that displays only files of the given types.
    pub fn new(starting_directory: &'a std::path::Path, types: Types<'a>) -> Self {
        FileNavigator {
            common: widget::CommonBuilder::new(),
            style: Style::new(),
            starting_directory: starting_directory,
            types: types,
            show_hidden: false,
        }
    }

    /// Begin building a `FileNavigator` that will display all file types.
    pub fn all(starting_directory: &'a std::path::Path) -> Self {
        Self::new(starting_directory, Types::All)
    }

    /// Begin building a `FileNavigator` that will only display files whose extensions match one
    /// of those within the given extension list.
    ///
    /// i.e. A `FileNavigator` used for navigating lossless audio files might use the following
    /// list of extensions: `&["wav", "wave", "aiff"]`.
    pub fn with_extension(starting_directory: &'a std::path::Path, exts: &'a [&'a str]) -> Self {
        Self::new(starting_directory, Types::WithExtension(exts))
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

    /// Whether to show hidden files and directories. On Windows a hidden file is identified
    /// by a file attribute flag, on other OSs the file name starts with a '.'.
    pub fn show_hidden_files(mut self, show_hidden: bool) -> Self {
        self.show_hidden = show_hidden;
        self
    }

    builder_methods!{
        pub font_size { style.font_size = Some(FontSize) }
    }

}


impl<'a> Widget for FileNavigator<'a> {
    type State = State;
    type Style = Style;
    type Event = Vec<Event>;

    fn common(&self) -> &widget::CommonBuilder {
        &self.common
    }

    fn common_mut(&mut self) -> &mut widget::CommonBuilder {
        &mut self.common
    }

    fn init_state(&self) -> State {
        State {
            scrollable_canvas_idx: widget::IndexSlot::new(),
            scrollbar_idx: widget::IndexSlot::new(),
            directory_stack: Vec::new(),
            directory_view_indices: Vec::new(),
            starting_directory: std::path::PathBuf::new(),
        }
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    /// Update the state of the Button.
    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs { idx, state, style, rect, mut ui, .. } = args;
        let FileNavigator { starting_directory, types, .. } = self;

        if starting_directory != state.starting_directory {
            state.update(|state| {
                let width = style.column_width(&ui.theme);
                let path = starting_directory.to_path_buf();
                state.starting_directory = path.clone();
                state.directory_stack.clear();
                let dir = Directory { path: path, column_width: width };
                state.directory_stack.push(dir);
            });
        }

        let color = style.color(&ui.theme);
        let unselected_color = style.unselected_color(&ui.theme)
            .unwrap_or_else(|| color.plain_contrast().plain_contrast());
        let text_color = style.text_color(&ui.theme)
            .unwrap_or_else(|| color.plain_contrast());

        let scrollable_canvas_idx = state.scrollable_canvas_idx.get(&mut ui);
        widget::Rectangle::fill(rect.dim())
            .xy(rect.xy())
            .color(color::TRANSPARENT)
            .parent(idx)
            .scroll_kids_horizontally()
            .set(scrollable_canvas_idx, &mut ui);

        // A scrollbar for the `FOOTER` canvas.
        let scrollbar_idx = state.scrollbar_idx.get(&mut ui);
        widget::Scrollbar::x_axis(scrollable_canvas_idx)
            .color(color.plain_contrast())
            .auto_hide(true)
            .set(scrollbar_idx, &mut ui);

        // Collect all events that might occur.
        let mut events = Vec::new();

        // Instantiate a view for every directory in the stack.
        let mut i = 0;
        while i < state.directory_stack.len() {

            // Retrive the NodeIndex, or create one if necessary.
            let (view_idx, resize_idx) = match state.directory_view_indices.get(i) {
                Some(&indices) => indices,
                None => {
                    let view_idx = ui.new_unique_node_index();
                    let resize_idx = ui.new_unique_node_index();
                    let new_indices = (view_idx, resize_idx);
                    state.update(|state| state.directory_view_indices.push(new_indices));
                    new_indices
                },
            };

            let resize_handle_width = style.resize_handle_width(&ui.theme);
            let mut column_width = state.directory_stack[i].column_width;

            // Check to see if the resize handle has received any events.
            if let Some(resize_rect) = ui.rect_of(resize_idx) {
                let mut scroll_x = 0.0;
                for drag in ui.widget_input(resize_idx).drags().left() {
                    let target_w = column_width + drag.delta_xy[0];
                    let min_w = resize_rect.w() * 3.0;
                    let end_w = column_width + (rect.right() - resize_rect.right());
                    column_width = min_w.max(target_w);
                    state.update(|state| state.directory_stack[i].column_width = column_width);
                    // If we've dragged the column past end of the rect, scroll it.
                    if target_w > end_w {
                        scroll_x += target_w - end_w;
                    }
                }
                if scroll_x > 0.0 {
                    ui.scroll_widget(scrollable_canvas_idx, [-scroll_x, 0.0]);
                }
            }

            // Instantiate the `DirectoryView` widget and check for events.
            enum Action { EnterDir(std::path::PathBuf), ExitDir }

            let mut maybe_action = None;

            let directory_view_width = column_width - resize_handle_width;
            let font_size = style.font_size(&ui.theme);
            for event in DirectoryView::new(&state.directory_stack[i].path, types)
                .h(rect.h())
                .w(directory_view_width)
                .and(|view| if i == 0 { view.mid_left_of(idx) } else { view.right(0.0) })
                .color(color)
                .unselected_color(unselected_color)
                .text_color(text_color)
                .font_size(font_size)
                .show_hidden_files(self.show_hidden)
                .parent(scrollable_canvas_idx)
                .set(view_idx, &mut ui)
            {
                match event {

                    directory_view::Event::SelectEntry(path) => {
                        if path.is_dir() {
                            maybe_action = Some(Action::EnterDir(path.clone()));
                        } else {
                            maybe_action = Some(Action::ExitDir);
                        }
                        let event = Event::ChangeSelection(vec![path]);
                        events.push(event);
                    },

                    directory_view::Event::SelectEntries(paths) => {
                        maybe_action = Some(Action::ExitDir);
                        let event = Event::ChangeSelection(paths);
                        events.push(event);
                    },

                    directory_view::Event::DoubleClick(path) => {
                        let event = Event::DoubleClick(path);
                        events.push(event);
                    },

                    directory_view::Event::KeyPress(paths, key_press) => {
                        use input;

                        match key_press.key {
                            input::Key::Right => if paths.len() == 1 {
                                if paths[0].is_dir() {
                                    // TODO: Select top child of this dir and give keyboard
                                    // capturing to newly selected child.
                                }
                            },
                            input::Key::Left => {
                                // TODO: Exit top dir, enter parent dir and ensure no children are
                                // selected.
                            },
                            _ => (),
                        }

                        let event = Event::KeyPress(paths, key_press);
                        events.push(event);
                    },

                }
            }



            match maybe_action {

                // If we've entered a directory, clear the stack from this point and add our new
                // directory to the top of the stack.
                Some(Action::EnterDir(path)) => {
                    state.update(|state| {
                        let num_to_remove = state.directory_stack.len() - 1 - i;
                        for _ in 0..num_to_remove {
                            state.directory_stack.pop();
                        }
                        let dir = Directory { path: path.clone(), column_width: column_width };
                        state.directory_stack.push(dir);

                        let event = Event::ChangeDirectory(path);
                        events.push(event);
                    });

                    // If the resulting total width of all `DirectoryView`s would exceed the
                    // width of the `FileNavigator` itself, scroll toward the top-most
                    // `DirectoryView`.
                    let total_w = state.directory_stack.iter().fold(0.0, |t, d| t + d.column_width);
                    let overlap = total_w - rect.w();
                    if overlap > 0.0 {
                        ui.scroll_widget(scrollable_canvas_idx, [-overlap, 0.0]);
                    }
                },

                Some(Action::ExitDir) => {
                    let num_to_remove = state.directory_stack.len() - 1 - i;
                    for _ in 0..num_to_remove {
                        state.update(|state| { state.directory_stack.pop(); });
                    }
                },

                None => (),
            }

            // Instantiate the width-resizing handle's `Rectangle`.
            let resize_color = color.plain_contrast().plain_contrast();
            let resize_color = match ui.widget_input(resize_idx).mouse() {
                Some(mouse) => match mouse.buttons.left().is_down() {
                    true => resize_color.clicked().alpha(0.5),
                    false => resize_color.highlighted().alpha(0.2),
                },
                None => resize_color.alpha(0.2),
            };
            widget::Rectangle::fill([resize_handle_width, rect.h()])
                .color(resize_color)
                .right(0.0)
                .parent(scrollable_canvas_idx)
                .set(resize_idx, &mut ui);

            i += 1;
        }

        // If the canvas is pressed.
        if ui.widget_input(scrollable_canvas_idx).presses().mouse().left().next().is_some() {
            state.update(|state| {
                // Unselect everything.
                while state.directory_stack.len() > 1 {
                    state.directory_stack.pop();
                }
                // TODO: Need to unselect the selected directory here.
            });
        }

        events
    }

}

impl<'a> Colorable for FileNavigator<'a> {
    builder_method!(color { style.color = Some(Color) });
}


/// Lists the contents of a single directory.
///
/// Reacts to events for selection of one or more files, de-selection, deletion and
/// double-clicking.
pub mod directory_view {
    use {
        color,
        Color,
        Colorable,
        FontSize,
        NodeIndex,
        Positionable,
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
        // Whether to show hidden files and directories.
        show_hidden: bool,
    }

    /// Unique state stored within the widget graph for each `FileNavigator`.
    #[derive(Debug, PartialEq)]
    pub struct State {
        scrollable_canvas_idx: widget::IndexSlot,
        scrollbar_idx: widget::IndexSlot,
        /// The absolute path, `Rectangle` and `Text` indices for each file in the directory.
        entries: Vec<Entry>,
        /// A `Text` and `Rectangle` index for each entry.
        ///
        /// Keep this in a separate stack to the `entries` so that we re-use them.
        indices: Vec<(NodeIndex, NodeIndex)>,
        /// The absolute path to the directory.
        directory: std::path::PathBuf,
        /// Keeps track of the indices of each selected entry that has been pressed in order to
        /// perform multi-file selection when `SHIFT` or `ALT` is held.
        last_selected_entries: Vec<usize>,
    }

    /// Data stored for each `File` in the `State`.
    #[derive(Clone, Debug, PartialEq)]
    pub struct Entry {
        path: std::path::PathBuf,
        is_selected: bool,
    }

    widget_style!{
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
        /// A single entry was selected.
        SelectEntry(std::path::PathBuf),
        /// Multiple entries have been selected.
        SelectEntries(Vec<std::path::PathBuf>),
        /// One or more entries have been double clicked.
        DoubleClick(Vec<std::path::PathBuf>),
        /// The given key has been pressed while the given entries were selected.
        KeyPress(Vec<std::path::PathBuf>, event::KeyPress),
    }

    #[cfg(target_os = "windows")]
    /// Check if a file is hidden on windows, using the file attributes.
    fn is_file_hidden(path: &std::path::PathBuf) -> bool {

        use std::os::windows::fs::MetadataExt;
        use std::fs;

        const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;

        let metadata = fs::metadata(&path).ok();
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
            return name.to_string_lossy().to_owned().starts_with(".");
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

        fn init_state(&self) -> Self::State {
            State {
                scrollable_canvas_idx: widget::IndexSlot::new(),
                scrollbar_idx: widget::IndexSlot::new(),
                indices: Vec::new(),
                entries: Vec::new(),
                directory: std::path::PathBuf::new(),
                last_selected_entries: Vec::new(),
            }
        }

        fn style(&self) -> Self::Style {
            self.style.clone()
        }

        fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
            let widget::UpdateArgs { idx, state, style, rect, mut ui, .. } = args;
            let DirectoryView { directory, types, .. } = self;

            if directory != &state.directory {
                state.update(|state| {
                    state.directory = directory.to_path_buf();
                    state.last_selected_entries.clear();
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
                    for (i, entry_path) in entry_paths.enumerate() {
                        // Ensure we have at least as many index pairs as we have entries.
                        if i == state.indices.len() {
                            let rect_idx = ui.new_unique_node_index();
                            let text_idx = ui.new_unique_node_index();
                            state.indices.push((rect_idx, text_idx));
                        }

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

            let scrollable_canvas_idx = state.scrollable_canvas_idx.get(&mut ui);
            widget::Rectangle::fill(rect.dim())
                .scroll_kids_vertically()
                .xy(rect.xy())
                .color(unselected_rect_color.alpha(0.8))
                .parent(idx)
                .set(scrollable_canvas_idx, &mut ui);

            // A scrollbar for the `FOOTER` canvas.
            let scrollbar_idx = state.scrollbar_idx.get(&mut ui);
            widget::Scrollbar::y_axis(scrollable_canvas_idx)
                .color(color.plain_contrast())
                .auto_hide(true)
                .set(scrollbar_idx, &mut ui);

            // Collect any events that have occurred.
            let mut events = Vec::new();

            let mut last_rect_idx = None;
            for i in 0..state.entries.len() {

                let (rect_idx, text_idx) = state.indices[i];
                let (is_selected, is_directory) = {
                    let entry = &state.entries[i];
                    (entry.is_selected, entry.path.is_dir())
                };

                {
                    // Get the file/directory name.
                    let entry_name = state.entries[i].path.file_name()
                        .and_then(|name| name.to_str())
                        .map(|s| {
                            let mut string = s.to_string();
                            if is_directory {
                                string.push('/');
                            }
                            string
                        })
                        .unwrap_or_else(String::new);

                    // Determine the color of this file's `Rectangle`.
                    let rect_color = if is_selected {
                        color
                    } else {
                        match ui.widget_input(rect_idx).mouse() {
                            None => color::TRANSPARENT,
                            Some(mouse) => if mouse.buttons.left().is_down() {
                                color
                            } else {
                                unselected_rect_color.highlighted().alpha(0.5)
                            },
                        }
                    };

                    widget::Rectangle::fill([rect.w(), file_h])
                        .color(rect_color)
                        .and(|r| match last_rect_idx {
                            None => r.mid_top_of(scrollable_canvas_idx),
                            Some(last) => r.down_from(last, 0.0),
                        })
                        .parent(scrollable_canvas_idx)
                        .set(rect_idx, &mut ui);

                    widget::Text::new(&entry_name)
                        .color(text_color)
                        .font_size(font_size)
                        .mid_left_with_margin_on(rect_idx, 10.0)
                        .align_text_left()
                        .graphics_for(rect_idx)
                        .set(text_idx, &mut ui);
                }

                last_rect_idx = Some(rect_idx);

                for widget_event in ui.widget_input(rect_idx).events() {
                    use event;
                    use input::{self, MouseButton};

                    match widget_event {

                        // Check if the entry has been `DoubleClick`ed.
                        event::Widget::DoubleClick(click) => {
                            if let input::MouseButton::Left = click.button {
                                if is_selected {
                                    let paths = state.entries.iter()
                                        .filter(|e| e.is_selected)
                                        .map(|e| e.path.clone())
                                        .collect();
                                    let event = Event::DoubleClick(paths);
                                    events.push(event);
                                }
                            }
                        },

                        // Check for whether or not the file should be selected.
                        event::Widget::Press(press) => match press.button {

                            // Keyboard check whether the selection has been bumped up or down.
                            event::Button::Keyboard(key) => {
                                if let Some(&i) = state.last_selected_entries.last() {
                                    match key {

                                        // Bump the selection up the list.
                                        input::Key::Up => state.update(|state| {
                                            // Clear old selected entries.
                                            state.last_selected_entries.clear();
                                            for entry in &mut state.entries {
                                                entry.is_selected = false;
                                            }

                                            let i = if i == 0 { 0 } else { i - 1 };
                                            state.entries[i].is_selected = true;
                                            state.last_selected_entries.push(i);

                                            let path = state.entries[i].path.clone();
                                            let event = Event::SelectEntry(path);
                                            events.push(event);
                                        }),

                                        // Bump the selection down the list.
                                        input::Key::Down => state.update(|state| {
                                            // Clear old selected entries.
                                            state.last_selected_entries.clear();
                                            for entry in &mut state.entries {
                                                entry.is_selected = false;
                                            }

                                            let num_selected = state.entries.len();
                                            let last_idx = num_selected - 1;
                                            let i = if i < last_idx { i + 1 } else { last_idx };
                                            state.entries[i].is_selected = true;
                                            state.last_selected_entries.push(i);

                                            let path = state.entries[i].path.clone();
                                            let event = Event::SelectEntry(path);
                                            events.push(event);
                                        }),

                                        _ => (),
                                    }

                                    // For any other pressed keys, yield an event along
                                    // with all the paths of all selected entries.
                                    let paths = state.entries.iter()
                                        .filter(|e| e.is_selected)
                                        .map(|e| e.path.clone())
                                        .collect();
                                    let key_press = event::KeyPress {
                                        key: key,
                                        modifiers: press.modifiers,
                                    };
                                    let event = Event::KeyPress(paths, key_press);
                                    events.push(event);
                                }
                            },

                            // Check for a left mouse press.
                            event::Button::Mouse(MouseButton::Left, _) => {
                                let is_shift_down = press.modifiers.contains(input::keyboard::SHIFT);
                                let is_alt_down = press.modifiers.contains(input::keyboard::ALT);

                                match state.last_selected_entries.last() {

                                    // If there is already a currently selected file and shift is
                                    // held, extend the selection to this file.
                                    Some(&idx) if is_shift_down => {
                                        let start_idx_range = std::cmp::min(idx, i);
                                        let end_idx_range = std::cmp::max(idx, i);

                                        state.update(|state| {
                                            // Remove all selected entries other than the last.
                                            while state.last_selected_entries.len() > 1 {
                                                state.last_selected_entries.remove(0);
                                            }

                                            // Set `is_selected` only for the range.
                                            for (i, entry) in state.entries.iter_mut().enumerate() {
                                                if start_idx_range <= i && i <= end_idx_range {
                                                    entry.is_selected = true;
                                                } else {
                                                    entry.is_selected = false;
                                                }
                                            }
                                        });

                                        let paths = state.entries.iter()
                                            .take(end_idx_range + 1)
                                            .skip(start_idx_range)
                                            .map(|e| e.path.clone())
                                            .collect();
                                        let event = Event::SelectEntries(paths);
                                        events.push(event);
                                    },

                                    // If alt is down, additively select or deselect this file.
                                    Some(_) | None if is_alt_down => {
                                        state.update(|state| {
                                            let new_is_selected = !is_selected;
                                            state.entries[i].is_selected = new_is_selected;
                                            if new_is_selected {
                                                state.last_selected_entries.push(i);
                                            }
                                        });

                                        let num_entries_selected = state.entries.iter()
                                            .filter(|e| e.is_selected)
                                            .count();

                                        if num_entries_selected == 0 {
                                            state.update(|state| state.last_selected_entries.clear());
                                        }

                                        // If more than one file, produce a `SelectEntries` event.
                                        if num_entries_selected != 1 {
                                            let paths = state.entries.iter()
                                                .filter_map(|e| {
                                                    if e.is_selected { Some(e.path.clone()) }
                                                    else { None }
                                                })
                                                .collect();
                                            let event = Event::SelectEntries(paths);
                                            events.push(event);

                                        // Otherwise, `SelectEntry`.
                                        } else {
                                            let path = state.entries.iter()
                                                .find(|e| e.is_selected)
                                                .unwrap().path.clone();
                                            let event = Event::SelectEntry(path);
                                            events.push(event);
                                        }
                                    },

                                    // Otherwise if there are no currently selected entries, select
                                    // this file.
                                    _ if !is_selected => {
                                        // Deselect all other selected files.
                                        if !state.last_selected_entries.is_empty() {
                                            state.update(|state| {
                                                state.last_selected_entries.clear();
                                            });
                                        }

                                        for j in 0..state.entries.len() {
                                            if state.entries[j].is_selected {
                                                state.update(|state| {
                                                    state.entries[j].is_selected = false;
                                                });
                                            }
                                        }

                                        // Select the current file.
                                        state.update(|state| {
                                            state.entries[i].is_selected = true;
                                            state.last_selected_entries.push(i);
                                        });
                                        let path = state.entries[i].path.clone();
                                        let event = Event::SelectEntry(path);
                                        events.push(event);
                                    },

                                    _ => (),
                                }

                            },

                            _ => (),
                        },

                        _ => (),
                    }
                }

            }

            // If the scrollable `Rectangle` was pressed, deselect all entries.
            if ui.widget_input(scrollable_canvas_idx).presses().mouse().left().next().is_some() {
                // Deselect all entries.
                state.update(|state| {
                    for entry in &mut state.entries {
                        entry.is_selected = false;
                    }
                    state.last_selected_entries.clear();
                });
                let event = Event::SelectEntries(Vec::new());
                events.push(event);
            }

            events
        }
    }

    impl<'a> Colorable for DirectoryView<'a> {
        builder_method!(color { style.color = Some(Color) });
    }

}


/// Displays some basic information about the file.
pub mod file_view {
}
