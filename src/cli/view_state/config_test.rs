use crate::cli::view_state::ViewState;

#[test]
fn read_and_write_config_file_succeeds() -> anyhow::Result<()> {
    // Arrange
    let mut view_state = ViewState {
        accepted_license_terms: true,
        config_file_path: Some(
            std::env::temp_dir().join(format!("space_test_{}", uuid::Uuid::new_v4())),
        ),
        ..Default::default()
    };

    // Act
    view_state.write_config_file()?; // Write true
    view_state.accepted_license_terms = false;
    view_state.read_config_file()?;

    // Assert
    assert!(view_state.accepted_license_terms);

    Ok(())
}

#[test]
fn accept_license_terms_updates_view_state_and_writes_config_file() -> anyhow::Result<()> {
    // Arrange
    let mut view_state = ViewState {
        accepted_license_terms: false,
        config_file_path: Some(
            std::env::temp_dir().join(format!("space_test_{}", uuid::Uuid::new_v4())),
        ),
        ..Default::default()
    };

    // Act
    view_state.accept_license_terms();

    // Assert
    assert!(view_state.accepted_license_terms);
    view_state.accepted_license_terms = false;
    view_state.read_config_file()?;
    assert!(view_state.accepted_license_terms);

    Ok(())
}
