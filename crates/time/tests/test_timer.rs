use std::time::Duration;

use inox_time::Timer;

#[test]
fn update_advances_frame_and_records_delta() {
    let mut timer = Timer::default();
    assert_eq!(timer.current_frame(), 0);
    assert_eq!(*timer.dt(), Duration::ZERO);

    timer.update();

    assert_eq!(timer.current_frame(), 1);
    assert!(timer.dt() >= &Duration::ZERO);
    assert_eq!(timer.fps(), 1);
}
