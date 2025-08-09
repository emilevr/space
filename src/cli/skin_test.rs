use super::Skin;
use ratatui::style::{Modifier, Style};

#[test]
fn value_style_with_no_fg_does_not_set_fg() {
    // Arrange
    let skin = Skin {
        value_fg_color: None,
        ..Default::default()
    };

    // Act
    let style = skin.value_style();

    // Assert
    assert_eq!(Style::default(), style);
}

#[test]
fn value_style_with_fg_sets_fg() {
    // Arrange
    let skin = Skin::default();

    // Act
    let style = skin.value_style();

    // Assert
    assert_eq!(Style::default().fg(skin.value_fg_color.unwrap()), style);
}

#[test]
fn default_skin_value_style_does_not_reverse() {
    // Arrange
    let skin = Skin::default();

    // Act
    let style = skin.value_style();

    // Assert
    assert_ne!(
        Style::default()
            .fg(skin.value_fg_color.unwrap())
            .add_modifier(Modifier::REVERSED),
        style
    );
}

#[test]
fn value_style_with_reverse_applies_reverse() {
    // Arrange
    let skin = Skin {
        value_style_reversed: true,
        ..Default::default()
    };

    // Act
    let style = skin.value_style();

    // Assert
    assert_eq!(
        Style::default()
            .fg(skin.value_fg_color.unwrap())
            .add_modifier(Modifier::REVERSED),
        style
    );
}
