use crate::audio::AUDIO_PLAYER;
use crate::{visiting, Message, Stage, State};
use iced::widget::{self, column, container, row, text};
use iced::{Alignment, Length, Theme};
use std::io::Write;
use std::sync::atomic::Ordering;

#[derive(Clone, Debug)]
pub struct Configs {
    pub shown: bool,
    pub full_screened: bool,
    pub config_path: String,
    pub theme: Theme,
    pub from_date: visiting::ShootingTime,
    pub volume_percentage: f32,
    pub id: iced::window::Id,
}

pub fn settings_over(config: Configs, content: iced::Element<Message>) -> iced::Element<Message> {
    let modal = container(
        column![
            text("设置").size(38),
            column![
                column![
                    widget::toggler(
                        String::from("暗色主题"),
                        config.theme == Theme::Dark,
                        |b| Message::IsDarkTheme(b)
                    )
                    .text_size(28),
                    widget::toggler(
                        String::from("程序退出时删除缓存"),
                        crate::DELETE_FILES_ON_EXIT.load(Ordering::Relaxed),
                        |_| Message::SwitchDeleteFilesStatus
                    )
                    .text_size(28),
                    text("音量控制").size(32),
                    row![
                        iced::widget::Slider::new(
                            0.0..=120.0,
                            config.volume_percentage,
                            crate::Message::ModifyVolume
                        )
                        .height(30.0)
                        .width(Length::Fill),
                        text(format!("{:>4}%", config.volume_percentage)).size(25)
                    ]
                    .align_items(Alignment::Center),
                    row![
                        widget::tooltip(
                            crate::button_from_svg(include_bytes!("./runtime/plus.svg"),)
                                .width(Length::Fixed(40.0))
                                .on_press(Message::ScaleEnlarge),
                            "放大「按 +」",
                            widget::tooltip::Position::Bottom,
                        )
                        .style(iced::theme::Container::Box),
                        widget::tooltip(
                            crate::button_from_svg(include_bytes!("./runtime/minus.svg"),)
                                .width(Length::Fixed(40.0))
                                .on_press(Message::ScaleDown),
                            "缩小「按 -」",
                            widget::tooltip::Position::Bottom,
                        )
                        .style(iced::theme::Container::Box),
                        if config.full_screened {
                            widget::tooltip(
                                crate::button_from_svg(include_bytes!("./runtime/compress.svg"))
                                    .width(Length::Fixed(40.0))
                                    .on_press(Message::ToggleMode),
                                "窗口显示「Esc」",
                                widget::tooltip::Position::Bottom,
                            )
                            .style(iced::theme::Container::Box)
                        } else {
                            widget::tooltip(
                                crate::button_from_svg(include_bytes!("./runtime/expand.svg"))
                                    .width(Length::Fixed(40.0))
                                    .on_press(Message::ToggleMode),
                                "全屏显示「Alt + Enter」",
                                widget::tooltip::Position::Bottom,
                            )
                            .style(iced::theme::Container::Box)
                        },
                        if AUDIO_PLAYER
                            .lock()
                            .unwrap()
                            .as_ref()
                            .unwrap()
                            .sink
                            .is_paused()
                        {
                            row![widget::tooltip(
                                crate::button_from_svg(include_bytes!("./runtime/play.svg"),)
                                    .width(Length::Fixed(40.0))
                                    .on_press(Message::SwitchMusicStatus),
                                "播放 / 暂停",
                                widget::tooltip::Position::Bottom
                            )
                            .style(iced::theme::Container::Box)
                            .gap(5)]
                        } else {
                            row![
                                widget::tooltip(
                                    crate::button_from_svg(include_bytes!("./runtime/pause.svg"),)
                                        .width(Length::Fixed(40.0))
                                        .on_press(Message::SwitchMusicStatus),
                                    "播放 / 暂停「按 M」",
                                    widget::tooltip::Position::Bottom
                                )
                                .style(iced::theme::Container::Box)
                                .gap(5),
                                widget::tooltip(
                                    crate::button_from_svg(include_bytes!(
                                        "./runtime/square-right.svg"
                                    ))
                                    .width(Length::Fixed(40.0))
                                    .on_press(Message::NextSong),
                                    "跳到下一首「按 N」",
                                    widget::tooltip::Position::Bottom
                                )
                                .style(iced::theme::Container::Box)
                            ]
                        },
                    ]
                ]
                .spacing(10),
                widget::button(text("设置好啦！").size(32)).on_press(Message::HideSettings)
            ]
            .align_items(Alignment::End)
            .spacing(10)
        ]
        .spacing(20),
    )
    .width(Length::Fixed(350.0))
    .padding(10)
    .style(iced::theme::Container::Box);
    use modal::Modal;
    Modal::new(content, modal)
        .on_blur(Message::HideSettings)
        .into()
}

