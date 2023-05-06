#[derive(Clone, Debug)]
pub enum QuadButton {
    Primary,
    Positive,
}
use iced::{Background, Color};
use iced_native::widget::button::{Appearance, StyleSheet};
impl StyleSheet for QuadButton {
    type Style = iced_native::Theme;

    fn active(&self, _: &Self::Style) -> Appearance {
        let background_color = match self {
            Self::Primary => Color::from_rgb8(51, 89, 218),
            Self::Positive => Color::from_rgb8(18, 102, 79),
        };
        Appearance {
            shadow_offset: iced_native::Vector { x: 5.0, y: 5.0 },
            border_radius: 10.0,
            background: Some(Background::Color(background_color)),
            text_color: Color::from_rgb(1.0, 1.0, 1.0),
            ..Default::default()
        }
    }

    fn hovered(&self, style: &Self::Style) -> Appearance {
        let active = self.active(style);
        Appearance {
            background: active.background.map(|background| match background {
                Background::Color(color) => Background::Color(Color {
                    a: color.a * 0.9,
                    ..color
                }),
            }),
            ..active
        }
    }

    fn pressed(&self, style: &Self::Style) -> Appearance {
        Appearance {
            ..self.active(style)
        }
    }

    fn disabled(&self, style: &Self::Style) -> Appearance {
        let active = self.active(style);

        Appearance {
            background: active.background.map(|background| match background {
                Background::Color(color) => Background::Color(Color {
                    a: color.a * 0.5,
                    ..color
                }),
            }),
            text_color: Color {
                a: active.text_color.a * 0.5,
                ..active.text_color
            },
            ..active
        }
    }
}
