use {
    Color,
    Labelable,
    Positionable,
    Colorable,
    Sizeable,
    Widget,
    Ui,
    UiBuilder
};
use event::{self, Input};
use input::{self, Button, Key, Motion, MouseButton};
use input::keyboard::ModifierKey;
use widget;
use position::Point;


///// Test assist code.



fn left_click_mouse(ui: &mut Ui) {
    press_mouse_button(MouseButton::Left, ui);
    release_mouse_button(MouseButton::Left, ui);
}

fn release_mouse_button(button: MouseButton, ui: &mut Ui) {
    let event = Input::Release(Button::Mouse(button));
    ui.handle_event(event);
}

fn press_mouse_button(button: MouseButton, ui: &mut Ui) {
    let event = Input::Press(Button::Mouse(button));
    ui.handle_event(event);
}

fn move_mouse_to_widget(widget_id: widget::Id, ui: &mut Ui) {
    ui.xy_of(widget_id).map(|point| {
        let abs_xy = to_window_coordinates(point, ui);
        move_mouse_to_abs_coordinates(abs_xy[0], abs_xy[1], ui);
    });
}

fn move_mouse_to_abs_coordinates(x: f64, y: f64, ui: &mut Ui) {
    ui.handle_event(Input::Motion(Motion::MouseCursor { x: x, y: y }));
}

fn test_handling_basic_input_event(ui: &mut Ui, event: Input) {
    ui.handle_event(event.clone());
    assert_event_was_pushed(ui, event::Event::Raw(event));
}

fn assert_event_was_pushed(ui: &Ui, event: event::Event) {
    let found = ui.global_input().events().find(|evt| **evt == event);
    assert!(found.is_some(),
            format!("expected to find event: {:?} in: \nevents: {:?}",
                    event,
                    ui.global_input().events().collect::<Vec<&event::Event>>()));
}

fn to_window_coordinates(xy: Point, ui: &Ui) -> Point {
    let x = (ui.win_w / 2.0) + xy[0];
    let y = (ui.win_h / 2.0) - xy[1];
    [x, y]
}

fn windowless_ui() -> Ui {
    UiBuilder::new([800.0, 600.0]).build()
}


///// Actual tests.


#[test]
fn ui_should_reset_global_input_after_widget_are_set() {

    let ui = &mut windowless_ui();
    ui.win_w = 250.0;
    ui.win_h = 300.0;

    let (canvas, button) = {
        let mut id_generator = ui.widget_id_generator();
        (id_generator.next(), id_generator.next())
    };

    move_mouse_to_widget(button, ui);
    for _ in 0..2 {
        left_click_mouse(ui);
        let ui = &mut ui.set_widgets();

        assert!(ui.global_input().events().next().is_some());

        widget::Canvas::new()
            .color(Color::Rgba(1.0, 1.0, 1.0, 1.0))
            .set(canvas, ui);
        widget::Button::new()
            .w_h(100.0, 200.0)
            .label("MyButton")
            .bottom_right_of(canvas)
            .set(button, ui);
    }

    assert!(ui.global_input().events().next().is_none());
}


#[test]
fn ui_should_push_input_events_to_aggregator() {
    let ui = &mut windowless_ui();

    test_handling_basic_input_event(ui, Input::Press(input::Button::Keyboard(Key::LCtrl)));
    test_handling_basic_input_event(ui, Input::Release(input::Button::Keyboard(Key::LCtrl)));
    test_handling_basic_input_event(ui, Input::Text("my string".to_string()));
    test_handling_basic_input_event(ui, Input::Resize(55.0, 99.0));
    test_handling_basic_input_event(ui, Input::Focus(true));
}

#[test]
fn high_level_scroll_event_should_be_created_from_a_raw_mouse_scroll() {
    let mut ui = windowless_ui();
    ui.handle_event(Input::Motion(Motion::Scroll { x: 10.0, y: 33.0 }));

    let expected_scroll = event::Scroll{
        x: 10.0,
        y: 33.0,
        modifiers: ModifierKey::default()
    };
    let event = ui.global_input().events().next().expect("expected a scroll event");
    if let event::Event::Ui(event::Ui::Scroll(_, scroll)) = *event {
        assert_eq!(expected_scroll, scroll);
    }
}
