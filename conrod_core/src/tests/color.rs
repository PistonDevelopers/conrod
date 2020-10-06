use color::{hsl_to_rgb, rgb_to_hsl};
use std::cmp::Ordering::Equal;

///// Test assist code.

fn convert_rgb_to_hsl_to_rgb(expected_r: f32, expected_g: f32, expected_b: f32) -> (f32, f32, f32) {
    let (h, s, l) = rgb_to_hsl(expected_r, expected_g, expected_b);
    hsl_to_rgb(h, s, l)
}

fn compare_rgb_pairs(expected: (f32, f32, f32), actual: (f32, f32, f32)) -> bool {
    let (expected_r, expected_g, expected_b) = expected;
    let (actual_r, actual_g, actual_b) = actual;
    let r_comp = expected_r.partial_cmp(&actual_r).unwrap();
    let g_comp = expected_g.partial_cmp(&actual_g).unwrap();
    let b_comp = expected_b.partial_cmp(&actual_b).unwrap();
    r_comp == Equal && g_comp == Equal && b_comp == Equal
}

///// Actual tests.

#[test]
fn rgb_to_hsl_black() {
    // black (0,0,0) should convert to hsl (0.0, 0.0, 0.0)
    let (r, g, b) = (0.0, 0.0, 0.0);
    let actual = convert_rgb_to_hsl_to_rgb(r, g, b);
    assert!(compare_rgb_pairs((r, g, b), actual))
}

#[test]
fn rgb_to_hsl_white() {
    // white (255,255,255) should convert to hsl (0.0, 0.0, 1.0)
    let (r, g, b) = (1.0, 1.0, 1.0);
    let actual = convert_rgb_to_hsl_to_rgb(r, g, b);
    assert!(compare_rgb_pairs((r, g, b), actual))
}

#[test]
fn rgb_to_hsl_gray() {
    // gray rgb (128,128,128) should convert to hsl (0.0, 0.0, 0.5)
    let (r, g, b) = (0.5, 0.5, 0.5);
    let actual = convert_rgb_to_hsl_to_rgb(r, g, b);
    assert!(compare_rgb_pairs((r, g, b), actual));
}

#[test]
fn rgb_to_hsl_purple() {
    // purple rgb (128,0,128) should convert to hsl (5.23598766, 1.0, 0.25)
    let (r, g, b) = (0.5, 0.0, 0.5);
    let actual = convert_rgb_to_hsl_to_rgb(r, g, b);
    assert!(compare_rgb_pairs((r, g, b), actual));
}

#[test]
fn rgb_to_hsl_silver() {
    // silver rgb (191,191,191) should convert to hsl (0.0, 0.0, 0.75)
    let (r, g, b) = (0.75, 0.75, 0.75);
    let actual = convert_rgb_to_hsl_to_rgb(r, g, b);
    assert!(compare_rgb_pairs((r, g, b), actual));
}
