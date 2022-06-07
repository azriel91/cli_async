use crossterm::style::{Attribute, Attributes, Color, ContentStyle};
use once_cell::sync::Lazy;

/// Colours for UI output on terminal
pub struct Colours;

impl Colours {
    /// Logo left color.
    pub const LOGO_LEFT: Lazy<ContentStyle> = Lazy::new(|| ContentStyle {
        foreground_color: Some(Color::Blue),
        background_color: None,
        attributes: Attributes::from(Attribute::Bold),
    });
    /// Logo left color.
    pub const LOGO_RIGHT: Lazy<ContentStyle> = Lazy::new(|| ContentStyle {
        foreground_color: Some(Color::Green),
        background_color: None,
        attributes: Attributes::from(Attribute::Bold),
    });

    /// Styling for a report border.
    pub const REPORT_BORDER: Lazy<ContentStyle> = Lazy::new(|| ContentStyle {
        foreground_color: Some(Color::Blue),
        background_color: None,
        attributes: Attributes::from(Attribute::Bold),
    });
    /// Styling for a report section title.
    pub const REPORT_TITLE: Lazy<ContentStyle> = Lazy::new(|| ContentStyle {
        foreground_color: Some(Color::Cyan),
        background_color: None,
        attributes: Attributes::from(Attribute::Bold),
    });
    /// Styling for a report error section title.
    pub const REPORT_TITLE_ERROR: Lazy<ContentStyle> = Lazy::new(|| ContentStyle {
        foreground_color: Some(Color::Red),
        background_color: None,
        attributes: Attributes::from(Attribute::Bold),
    });
    /// Styling for a report label.
    pub const REPORT_LABEL: Lazy<ContentStyle> = Lazy::new(|| ContentStyle {
        foreground_color: None,
        background_color: None,
        attributes: Attributes::from(Attribute::Bold),
    });
    /// Styling for a report item success.
    pub const REPORT_ITEM_SUCCESS: Lazy<ContentStyle> = Lazy::new(|| ContentStyle {
        foreground_color: Some(Color::Green),
        background_color: None,
        attributes: Attributes::from(Attribute::Bold),
    });
    /// Styling for a report item partial success.
    pub const REPORT_ITEM_PARTIAL_SUCCESS: Lazy<ContentStyle> = Lazy::new(|| ContentStyle {
        foreground_color: Some(Color::Rgb {
            r: 216,
            g: 216,
            b: 0,
        }),
        background_color: None,
        attributes: Attributes::from(Attribute::Bold),
    });
    /// Styling for a report item failure.
    pub const REPORT_ITEM_FAILURE: Lazy<ContentStyle> = Lazy::new(|| ContentStyle {
        foreground_color: Some(Color::Red),
        background_color: None,
        attributes: Attributes::from(Attribute::Bold),
    });
    /// Styling for a report error item.
    pub const REPORT_ERROR_ITEM: Lazy<ContentStyle> = Lazy::new(|| ContentStyle {
        foreground_color: None,
        background_color: None,
        attributes: Attributes::default(),
    });
    /// Styling for a report error item.
    pub const REPORT_ERROR_MESSAGE: Lazy<ContentStyle> = Lazy::new(|| ContentStyle {
        foreground_color: Some(Color::Yellow),
        background_color: None,
        attributes: Attributes::default(),
    });
}
