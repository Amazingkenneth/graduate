use iced::widget::*;
use iced::*;
pub fn main() -> iced::Result {
    Graduate::run(Settings {
        window: window::Settings {
            size: (1400, 800),
            ..window::Settings::default()
        },
        ..Settings::default()
    })
}

enum Graduate {
    Loading,
    ChoosingCharacter(ChoosingState),
    ShowingPlots,
    Graduated,
}

struct ChoosingState {
    chosen_character: i32,
    on_image: i32,
    idx: Vec<EntryImage>,
}
struct EntryImage {}

#[derive(Debug, Clone)]
enum StageMessage {
    ImageChanged(u32),
    CharacterChanged(u32),
    ImputChanged(u32),
}
impl Application for Graduate {
    type Executor = executor::Default;
    type Flags = ();
    type Message = StageMessage;
    type Theme = Theme;

    fn new(_flags: ()) -> (Graduate, Command<Self::Message>) {
        (Graduate::Loading, Command::none())
    }

    fn title(&self) -> String {
        let subtitle = match self {
            Graduate::Loading => "Loading",
            Graduate::Loaded { pokemon, .. } => &pokemon.name,
            Graduate::Errored { .. } => "Whoops!",
        };

        format!("{} - 有你，才是一班。", subtitle)
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::PlotFound(Ok(pokemon)) => {
                *self = Graduate::Loaded { pokemon };

                Command::none()
            }
            Message::PlotFound(Err(_error)) => {
                *self = Graduate::Errored;

                Command::none()
            }
            Message::Search => match self {
                Graduate::Loading => Command::none(),
                _ => {
                    *self = Graduate::Loading;

                    Command::perform(Plot::search(), Message::PokemonFound)
                }
            },
        }
    }

    fn view(&self) -> Element<Message> {
        let content = match self {
            Graduate::Loading => {
                column![text("Searching for the next photo").size(40),]
                    .width(Length::Shrink)
            }
            Graduate::Loaded { pokemon } => column![
                pokemon.view(),
                button("Keep searching!").on_press(Message::Search)
            ]
            .max_width(500)
            .spacing(20)
            .align_items(Alignment::End),
            Graduate::Errored => column![
                text("Whoops! Something went wrong...").size(40),
                button("Try again").on_press(Message::Search)
            ]
            .spacing(20)
            .align_items(Alignment::End),
        };

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
