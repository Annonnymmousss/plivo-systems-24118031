use flaky_network::config::TransportConfig;

#[test]
fn fec_width_uses_at_most_half_the_playout_budget_for_future_recovery() {
    assert_eq!(TransportConfig::for_delay(39.0).fec_width, 2);
    assert_eq!(TransportConfig::for_delay(45.0).fec_width, 2);
    assert_eq!(TransportConfig::for_delay(85.0).fec_width, 3);
    assert_eq!(TransportConfig::for_delay(170.0).fec_width, 5);
    assert_eq!(TransportConfig::for_delay(10_000.0).fec_width, 5);
}

#[test]
fn recovery_storage_is_bounded_and_scales_with_delay() {
    let short = TransportConfig::for_delay(45.0);
    let moderate = TransportConfig::for_delay(85.0);
    let long = TransportConfig::for_delay(10_000.0);
    let extreme = TransportConfig::for_delay(f64::MAX);

    assert_eq!(short.recovery_slots, 16);
    assert_eq!(moderate.recovery_slots, 16);
    assert!(long.recovery_slots > moderate.recovery_slots);
    assert!(long.recovery_slots <= 2_048);
    assert_eq!(extreme.recovery_slots, 2_048);
}

#[test]
fn invalid_delay_uses_safe_defaults() {
    assert_eq!(
        TransportConfig::for_delay(f64::NAN),
        TransportConfig::for_delay(120.0)
    );
    assert_eq!(
        TransportConfig::for_delay(-1.0),
        TransportConfig::for_delay(120.0)
    );
}
