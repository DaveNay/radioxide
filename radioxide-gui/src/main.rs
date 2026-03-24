use iced::widget::text;
use iced::Element;

fn main() -> iced::Result {
    iced::application("Radioxide GUI", RadioxideGUI::update, RadioxideGUI::view)
        .run_with(|| (RadioxideGUI, iced::Task::none()))
}

struct RadioxideGUI;

#[derive(Debug, Clone, Copy)]
enum Message {}

impl RadioxideGUI {
    fn update(&mut self, _message: Message) -> iced::Task<Message> {
        iced::Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        text("Welcome to Radioxide GUI").into()
    }
}
