use iced::widget::image;
use iced::{Color, Rectangle};
use iced_audio::{h_slider};

use super::colors;

// Custom style for the Rect HSlider

pub struct RectStyle;
impl RectStyle {
    const ACTIVE_RECT_STYLE: h_slider::RectAppearance =
        h_slider::RectAppearance {
            back_color: colors::EMPTY,
            back_border_width: 1.0,
            back_border_radius: 2.0,
            back_border_color: colors::BORDER,
            filled_color: colors::FILLED,
            handle_width: 4,
            handle_color: colors::HANDLE,
            handle_filled_gap: 1.0,
        };
}
impl h_slider::StyleSheet for RectStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> h_slider::Appearance {
        h_slider::Appearance::Rect(Self::ACTIVE_RECT_STYLE)
    }

    fn hovered(&self, _style: &Self::Style) -> h_slider::Appearance {
        h_slider::Appearance::Rect(h_slider::RectAppearance {
            filled_color: colors::FILLED_HOVER,
            handle_width: 5,
            ..Self::ACTIVE_RECT_STYLE
        })
    }

    fn dragging(&self, style: &Self::Style) -> h_slider::Appearance {
        self.hovered(style)
    }

    fn mod_range_appearance(
        &self,
        _style: &Self::Style,
    ) -> Option<h_slider::ModRangeAppearance> {
        Some(h_slider::ModRangeAppearance {
            placement: h_slider::ModRangePlacement::Bottom {
                height: 3.0,
                offset: 2.0,
            },
            back_border_color: Color::TRANSPARENT,
            back_border_width: 0.0,
            back_border_radius: 2.0,
            back_color: Some(colors::KNOB_ARC_EMPTY),
            filled_color: colors::KNOB_ARC,
            filled_inverse_color: colors::KNOB_ARC_RIGHT,
        })
    }
}
