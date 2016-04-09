use {
    Theme,
    Canvas,
    CharacterCache,
    Color,
    FontSize,
    Labelable,
    Positionable,
    Colorable,
    Sizeable,
    Widget
};
use backend::event::Event;
use backend::graphics::{Character, ImageSize};
use event::{self, Input, Motion, UiEvent};
use input::{self, Button, Key, MouseButton, Provider};
use input::keyboard::ModifierKey;
use widget::{Index, self};
use widget::button::Button as ButtonWidget;
use position::Point;


///// Test assist code.


type Ui = ::Ui<MockBackend>;

fn left_click_mouse(ui: &mut Ui) {
    press_mouse_button(MouseButton::Left, ui);
    release_mouse_button(MouseButton::Left, ui);
}

fn release_mouse_button(button: MouseButton, ui: &mut Ui) {
    let event = Event::Input(Input::Release(Button::Mouse(button)));
    ui.handle_event(event);
}

fn press_mouse_button(button: MouseButton, ui: &mut Ui) {
    let event = Event::Input(Input::Press(Button::Mouse(button)));
    ui.handle_event(event);
}

fn move_mouse_to_widget(widget_idx: Index, ui: &mut Ui) {
    ui.xy_of(widget_idx).map(|point| {
        let abs_xy = to_window_coordinates(point, ui);
        move_mouse_to_abs_coordinates(abs_xy[0], abs_xy[1], ui);
    });
}

fn move_mouse_to_abs_coordinates(x: f64, y: f64, ui: &mut Ui) {
    ui.handle_event(Event::Input(Input::Move(Motion::MouseCursor(x, y))));
}

fn test_handling_basic_input_event(ui: &mut Ui, event: Input) {
    ui.handle_event(Event::Input(event.clone()));
    assert_event_was_pushed(ui, UiEvent::Raw(event));
}

fn assert_event_was_pushed(ui: &Ui, event: UiEvent) {
    let found = ui.global_input.events().find(|evt| **evt == event);
    assert!(found.is_some(),
            format!("expected to find event: {:?} in: \nevents: {:?}",
                    event,
                    ui.global_input.events().collect::<Vec<&UiEvent>>()));
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

#[derive(Clone)]
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


///// Actual tests.


#[test]
fn ui_should_reset_global_input_after_widget_are_set() {
    let mut ui = windowless_ui();
    ui.win_w = 250.0;
    ui.win_h = 300.0;

    const CANVAS_ID: widget::Id = widget::Id(0);
    const BUTTON_ID: widget::Id = widget::Id(1);

    move_mouse_to_widget(Index::Public(BUTTON_ID), &mut ui);
    left_click_mouse(&mut ui);

    assert!(ui.global_input.events().next().is_some());
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

    assert!(ui.global_input.events().next().is_none());
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
fn ui_should_push_input_events_to_aggregator() {
    let mut ui = windowless_ui();

    test_handling_basic_input_event(&mut ui, Input::Press(input::Button::Keyboard(Key::LCtrl)));
    test_handling_basic_input_event(&mut ui, Input::Release(input::Button::Keyboard(Key::LCtrl)));
    test_handling_basic_input_event(&mut ui, Input::Text("my string".to_string()));
    test_handling_basic_input_event(&mut ui, Input::Resize(55, 99));
    test_handling_basic_input_event(&mut ui, Input::Focus(true));
    test_handling_basic_input_event(&mut ui, Input::Cursor(true));
}

#[test]
fn high_level_scroll_event_should_be_created_from_a_raw_mouse_scroll() {
    let mut ui = windowless_ui();
    ui.handle_event(Input::Move(Motion::MouseScroll(10.0, 33.0)));

    let expected_scroll = event::Scroll{
        x: 10.0,
        y: 33.0,
        modifiers: ModifierKey::default()
    };
    let actual_scroll = ui.global_input.scroll().expect("expected a scroll event");
    assert_eq!(expected_scroll, actual_scroll);
}
