use crate::size::{
    Size, SizeDisplayData, SizeDisplayFormat, BINARY_DISPLAY_DATA, METRIC_DISPLAY_DATA,
};
use rstest::rstest;

#[rstest]
#[case(100, 10, 90)]
#[case(10, 100, 0)]
fn subtract_results_in_correct_internal_value(
    #[case] value: u64,
    #[case] delta: u64,
    #[case] expected_value: u64,
) {
    // Arrange
    let mut size = Size::new(value);

    // Act
    size.subtract(delta);

    // Assert
    assert_eq!(expected_value, size.get_value());
}

#[rstest]
#[case(1000, 100, 0.1f32)]
#[case(100, 100, 1f32)]
#[case(10, 0, 0f32)]
#[case(0, 10, 0f32)]
fn get_fraction_returns_correct_value(
    #[case] total_size_in_bytes: u64,
    #[case] value: u64,
    #[case] expected_fraction: f32,
) {
    // Arrange
    let size = Size::new(value);

    // Act
    let fraction = size.get_fraction(total_size_in_bytes);

    // Assert
    assert_eq!(expected_fraction, fraction);
}

#[rstest]
#[case(0, SizeDisplayFormat::Binary, BINARY_DISPLAY_DATA[2])]
#[case(1024, SizeDisplayFormat::Binary, BINARY_DISPLAY_DATA[2])]
#[case(1025, SizeDisplayFormat::Binary, BINARY_DISPLAY_DATA[2])]
#[case(1048576, SizeDisplayFormat::Binary, BINARY_DISPLAY_DATA[2])]
#[case(1048577, SizeDisplayFormat::Binary, BINARY_DISPLAY_DATA[1])]
#[case(1073741824, SizeDisplayFormat::Binary, BINARY_DISPLAY_DATA[1])]
#[case(1073741825, SizeDisplayFormat::Binary, BINARY_DISPLAY_DATA[0])]
#[case(u64::MAX, SizeDisplayFormat::Binary, BINARY_DISPLAY_DATA[0])]
#[case(0, SizeDisplayFormat::Metric, METRIC_DISPLAY_DATA[2])]
#[case(1000, SizeDisplayFormat::Metric, METRIC_DISPLAY_DATA[2])]
#[case(1001, SizeDisplayFormat::Metric, METRIC_DISPLAY_DATA[2])]
#[case(1000000, SizeDisplayFormat::Metric, METRIC_DISPLAY_DATA[2])]
#[case(1000001, SizeDisplayFormat::Metric, METRIC_DISPLAY_DATA[1])]
#[case(1000000000, SizeDisplayFormat::Metric, METRIC_DISPLAY_DATA[1])]
#[case(1000000001, SizeDisplayFormat::Metric, METRIC_DISPLAY_DATA[0])]
#[case(u64::MAX, SizeDisplayFormat::Metric, METRIC_DISPLAY_DATA[0])]
fn get_best_format_returns_correct_display_data(
    #[case] size_in_bytes: u64,
    #[case] display_format: SizeDisplayFormat,
    #[case] expected_display_data: &SizeDisplayData,
) {
    // Arrange

    // Act
    let display_data = Size::get_best_format(size_in_bytes, &display_format);

    // Assert
    assert_eq!(expected_display_data, display_data);
}

#[test]
fn get_value_returns_correct_value() {
    // Arrange
    const VALUE: u64 = 123;
    let item = Size { value: VALUE };

    // Act
    let value = item.get_value();

    // Assert
    assert_eq!(VALUE, value);
}
