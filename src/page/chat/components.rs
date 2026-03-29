use std::string::ToString;

use iced::Length::Fill;
use iced::widget::text::Wrapping;
use iced::widget::{
    Column, button, column, container, image, markdown, rule, scrollable, space, text, text_input,
    tooltip,
};
use iced::{Element, Pixels, Theme};
use itertools::Itertools;
use matrix_sdk::Room;
use unicode_segmentation::UnicodeSegmentation;

use super::{Message, Page};
use crate::extensions::PushMaybe;
use crate::page::chat::message::RenderedMessage;
use crate::styling::{
    FONT_BOLD, SPACING_LARGE, SPACING_MEDIUM, SPACING_SMALL, TEXT_LARGE, TEXT_MED, TEXT_SMALL,
    get_app_theme,
};

const ROOM_IMAGE_SIZE: f32 = 36.0;
const CHANNEL_LIST_WIDTH: Pixels = Pixels(240.0);

pub fn space_list(page: &Page) -> Element<'_, Message> {
    container(column([
        scrollable(
            page.client
                .joined_space_rooms()
                .iter()
                .sorted_by_key(|v| v.room_id())
                .map(|room| room_image(page, room, Message::OpenSpace(room.room_id().to_owned())))
                .collect::<Column<_>>()
                .spacing(SPACING_LARGE),
        )
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::hidden(),
        ))
        .height(Fill)
        .into(),
        button(text("s").center())
            .width(ROOM_IMAGE_SIZE)
            .height(ROOM_IMAGE_SIZE)
            .style(button::subtle)
            .on_press(Message::OpenSettings)
            .into(),
    ]))
    .padding(SPACING_LARGE)
    .align_left(ROOM_IMAGE_SIZE + SPACING_LARGE.0 * 2.0)
    .height(Fill)
    .into()
}

pub fn room_list(page: &Page) -> Element<'_, Message> {
    container(
        Column::new()
            .push_maybe(
                page.current_space
                    .as_ref()
                    .and_then(|space_id| page.client.get_room(space_id.as_ref()))
                    .and_then(|room| {
                        room.cached_display_name()
                            .map(|v| v.to_string())
                            .or_else(|| room.name())
                            .or_else(|| Some(room.room_id().to_string()))
                    })
                    .map(text),
            )
            .extend([
                rule::horizontal(SPACING_SMALL).into(),
                scrollable(
                    page.space_rooms
                        .iter()
                        .filter_map(|room| page.client.get_room(room))
                        .sorted_by_key(|room| room.room_id().to_owned())
                        .map(|room| {
                            button(text(room_name(&room)).wrapping(Wrapping::WordOrGlyph))
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
        .push(text("room description").wrapping(Wrapping::None).center())
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

pub fn message(msg: &RenderedMessage) -> Element<'_, Message> {
    column([
        text(msg.sender.clone()).font(FONT_BOLD).into(),
        markdown::view(
            &msg.content,
            markdown::Settings {
                text_size: TEXT_MED,
                h1_size: TEXT_LARGE,
                h2_size: TEXT_LARGE,
                h3_size: TEXT_MED,
                h4_size: TEXT_MED,
                h5_size: TEXT_SMALL,
                h6_size: TEXT_SMALL,
                code_size: TEXT_MED,
                spacing: SPACING_SMALL,
                style: markdown::Style::from(get_app_theme()),
            },
        )
        .map(Message::UrlClicked),
    ])
    .into()
}

fn room_image<'a, M: 'a + Clone>(state: &Page, room: &Room, on_press: M) -> Element<'a, M> {
    let content: Element<'a, M> = state
        .room_avatars
        .get(&room.room_id().to_owned())
        .cloned()
        .flatten()
        .map_or_else(
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
