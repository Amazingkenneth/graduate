use iced::widget::*;
use iced::*;
mod exchange;
pub fn main() -> iced::Result {
    Memories::run(Settings {
        window: window::Settings {
            size: (1400, 800),
            ..window::Settings::default()
        },
        ..Settings::default()
    })
}

enum Memories {
    // Loading,
    // Errored,
    ChoosingCharacter(ChoosingState),
    ShowingPlots,
    Graduated,
}

pub struct ChoosingState {
    chosen_character: u32,
    on_image: u32,
    idx: Vec<EntryImage>,
    // 当前图片为 idx[on_image]
}
pub struct EntryImage {
    number: u16,
    name: String,
    description: String,
    image: image::Handle,
}

#[derive(Debug)]
enum Message {
    ImageChanged(u32),
}

impl Application for Memories {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;
    type Theme = Theme;

    fn new(_flags: ()) -> (Memories, Command<Message>) {
        (Memories::Loading, Command::none())
    }
    fn title(&self) -> String {
        let subtitle = match self {
            Memories::Loading => "加载中",
            _ => "Whoops!",
        };
        format!("{} - 有你，才是一班。", subtitle)
    }
    fn update(&mut self, message: Message) -> Command<Message> {
        match self {
            Memories::ChoosingCharacter(s) => match message {
                Message::ImageChanged(to) => {
                    s.on_image = to;

                }
            },
        }
    }
    fn view(&self) -> Element<Message> {
        container(image("data/image/grade7/开学合照.jpg"))
            .width(Length::Fill)
            .center_x()
            .into()
    }
}
