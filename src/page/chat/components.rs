use iced::Length::Fill;
use iced::widget::{
    Column, button, column, container, image, rule, scrollable, space, text, text_input, tooltip,
};
use iced::{Element, Theme};
use matrix_sdk::Room;
use unicode_segmentation::UnicodeSegmentation;

use super::{Message, Page};
use crate::extensions::ColumnExt;

pub fn space_list(page: &Page) -> Element<'_, Message> {
    container(
        scrollable(
            Column::new()
                .extend(page.client.joined_space_rooms().iter().map(|room| {
                    room_image(page, room, Message::OpenSpace(room.room_id().to_owned()))
                }))
                .spacing(8),
        )
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::hidden(),
        )),
    )
    .padding(8)
    .align_left(52)
    .height(Fill)
    .into()
}

pub fn channel_list(page: &Page) -> Element<'_, Message> {
    container(
        column([
            text("rooms").into(),
            rule::horizontal(2).into(),
            scrollable(
                page.space_rooms
                    .iter()
                    .filter_map(|room| page.client.get_room(room))
                    .map(|room| {
                        button(text(room_name(&room)).wrapping(text::Wrapping::WordOrGlyph))
                            .style(button::subtle)
                            .width(Fill)
                            .on_press(Message::OpenRoom(room.room_id().to_owned()))
                            .into()
                    })
                    .collect::<Column<_>>()
                    .spacing(8),
            )
            .spacing(8)
            .into(),
        ])
        .push_maybe(page.needs_verification.then(|| {
            column([
                text("This device is not verified").into(),
                button("Verify").into(),
            ])
            .into()
        }))
        .spacing(8),
    )
    .padding(8)
    .align_left(240)
    .height(Fill)
    .style(|theme: &Theme| {
        let p = theme.extended_palette();
        container::Style {
            text_color: Some(p.background.weakest.text),
            background: Some(p.background.weakest.color.into()),
            ..Default::default()
        }
    })
    .into()
}

pub fn room_pane(page: &Page) -> Element<'_, Message> {
    Column::new()
        .push(
            text("room description")
                .wrapping(text::Wrapping::None)
                .center(),
        )
        .push(rule::horizontal(2))
        .push(
            scrollable(messages_pane(page))
                .height(Fill)
                .width(Fill)
                .spacing(2.),
        )
        .push(container(
            text_input("Send a message...", &page.text).on_input(Message::UpdateMessage),
        ))
        .spacing(8)
        .padding(8)
        .into()
}

pub fn messages_pane(page: &Page) -> Element<'_, Message> {
    page.current_room.as_ref().map_or_else(
        || space().into(),
        |room_id| {
            page.messages.get(room_id).map_or_else(
                || space().height(Fill).into(),
                |messages| {
                    column(messages.iter().map(|v| {
                        text(v.sender.as_ref().map_or_else(
                            || format!("//// {}", v.message_content),
                            |sender| format!("{}: {}", sender, v.message_content),
                        ))
                        .into()
                    }))
                    .height(Fill)
                    .into()
                },
            )
        },
    )
}

fn room_image<'a, M: 'a + Clone>(state: &Page, room: &Room, on_press: M) -> Element<'a, M> {
    let content: Element<'a, M> = state.avatars.get(&room.room_id().to_owned()).map_or_else(
        || {
            text(room_short_name(room))
                .width(36)
                .height(36)
                .center()
                .into()
        },
        |handle| image(handle).width(36).height(36).into(),
    );
    tooltip(
        button(content)
            .padding(0)
            .clip(true)
            .style(room_image_button_style)
            .on_press(on_press),
        container(text(room_name(room)))
            .padding(4.0)
            .style(container::secondary),
        tooltip::Position::Right,
    )
    .into()
}

fn room_image_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::secondary(theme, status);
    let palette = theme.extended_palette();
    style.border = style.border.rounded(36);
    style.with_background(
        match status {
            button::Status::Active => palette.background.weakest,
            button::Status::Hovered => palette.background.strong,
            button::Status::Pressed => palette.background.strongest,
            button::Status::Disabled => palette.background.neutral,
        }
        .color,
    )
}

fn room_short_name(room: &Room) -> String {
    let name = room_name(room);
    name.graphemes(true)
        .flat_map(|v| v.chars().filter(|v| v.is_alphanumeric()))
        .take(2)
        .collect()
}

fn room_name(room: &Room) -> String {
    room.cached_display_name()
        .map(|v| v.to_string())
        .or_else(|| room.name())
        .unwrap_or_else(|| "Unknown Room".to_owned())
}
