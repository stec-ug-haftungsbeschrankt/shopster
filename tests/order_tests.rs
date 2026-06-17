use std::convert::TryFrom;
use stec_shopster::DbOrderStatus;

/// Test successful conversions from valid i32 values to DbOrderStatus
#[test]
fn test_valid_order_status_conversions() {
    // Test all valid status values
    assert_eq!(DbOrderStatus::try_from(0).unwrap(), DbOrderStatus::New);
    assert_eq!(DbOrderStatus::try_from(1).unwrap(), DbOrderStatus::InProgress);
    assert_eq!(DbOrderStatus::try_from(2).unwrap(), DbOrderStatus::ReadyToShip);
    assert_eq!(DbOrderStatus::try_from(3).unwrap(), DbOrderStatus::Shipping);
    assert_eq!(DbOrderStatus::try_from(4).unwrap(), DbOrderStatus::Done);
}

/// Test that invalid i32 values return errors instead of panicking
#[test]
fn test_invalid_order_status_conversions() {
    // Test negative values
    assert!(DbOrderStatus::try_from(-1).is_err());
    assert!(DbOrderStatus::try_from(-100).is_err());

    // Test out-of-range positive values
    assert!(DbOrderStatus::try_from(5).is_err());
    assert!(DbOrderStatus::try_from(10).is_err());
    assert!(DbOrderStatus::try_from(100).is_err());
    assert!(DbOrderStatus::try_from(i32::MAX).is_err());
    assert!(DbOrderStatus::try_from(i32::MIN).is_err());
}

/// Test that error messages are informative
#[test]
fn test_order_status_error_messages() {
    let invalid_values = vec![-1, 5, 10, 99];

    for val in invalid_values {
        let result = DbOrderStatus::try_from(val);
        assert!(result.is_err(), "Expected error for value: {}", val);

        let error_msg = result.unwrap_err();
        assert!(
            error_msg.contains(&val.to_string()),
            "Error message should contain the invalid value: {}",
            val
        );
        assert!(
            error_msg.contains("Unknown order status"),
            "Error message should describe the issue"
        );
    }
}

/// Test round-trip conversion: DbOrderStatus -> i32 -> DbOrderStatus
#[test]
fn test_round_trip_conversion() {
    let statuses = vec![
        DbOrderStatus::New,
        DbOrderStatus::InProgress,
        DbOrderStatus::ReadyToShip,
        DbOrderStatus::Shipping,
        DbOrderStatus::Done,
    ];

    for original_status in statuses {
        // Convert to i32
        let status_ref = &original_status;
        let as_i32 = i32::from(status_ref);

        // Convert back to DbOrderStatus
        let converted_back = DbOrderStatus::try_from(as_i32)
            .expect("Should successfully convert valid i32 back to DbOrderStatus");

        // Verify we got the same status
        assert_eq!(
            converted_back, original_status,
            "Round-trip conversion failed for status: {:?}",
            original_status
        );
    }
}

/// Test that the conversion is safe and doesn't panic even with extreme values
#[test]
fn test_extreme_values_dont_panic() {
    // These should not panic, they should return errors
    let extreme_values = vec![
        i32::MIN,
        i32::MIN + 1,
        -1000000,
        -1,
        5,
        1000,
        i32::MAX - 1,
        i32::MAX,
    ];

    for val in extreme_values {
        let result = DbOrderStatus::try_from(val);
        // All of these should produce an error, not a panic
        assert!(
            result.is_err(),
            "Should return error for extreme value: {}",
            val
        );
    }
}

/// Verify that valid statuses match expected numeric values
#[test]
fn test_status_numeric_mapping() {
    let mappings = vec![
        (0, DbOrderStatus::New),
        (1, DbOrderStatus::InProgress),
        (2, DbOrderStatus::ReadyToShip),
        (3, DbOrderStatus::Shipping),
        (4, DbOrderStatus::Done),
    ];

    for (expected_num, status) in mappings {
        let num_from_status = i32::from(&status);
        assert_eq!(
            num_from_status, expected_num,
            "Status {:?} should map to {}",
            status, expected_num
        );

        let status_from_num = DbOrderStatus::try_from(expected_num)
            .expect("Valid status number should convert");
        assert_eq!(status, status_from_num, "Round-trip failed for value: {}", expected_num);
    }
}


