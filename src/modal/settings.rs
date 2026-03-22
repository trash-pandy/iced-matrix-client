use iced::widget::space;
use iced::{Element, Subscription, Task};

use crate::app::ViewLike;
use crate::modal::ModalMessage;

crate::msg_adapter_impl!(Message, ModalMessage, Settings);

#[derive(Debug, Clone)]
pub enum Message {}

#[derive(Debug)]
pub struct Modal;

impl ViewLike<ModalMessage> for Modal {
    type Message = Message;

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::none()
    }

    fn update(&mut self, _message: Self::Message) -> Task<Self::Message> {
        Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        space().into()
    }
}
