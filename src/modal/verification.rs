use iced::Length::{Fill, Shrink};
use iced::widget::{Container, button, column, container, text};
use iced::{Element, Subscription, Task};

use crate::app::{AppMessenger, ViewLike};
use crate::modal::ModalMessage;
use crate::styling::MODAL_SHADOW;

crate::msg_adapter_impl!(Message, ModalMessage, Verification);

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
        Container::new(column([
            text("verification modal").into(),
            button("close").into(),
        ]))
        .center_x(Fill)
        .center_y(Shrink)
        .max_width(400)
        .padding(9)
        .style(|theme| container::bordered_box(theme).shadow(MODAL_SHADOW))
        .into()
    }
}
