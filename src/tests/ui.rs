use events::InputProvider;
use events::ui_event::UiEvent;
use {
    Theme,
    CharacterCache,
    Labelable,
    Canvas,
    Color,
    Positionable,
    Colorable,
    Sizeable,
    Widget
};
use input::{Input, Motion, Button, self};
use input::keyboard::Key;
use input::mouse::MouseButton;
use graphics::ImageSize;
use graphics::character::Character;
use graphics::types::FontSize;
use widget::{Index, self};
use widget::button::Button as ButtonWidget;
use position::Point;

type Ui = ::Ui<MockCharacterCache>;

#[test]
fn ui_should_reset_global_input_after_widget_are_set() {
    let mut ui = windowless_ui();
    ui.win_w = 250.0;
    ui.win_h = 300.0;

    const CANVAS_ID: widget::Id = widget::Id(0);
    const BUTTON_ID: widget::Id = widget::Id(1);

    move_mouse_to_widget(Index::Public(BUTTON_ID), &mut ui);
    left_click_mouse(&mut ui);

    assert!(ui.global_input.all_events().next().is_some());
    ui.set_widgets(|ref mut ui| {

        Canvas::new()
            .color(Color::Rgba(1.0, 1.0, 1.0, 1.0))
            .set(CANVAS_ID, ui);
        ButtonWidget::new()
            .w_h(100.0, 200.0)
            .label("MyButton")
            .react(|| {})
            .bottom_right_of(CANVAS_ID)
            .set(BUTTON_ID, ui);
    });

    assert!(ui.global_input.all_events().next().is_none());
}

#[test]
fn ui_should_push_capturing_event_when_mouse_button_is_pressed_over_a_widget() {
    let mut ui = windowless_ui();
    ui.win_w = 250.0;
    ui.win_h = 300.0;

    const CANVAS_ID: widget::Id = widget::Id(0);
    const BUTTON_ID: widget::Id = widget::Id(1);
    ui.set_widgets(|ref mut ui| {

        Canvas::new()
            .color(Color::Rgba(1.0, 1.0, 1.0, 1.0))
            .set(CANVAS_ID, ui);
        ButtonWidget::new()
            .w_h(100.0, 200.0)
            .label("MyButton")
            .react(|| {})
            .bottom_right_of(CANVAS_ID)
            .set(BUTTON_ID, ui);
    });

    let button_idx = Index::Public(BUTTON_ID);
    move_mouse_to_widget(button_idx, &mut ui);
    press_mouse_button(MouseButton::Left, &mut ui);

    let expected_capture_event = UiEvent::WidgetCapturesKeyboard(button_idx);
    assert_event_was_pushed(&ui, expected_capture_event);

    // Now click somewhere on the background and widget should uncapture
    release_mouse_button(MouseButton::Left, &mut ui);
    move_mouse_to_abs_coordinates(1.0, 1.0, &mut ui);
    press_mouse_button(MouseButton::Left, &mut ui);

    let expected_uncapture_event = UiEvent::WidgetUncapturesKeyboard(button_idx);
    assert_event_was_pushed(&ui, expected_uncapture_event);
}

#[test]
fn ui_should_convert_mouse_cursor_event_into_mouse_relative_event() {
    let mut ui = windowless_ui();
    ui.win_w = 150.0;
    ui.win_h = 200.0;

    // MouseCursor event contains location in window coordinates, which
    // use the upper left corner as the origin.
    ui.handle_event(&Input::Move(Motion::MouseCursor(5.0, 140.0)));

    // MouseRelative events contain location coordinates where the center of the window is the origin.
    let expected_relative_event = UiEvent::Raw(
        Input::Move(Motion::MouseRelative(-70.0, -40.0))
    );
    assert_event_was_pushed(&ui, expected_relative_event);
}

#[test]
fn ui_should_push_input_events_to_aggregator() {
    let mut ui = windowless_ui();

    test_handling_basic_input_event(&mut ui, Input::Press(input::Button::Keyboard(Key::LCtrl)));
    test_handling_basic_input_event(&mut ui, Input::Release(input::Button::Keyboard(Key::LCtrl)));
    test_handling_basic_input_event(&mut ui, Input::Text("my string".to_string()));
    test_handling_basic_input_event(&mut ui, Input::Resize(55, 99));
    test_handling_basic_input_event(&mut ui, Input::Focus(true));
    test_handling_basic_input_event(&mut ui, Input::Cursor(true));
}

type Ui = ::Ui<MockBackend>;

fn left_click_mouse(ui: &mut Ui) {
    press_mouse_button(MouseButton::Left, ui);
    release_mouse_button(MouseButton::Left, ui);
}

fn release_mouse_button(button: MouseButton, ui: &mut Ui) {
    let event = Input::Release(Button::Mouse(button));
    ui.handle_event(&event);
}

fn press_mouse_button(button: MouseButton, ui: &mut Ui) {
    let event = Input::Press(Button::Mouse(button));
    ui.handle_event(&event);
}

fn move_mouse_to_widget(widget_idx: Index, ui: &mut Ui) {
    ui.xy_of(widget_idx).map(|point| {
        let abs_xy = to_window_coordinates(point, ui);
        move_mouse_to_abs_coordinates(abs_xy[0], abs_xy[1], ui);
    });
}

fn move_mouse_to_abs_coordinates(x: f64, y: f64, ui: &mut Ui) {
    ui.handle_event(&Input::Move(Motion::MouseCursor(x, y)));
}

fn test_handling_basic_input_event(ui: &mut Ui, event: Input) {
    ui.handle_event(&event);
    assert_event_was_pushed(ui, UiEvent::Raw(event));
}

fn assert_event_was_pushed(ui: &Ui, event: UiEvent) {
    let found = ui.global_input.all_events().find(|evt| **evt == event);
    assert!(found.is_some(),
            format!("expected to find event: {:?} in: \nall_events: {:?}",
                    event,
                    ui.global_input.all_events().collect::<Vec<&UiEvent>>()));
}

fn to_window_coordinates(xy: Point, ui: &Ui) -> Point {
    let x = (ui.win_w / 2.0) + xy[0];
    let y = (ui.win_h / 2.0) - xy[1];
    [x, y]
}

fn windowless_ui() -> Ui {
    let theme = Theme::default();
    let cc = MockCharacterCache::new();
    Ui::new(cc, theme)
}

#[derive(Copy, Clone)]
struct MockBackend;

impl ::Backend for MockBackend {
    type Texture = MockImageSize;
    type CharacterCache = MockCharacterCache;
}

struct MockImageSize {
    w: u32,
    h: u32,
}

impl ImageSize for MockImageSize {
    fn get_size(&self) -> (u32, u32) {
        (self.w, self.h)
    }
}

#[derive(Clone)]
struct MockCharacterCache{
    my_char: Character<'static, MockImageSize>
}

impl MockCharacterCache {
    fn new() -> MockCharacterCache {
        const MOCK_IMAGE_SIZE: &'static MockImageSize = &MockImageSize{ w: 14, h: 22 };
        MockCharacterCache {
            my_char: Character{
                offset: [0.0, 0.0],
                size: [14.0, 22.0],
                texture: MOCK_IMAGE_SIZE,
            }
        }
    }
}

impl CharacterCache for MockCharacterCache {
    type Texture = MockImageSize;

    fn character(&mut self, _font_size: FontSize, _ch: char) -> Character<'static, MockImageSize> {
        self.my_char.clone()
    }

}
