use std::string::ToString;

use iced::Length::Fill;
use iced::widget::{
    Column, button, column, container, image, rule, scrollable, space, text, text_input, tooltip,
};
use iced::{Element, Pixels, Theme};
use matrix_sdk::Room;
use unicode_segmentation::UnicodeSegmentation;

use super::{Message, Page};
use crate::styling::{FONT_BOLD, SPACING_LARGE, SPACING_MEDIUM, SPACING_SMALL};
use crate::extensions::ColumnExt;
use crate::worker::messages::MessageContent;

const ROOM_IMAGE_SIZE: f32 = 36.0;
const CHANNEL_LIST_WIDTH: Pixels = Pixels(240.0);

pub fn space_list(page: &Page) -> Element<'_, Message> {
    container(
        scrollable(
            Column::new()
                .extend(page.client.joined_space_rooms().iter().map(|room| {
                    room_image(page, room, Message::OpenSpace(room.room_id().to_owned()))
                }))
                .spacing(SPACING_LARGE),
        )
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::hidden(),
        )),
    )
    .padding(SPACING_LARGE)
    .align_left(ROOM_IMAGE_SIZE + SPACING_LARGE.0 * 2.0)
    .height(Fill)
    .into()
}

pub fn channel_list(page: &Page) -> Element<'_, Message> {
    container(
        column([
            text("rooms").into(),
            rule::horizontal(SPACING_SMALL).into(),
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
                    .spacing(SPACING_LARGE),
            )
            .spacing(SPACING_LARGE)
            .into(),
        ])
        .push_maybe(page.needs_verification.then(|| {
            column([
                text("This device is not verified").into(),
                button("Verify").into(),
            ])
            .into()
        }))
        .spacing(SPACING_LARGE),
    )
    .padding(SPACING_LARGE)
    .align_left(CHANNEL_LIST_WIDTH)
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
        .push(rule::horizontal(SPACING_SMALL))
        .push(
            scrollable(messages_pane(page))
                .height(Fill)
                .width(Fill)
                .spacing(SPACING_SMALL),
        )
        .push(container(
            text_input("Send a message...", &page.text).on_input(Message::UpdateMessage),
        ))
        .spacing(SPACING_LARGE)
        .padding(SPACING_LARGE)
        .into()
}

pub fn messages_pane(page: &Page) -> Element<'_, Message> {
    page.current_room.as_ref().map_or_else(
        || space().into(),
        |room_id| {
            page.messages.get(room_id).map_or_else(
                || space().height(Fill).into(),
                |messages| {
                    column(messages.iter().map(message))
                        .height(Fill)
                        .spacing(SPACING_LARGE)
                        .into()
                },
            )
        },
    )
}

pub fn message(msg: &MessageContent) -> Element<'_, Message> {
    column([
        text(
            msg.sender
                .as_ref()
                .map_or_else(|| "system".to_owned(), ToString::to_string),
        )
        .font(FONT_BOLD)
        .into(),
        text(msg.message_content.clone()).into(),
    ])
    .into()
}

fn room_image<'a, M: 'a + Clone>(state: &Page, room: &Room, on_press: M) -> Element<'a, M> {
    let content: Element<'a, M> = state.avatars.get(&room.room_id().to_owned()).map_or_else(
        || {
            text(room_short_name(room))
                .width(ROOM_IMAGE_SIZE)
                .height(ROOM_IMAGE_SIZE)
                .center()
                .into()
        },
        |handle| {
            image(handle)
                .width(ROOM_IMAGE_SIZE)
                .height(ROOM_IMAGE_SIZE)
                .into()
        },
    );
    tooltip(
        button(content)
            .padding(0)
            .clip(true)
            .style(room_image_button_style)
            .on_press(on_press),
        container(text(room_name(room)))
            .padding(SPACING_MEDIUM)
            .style(container::secondary),
        tooltip::Position::Right,
    )
    .into()
}

fn room_image_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::secondary(theme, status);
    let palette = theme.extended_palette();
    style.border = style.border.rounded(ROOM_IMAGE_SIZE);
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
