use std::string::ToString;

use iced::widget::markdown;

use crate::worker::messages::MessageContent;

#[derive(Debug, Clone)]
pub struct RenderedMessage {
    pub sender: String,
    pub content: Vec<markdown::Item>,
}

pub fn render_message_content(msg: &MessageContent) -> RenderedMessage {
    RenderedMessage {
        sender: msg
            .sender
            .as_ref()
            .map_or_else(|| "# system".to_owned(), ToString::to_string),
        content: markdown::parse(&msg.content).collect(),
    }
}
