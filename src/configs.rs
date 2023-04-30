use crate::audio::AudioStream;
use crate::{visiting, Message, Stage, State};
use iced::widget::{
    self, column, container, horizontal_space, image, row, text, vertical_space, Column, Row,
};
use iced::{alignment, subscription, Alignment, Length, Theme};
// use iced_audio::core::normal_param::NormalParam;
// use iced_audio::native::h_slider::HSlider;
use std::io::Write;
use std::mem::ManuallyDrop;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use time;

#[derive(Clone, Debug)]
pub struct Configs {
    pub shown: bool,
    pub full_screened: bool,
    pub config_path: String,
    pub scale_factor: f64,
    pub theme: Theme,
    pub from_date: visiting::ShootingTime,
    pub volume_percentage: f32,
    pub aud_module: Arc<std::sync::Mutex<ManuallyDrop<AudioStream>>>,
    pub audio_paths: Vec<String>,
    pub daemon_running: Arc<AtomicBool>,
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
                            crate::button_from_svg(include_bytes!("./runtime/plus.svg").to_vec(),)
                                .width(Length::Fixed(40.0))
                                .on_press(Message::ScaleEnlarge),
                            "放大",
                            widget::tooltip::Position::Bottom,
                        )
                        .style(iced::theme::Container::Box),
                        widget::tooltip(
                            crate::button_from_svg(include_bytes!("./runtime/minus.svg").to_vec(),)
                                .width(Length::Fixed(40.0))
                                .on_press(Message::ScaleDown),
                            "缩小",
                            widget::tooltip::Position::Bottom,
                        )
                        .style(iced::theme::Container::Box),
                        if config.full_screened {
                            widget::tooltip(
                                crate::button_from_svg(
                                    include_bytes!("./runtime/compress.svg").to_vec(),
                                )
                                .width(Length::Fixed(40.0))
                                .on_press(Message::ToggleMode),
                                "窗口显示",
                                widget::tooltip::Position::Bottom,
                            )
                            .style(iced::theme::Container::Box)
                        } else {
                            widget::tooltip(
                                crate::button_from_svg(
                                    include_bytes!("./runtime/expand.svg").to_vec(),
                                )
                                .width(Length::Fixed(40.0))
                                .on_press(Message::ToggleMode),
                                "全屏显示",
                                widget::tooltip::Position::Bottom,
                            )
                            .style(iced::theme::Container::Box)
                        },
                        if config
                            .daemon_running
                            .load(std::sync::atomic::Ordering::Relaxed)
                            && !config.aud_module.lock().unwrap().sink.is_paused()
                        {
                            row![
                                widget::tooltip(
                                    crate::button_from_svg(
                                        include_bytes!("./runtime/pause.svg").to_vec(),
                                    )
                                    .width(Length::Fixed(40.0))
                                    .on_press(Message::SwitchMusicStatus),
                                    "播放 / 暂停",
                                    widget::tooltip::Position::Bottom
                                )
                                .style(iced::theme::Container::Box)
                                .gap(5),
                                widget::tooltip(
                                    crate::button_from_svg(
                                        include_bytes!("./runtime/square-right.svg").to_vec()
                                    )
                                    .width(Length::Fixed(40.0))
                                    .on_press(Message::NextSong),
                                    "跳到下一首",
                                    widget::tooltip::Position::Bottom
                                )
                                .style(iced::theme::Container::Box)
                            ]
                        } else {
                            row![widget::tooltip(
                                crate::button_from_svg(
                                    include_bytes!("./runtime/play.svg").to_vec(),
                                )
                                .width(Length::Fixed(40.0))
                                .on_press(Message::SwitchMusicStatus),
                                "播放 / 暂停",
                                widget::tooltip::Position::Bottom
                            )
                            .style(iced::theme::Container::Box)
                            .gap(5)]
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
        toml::Value::Float(configs.scale_factor),
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
        String::from("audio-playing"),
        toml::Value::Boolean(configs.daemon_running.load(Ordering::Relaxed)),
    );
    let mut buffer = std::fs::File::create(state.configs.config_path.clone()).unwrap();
    buffer
        .write_all(toml::to_string_pretty(&map).unwrap().as_bytes())
        .unwrap();
}