pub fn save_configs(state: &mut State) {
    if crate::DELETE_FILES_ON_EXIT.load(Ordering::Relaxed) {
        return;
    }
    let configs = &state.configs;
    let mut map = toml::map::Map::new();
    let stage = match &state.stage {
        Stage::EntryEvents(_) => "EntryEvents",
        Stage::ChoosingCharacter(chosen) => {
            if let Some(on_character) = chosen.on_character {
                map.insert(
                    String::from("on_character"),
                    toml::Value::Integer(on_character as i64),
                );
            } else {
                map.insert(String::from("on_character"), toml::Value::Integer(-1));
            }
            "ChoosingCharacter"
        }
        Stage::ShowingPlots(_) => "ShowingPlots",
        Stage::Graduated(_) => "Graduated",
    }
    .to_string();
    map.insert(String::from("stage"), toml::Value::String(stage));
    map.insert(
        String::from("volume-percentage"),
        toml::Value::Float(configs.volume_percentage.into()),
    );
    map.insert(
        String::from("scale-factor"),
        toml::Value::Float(crate::load_scale_factor()),
    );
    map.insert(
        String::from("light-theme"),
        toml::Value::Boolean(configs.theme == Theme::Light),
    );
    map.insert(
        String::from("from-date"),
        toml::Value::Datetime(configs.from_date.clone().into()),
    );
    map.insert(
        String::from("audio-paused"),
        toml::Value::Boolean(
            AUDIO_PLAYER
                .lock()
                .unwrap()
                .as_ref()
                .unwrap()
                .sink
                .is_paused(),
        ),
    );
    let mut buffer = std::fs::File::create(state.configs.config_path.clone()).unwrap();
    buffer
        .write_all(toml::to_string_pretty(&map).unwrap().as_bytes())
        .unwrap();
}
mod modal {
    use iced::alignment::Alignment;
    use iced::event;
    use iced::mouse;
    use iced::{Color, Element, Event, Length, Point, Rectangle, Size};
    use iced_core::layout::{self, Layout};
    use iced_core::overlay;
    use iced_core::renderer;
    use iced_core::widget::{self, Widget};
    use iced_core::{self, Clipboard, Shell};

