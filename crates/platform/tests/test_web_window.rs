use inox_platform::{canvas_mouse_button, map_canvas_mouse_coordinates, MouseButton, MouseState};

#[test]
fn maps_css_coordinates_to_device_pixels() {
    let (x, y, normalized_x, normalized_y) =
        map_canvas_mouse_coordinates(150.0, 75.0, 50.0, 25.0, 200.0, 100.0, 800, 400);
    assert_eq!((x, y), (400.0, 200.0));
    assert_eq!((normalized_x, normalized_y), (0.5, 0.5));
}

#[test]
fn clamps_coordinates_and_prevents_zero_dimension_division() {
    let (x, y, normalized_x, normalized_y) =
        map_canvas_mouse_coordinates(-10.0, f64::INFINITY, 0.0, 0.0, 0.0, 0.0, 0, 0);
    assert_eq!((x, y), (0.0, 0.0));
    assert_eq!((normalized_x, normalized_y), (0.0, 0.0));
}

#[test]
fn movement_does_not_report_a_button_press() {
    assert_eq!(canvas_mouse_button(MouseState::Move, 0), MouseButton::None);
    assert_eq!(canvas_mouse_button(MouseState::Down, 0), MouseButton::Left);
    assert_eq!(canvas_mouse_button(MouseState::Up, 2), MouseButton::Right);
}

#[cfg(target_arch = "wasm32")]
mod browser_contract {
    use super::*;

    #[test]
    fn device_pixel_dimensions_are_nonzero() {
        let (x, y, normalized_x, normalized_y) =
            map_canvas_mouse_coordinates(0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0, 0);
        assert_eq!((x, y), (0.0, 0.0));
        assert_eq!((normalized_x, normalized_y), (0.0, 0.0));
    }
}
