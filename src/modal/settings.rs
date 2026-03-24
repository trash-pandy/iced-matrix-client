use iced::widget::{Container, button, container};
use iced::{Element, Subscription, Task};

use crate::app::{AppMessenger, ViewLike};
use crate::modal::ModalMessage;

crate::msg_adapter_impl!(Message, ModalMessage, Settings);

#[derive(Debug, Clone)]
pub enum Message {
    Close,
}

#[derive(Debug, Clone)]
pub struct Modal {
    messenger: AppMessenger,
}

impl Modal {
    pub fn boot(messenger: AppMessenger) -> (Self, Task<Message>) {
        (Self { messenger }, Task::none())
    }
}

impl ViewLike<ModalMessage> for Modal {
    type Message = Message;

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::none()
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::Close => self.messenger.close_modal(),
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        Container::new(button("close").on_press(Message::Close))
            .width(400)
            .height(300)
            .style(container::success)
            .into()
    }
}