    /// A widget that centers a modal element over some base element
    pub struct Modal<'a, Message, Renderer> {
        base: Element<'a, Message, Renderer>,
        modal: Element<'a, Message, Renderer>,
        on_blur: Option<Message>,
    }

    impl<'a, Message, Renderer> Modal<'a, Message, Renderer> {
        /// Returns a new [`Modal`]
        pub fn new(
            base: impl Into<Element<'a, Message, Renderer>>,
            modal: impl Into<Element<'a, Message, Renderer>>,
        ) -> Self {
            Self {
                base: base.into(),
                modal: modal.into(),
                on_blur: None,
            }
        }

        /// Sets the message that will be produces when the background
        /// of the [`Modal`] is pressed
        pub fn on_blur(self, on_blur: Message) -> Self {
            Self {
                on_blur: Some(on_blur),
                ..self
            }
        }
    }

    impl<'a, Message, Renderer> Widget<Message, Renderer> for Modal<'a, Message, Renderer>
    where
        Renderer: iced_core::Renderer,
        Message: Clone,
    {
        fn children(&self) -> Vec<widget::Tree> {
            vec![
                widget::Tree::new(&self.base),
                widget::Tree::new(&self.modal),
            ]
        }

        fn diff(&self, tree: &mut widget::Tree) {
            tree.diff_children(&[&self.base, &self.modal]);
        }

        fn width(&self) -> Length {
            self.base.as_widget().width()
        }

        fn height(&self) -> Length {
            self.base.as_widget().height()
        }

        fn layout(
            &self,
            tree: &mut iced_core::widget::Tree,
            renderer: &Renderer,
            limits: &layout::Limits,
        ) -> layout::Node {
            self.base.as_widget().layout(tree, renderer, limits)
        }

        fn on_event(
            &mut self,
            state: &mut widget::Tree,
            event: Event,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            renderer: &Renderer,
            clipboard: &mut dyn Clipboard,
            shell: &mut Shell<'_, Message>,
            viewport: &Rectangle,
        ) -> event::Status {
            self.base.as_widget_mut().on_event(
                &mut state.children[0],
                event,
                layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            )
        }

        fn draw(
            &self,
            state: &widget::Tree,
            renderer: &mut Renderer,
            theme: &<Renderer as iced_core::Renderer>::Theme,
            style: &renderer::Style,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            viewport: &Rectangle,
        ) {
            self.base.as_widget().draw(
                &state.children[0],
                renderer,
                theme,
                style,
                layout,
                cursor,
                viewport,
            );
        }

        fn overlay<'b>(
            &'b mut self,
            state: &'b mut widget::Tree,
            layout: Layout<'_>,
            _renderer: &Renderer,
        ) -> Option<overlay::Element<'b, Message, Renderer>> {
            Some(overlay::Element::new(
                layout.position(),
                Box::new(Overlay {
                    content: &mut self.modal,
                    tree: &mut state.children[1],
                    size: layout.bounds().size(),
                    on_blur: self.on_blur.clone(),
                }),
            ))
        }

        fn mouse_interaction(
            &self,
            state: &widget::Tree,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            viewport: &Rectangle,
            renderer: &Renderer,
        ) -> mouse::Interaction {
            self.base.as_widget().mouse_interaction(
                &state.children[0],
                layout,
                cursor,
                viewport,
                renderer,
            )
        }

        fn operate(
            &self,
            state: &mut widget::Tree,
            layout: Layout<'_>,
            renderer: &Renderer,
            operation: &mut dyn widget::Operation<Message>,
        ) {
            self.base
                .as_widget()
                .operate(&mut state.children[0], layout, renderer, operation);
        }
    }

    struct Overlay<'a, 'b, Message, Renderer> {
        content: &'b mut Element<'a, Message, Renderer>,
        tree: &'b mut widget::Tree,
        size: Size,
        on_blur: Option<Message>,
    }

    impl<'a, 'b, Message, Renderer> overlay::Overlay<Message, Renderer>
        for Overlay<'a, 'b, Message, Renderer>
    where
        Renderer: iced_core::Renderer,
        Message: Clone,
    {
        fn layout(
            &mut self,
            renderer: &Renderer,
            _bounds: Size,
            position: Point,
            _translation: iced_core::Vector,
        ) -> layout::Node {
            let limits = layout::Limits::new(Size::ZERO, self.size)
                .width(Length::Fill)
                .height(Length::Fill);

            let mut child = self
                .content
                .as_widget()
                .layout(self.tree, renderer, &limits);

            child.align(Alignment::Center, Alignment::Center, limits.max());

            let mut node = layout::Node::with_children(self.size, vec![child]);
            node.move_to(position);

            node
        }

        fn on_event(
            &mut self,
            event: Event,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            renderer: &Renderer,
            clipboard: &mut dyn Clipboard,
            shell: &mut Shell<'_, Message>,
        ) -> event::Status {
            let content_bounds = layout.children().next().unwrap().bounds();

            if let Some(message) = self.on_blur.as_ref() {
                if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = &event {
                    if !cursor.is_over(content_bounds) {
                        shell.publish(message.clone());
                        return event::Status::Captured;
                    }
                }
            }

            self.content.as_widget_mut().on_event(
                self.tree,
                event,
                layout.children().next().unwrap(),
                cursor,
                renderer,
                clipboard,
                shell,
                &layout.bounds(),
            )
        }

        fn draw(
            &self,
            renderer: &mut Renderer,
            theme: &<Renderer as iced_core::Renderer>::Theme,
            style: &renderer::Style,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
        ) {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: layout.bounds(),
                    border_radius: Default::default(),
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                },
                Color {
                    a: 0.80,
                    ..Color::BLACK
                },
            );

            self.content.as_widget().draw(
                self.tree,
                renderer,
                theme,
                style,
                layout.children().next().unwrap(),
                cursor,
                &layout.bounds(),
            );
        }

        fn operate(
            &mut self,
            layout: Layout<'_>,
            renderer: &Renderer,
            operation: &mut dyn widget::Operation<Message>,
        ) {
            self.content.as_widget().operate(
                self.tree,
                layout.children().next().unwrap(),
                renderer,
                operation,
            );
        }

        fn mouse_interaction(
            &self,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            viewport: &Rectangle,
            renderer: &Renderer,
        ) -> mouse::Interaction {
            self.content.as_widget().mouse_interaction(
                self.tree,
                layout.children().next().unwrap(),
                cursor,
                viewport,
                renderer,
            )
        }

        fn overlay<'c>(
            &'c mut self,
            layout: Layout<'_>,
            renderer: &Renderer,
        ) -> Option<overlay::Element<'c, Message, Renderer>> {
            self.content.as_widget_mut().overlay(
                self.tree,
                layout.children().next().unwrap(),
                renderer,
            )
        }
    }

    impl<'a, Message, Renderer> From<Modal<'a, Message, Renderer>> for Element<'a, Message, Renderer>
    where
        Renderer: 'a + iced_core::Renderer,
        Message: 'a + Clone,
    {
        fn from(modal: Modal<'a, Message, Renderer>) -> Self {
            Element::new(modal)
        }
    }
}
