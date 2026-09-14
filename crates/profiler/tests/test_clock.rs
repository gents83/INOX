use inox_profiler::current_time_in_micros;

#[test]
fn current_time_is_after_unix_epoch() {
    assert!(current_time_in_micros() > 0);
}