mod modal {
    use iced_native::alignment::Alignment;
    use iced_native::widget::{self, Tree};
    use iced_native::{
        event, layout, mouse, overlay, renderer, Clipboard, Color, Element, Event, Layout, Length,
        Point, Rectangle, Shell, Size, Widget,
    };

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
        Renderer: iced_native::Renderer,
        Message: Clone,
    {
        fn children(&self) -> Vec<Tree> {
            vec![Tree::new(&self.base), Tree::new(&self.modal)]
        }

        fn diff(&self, tree: &mut Tree) {
            tree.diff_children(&[&self.base, &self.modal]);
        }

        fn width(&self) -> Length {
            self.base.as_widget().width()
        }

        fn height(&self) -> Length {
            self.base.as_widget().height()
        }

        fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
            self.base.as_widget().layout(renderer, limits)
        }

        fn on_event(
            &mut self,
            state: &mut Tree,
            event: Event,
            layout: Layout<'_>,
            cursor_position: Point,
            renderer: &Renderer,
            clipboard: &mut dyn Clipboard,
            shell: &mut Shell<'_, Message>,
        ) -> event::Status {
            self.base.as_widget_mut().on_event(
                &mut state.children[0],
                event,
                layout,
                cursor_position,
                renderer,
                clipboard,
                shell,
            )
        }

        fn draw(
            &self,
            state: &Tree,
            renderer: &mut Renderer,
            theme: &<Renderer as iced_native::Renderer>::Theme,
            style: &renderer::Style,
            layout: Layout<'_>,
            cursor_position: Point,
            viewport: &Rectangle,
        ) {
            self.base.as_widget().draw(
                &state.children[0],
                renderer,
                theme,
                style,
                layout,
                cursor_position,
                viewport,
            );
        }

        fn overlay<'b>(
            &'b mut self,
            state: &'b mut Tree,
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
            state: &Tree,
            layout: Layout<'_>,
            cursor_position: Point,
            viewport: &Rectangle,
            renderer: &Renderer,
        ) -> mouse::Interaction {
            self.base.as_widget().mouse_interaction(
                &state.children[0],
                layout,
                cursor_position,
                viewport,
                renderer,
            )
        }

        fn operate(
            &self,
            state: &mut Tree,
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
        tree: &'b mut Tree,
        size: Size,
        on_blur: Option<Message>,
    }

    impl<'a, 'b, Message, Renderer> overlay::Overlay<Message, Renderer>
        for Overlay<'a, 'b, Message, Renderer>
    where
        Renderer: iced_native::Renderer,
        Message: Clone,
    {
        fn layout(&self, renderer: &Renderer, _bounds: Size, position: Point) -> layout::Node {
            let limits = layout::Limits::new(Size::ZERO, self.size)
                .width(Length::Fill)
                .height(Length::Fill);

            let mut child = self.content.as_widget().layout(renderer, &limits);
            child.align(Alignment::Center, Alignment::Center, limits.max());

            let mut node = layout::Node::with_children(self.size, vec![child]);
            node.move_to(position);

            node
        }

        fn on_event(
            &mut self,
            event: Event,
            layout: Layout<'_>,
            cursor_position: Point,
            renderer: &Renderer,
            clipboard: &mut dyn Clipboard,
            shell: &mut Shell<'_, Message>,
        ) -> event::Status {
            let content_bounds = layout.children().next().unwrap().bounds();

            if let Some(message) = self.on_blur.as_ref() {
                if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = &event {
                    if !content_bounds.contains(cursor_position) {
                        shell.publish(message.clone());
                        return event::Status::Captured;
                    }
                }
            }

            self.content.as_widget_mut().on_event(
                self.tree,
                event,
                layout.children().next().unwrap(),
                cursor_position,
                renderer,
                clipboard,
                shell,
            )
        }

        fn draw(
            &self,
            renderer: &mut Renderer,
            theme: &Renderer::Theme,
            style: &renderer::Style,
            layout: Layout<'_>,
            cursor_position: Point,
        ) {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: layout.bounds(),
                    border_radius: renderer::BorderRadius::from(0.0),
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
                cursor_position,
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
            cursor_position: Point,
            viewport: &Rectangle,
            renderer: &Renderer,
        ) -> mouse::Interaction {
            self.content.as_widget().mouse_interaction(
                self.tree,
                layout.children().next().unwrap(),
                cursor_position,
                viewport,
                renderer,
            )
        }
    }

    impl<'a, Message, Renderer> From<Modal<'a, Message, Renderer>> for Element<'a, Message, Renderer>
    where
        Renderer: 'a + iced_native::Renderer,
        Message: 'a + Clone,
    {
        fn from(modal: Modal<'a, Message, Renderer>) -> Self {
            Element::new(modal)
        }
    }
}
