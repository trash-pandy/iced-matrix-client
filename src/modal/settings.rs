use iced::Length::{Fill, Shrink};
use iced::theme::Base;
use iced::widget::{Column, Container, button, container, row, text};
use iced::{Element, Subscription, Task, Theme};

use crate::app::{AppMessenger, ViewLike};
use crate::modal::ModalMessage;
use crate::styling::{MODAL_SHADOW, SPACING_MEDIUM, set_app_theme};

crate::msg_adapter_impl!(Message, ModalMessage, Settings);

#[derive(Debug, Clone)]
pub enum Message {
    Close,
    Theme(Theme),
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
            Message::Theme(theme) => set_app_theme(theme),
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        Container::new(
            Column::new()
                .push(
                    row(Theme::ALL.iter().map(|v| {
                        button(text(v.name()))
                            .on_press(Message::Theme(v.clone()))
                            .into()
                    }))
                    .spacing(SPACING_MEDIUM)
                    .wrap(),
                )
                .push(
                    container(
                        button("close")
                            .style(button::subtle)
                            .on_press(Message::Close),
                    )
                    .align_right(Fill)
                    .align_bottom(Fill),
                ),
        )
        .center_x(Fill)
        .center_y(Shrink)
        .max_width(400)
        .padding(9)
        .style(|theme| container::bordered_box(theme).shadow(MODAL_SHADOW))
        .into()
    }
}
