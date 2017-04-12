use theme;

#[test]
fn default_theme_json_serialization() {
    let theme = theme::Theme::default();
    assert_eq!(theme.into_json().dump(), r#"{"name":"Demo Theme","padding":{"x":[0,0],"y":[0,0]},"x_position":{"align":{"alignment":"Start","parent":null}},"y_position":{"direction":{"direction":"Backwards","offset":20,"parent":null}},"background_color":[0,0,0,1],"shape_color":[1,1,1,1],"border_color":[0,0,0,1],"border_width":1,"label_color":[0,0,0,1],"font_size_large":26,"font_size_medium":18,"font_size_small":12,"mouse_drag_threshold":0,"double_click_threshold":500}"#)
}