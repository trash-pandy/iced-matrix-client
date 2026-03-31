use std::str::FromStr;

use iced::Length::{Fill, Shrink};
use iced::alignment::Horizontal::Left;
use iced::widget::{Column, button, container, text, text_input};
use iced::{Element, Subscription, Task};
use matrix_sdk::authentication::matrix::MatrixSession;
use matrix_sdk::config::SyncSettings;
use matrix_sdk::ruma::{OwnedDeviceId, OwnedUserId};
use matrix_sdk::{SessionMeta, SessionTokens};
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use crate::app::{AppMessenger, ViewLike};
use crate::extensions::PushMaybe;
use crate::page::{self, PageMessage};
use crate::styling::{
    FONT_MEDIUM, SPACING_LARGE, SPACING_MEDIUM, TEXT_LARGE, TEXT_SMALL, labelled,
};

crate::msg_adapter_impl!(Message, PageMessage, Login);

#[derive(Debug, Clone)]
pub enum Message {
    RestoreSessionFailed,
    UpdateUsername(String),
    UpdateHomeserver(String),
    UpdatePassword(Zeroizing<String>),
    Login,
    DoneLogin(matrix_sdk::Client),
    Error(String),
}

#[derive(Debug, Clone)]
pub struct Page {
    homeserver: String,
    username: String,
    password: Zeroizing<String>,
    error: Option<String>,
    messenger: AppMessenger,
    logging_in: bool,
}

impl Page {
    pub fn new(messenger: AppMessenger) -> (Self, Task<Message>) {
        let page = Self {
            homeserver: String::new(),
            username: String::new(),
            password: Zeroizing::default(),
            error: None,
            messenger,
            logging_in: true,
        };

        (page, Task::future(restore_session()))
    }
}

async fn restore_session() -> Message {
    let Ok(login_info) = std::fs::read_to_string("./app-data/login") else {
        return Message::RestoreSessionFailed;
    };
    let Ok(login_info) = serde_json::from_str::<LoginInfo>(&login_info) else {
        return Message::RestoreSessionFailed;
    };
    let Ok(client) = matrix_sdk::Client::builder()
        .server_name_or_homeserver_url(login_info.homeserver)
        .sqlite_store("./app-data/db", None)
        .build()
        .await
    else {
        return Message::RestoreSessionFailed;
    };
    let login = client
        .matrix_auth()
        .restore_session(
            MatrixSession {
                meta: SessionMeta {
                    user_id: OwnedUserId::from_str(&login_info.user_id).unwrap(),
                    device_id: OwnedDeviceId::from(login_info.device_id),
                },
                tokens: SessionTokens {
                    access_token: login_info.access_token,
                    refresh_token: login_info.refresh_token,
                },
            },
            matrix_sdk::store::RoomLoadSettings::All,
        )
        .await;
    client.sync_once(SyncSettings::default()).await.ok();
    match login {
        Ok(()) => Message::DoneLogin(client),
        Err(e) => Message::Error(e.to_string()),
    }
}

impl ViewLike<PageMessage> for Page {
    type Message = Message;

    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::RestoreSessionFailed => {
                self.logging_in = false;
                Task::none()
            }
            Message::UpdateUsername(username) => {
                self.username = username;
                Task::none()
            }
            Message::UpdateHomeserver(homeserver) => {
                self.homeserver = homeserver;
                Task::none()
            }
            Message::UpdatePassword(password) => {
                self.password = password;
                Task::none()
            }
            Message::Login => {
                self.logging_in = true;
                let homeserver = self.homeserver.clone();
                let username = self.username.clone();
                let password = self.password.clone();
                Task::future(async move {
                    let client = matrix_sdk::Client::builder()
                        .server_name_or_homeserver_url(homeserver)
                        .build()
                        .await;
                    if let Err(e) = client {
                        return Message::Error(e.to_string());
                    }
                    let client = client.unwrap();
                    let login = client
                        .matrix_auth()
                        .login_username(username, &password)
                        .initial_device_display_name("iced-matrix-client")
                        .request_refresh_token()
                        .await;
                    client.sync_once(SyncSettings::default()).await.ok();
                    match login {
                        Ok(login) => {
                            std::fs::create_dir("app-data").unwrap();
                            std::fs::write(
                                "./app-data/login",
                                serde_json::to_string(&LoginInfo {
                                    homeserver: login.user_id.server_name().to_string(),
                                    user_id: login.user_id.to_string(),
                                    device_id: login.device_id.to_string(),
                                    access_token: login.access_token.clone(),
                                    refresh_token: login.refresh_token.clone(),
                                })
                                .unwrap(),
                            )
                            .unwrap();
                            Message::DoneLogin(client)
                        }
                        Err(e) => Message::Error(e.to_string()),
                    }
                })
            }
            Message::DoneLogin(client) => {
                self.messenger
                    .switch_page(move |init| page::chat::Page::boot(init, &client));
                Task::none()
            }
            Message::Error(e) => {
                self.error = Some(e);
                self.logging_in = false;
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        container(
            container(
                Column::new()
                    .push(
                        text("iced-matrix-client")
                            .font(FONT_MEDIUM)
                            .center()
                            .width(Fill)
                            .size(TEXT_LARGE),
                    )
                    .push(labelled(
                        "Homeserver",
                        text_input("bash.org", &self.homeserver)
                            .size(TEXT_SMALL)
                            .on_input(Message::UpdateHomeserver),
                    ))
                    .push(labelled(
                        "Username",
                        text_input("azurediamond", &self.username)
                            .size(TEXT_SMALL)
                            .on_input(Message::UpdateUsername),
                    ))
                    .push(labelled(
                        "Password",
                        text_input("hunter2", &self.password)
                            .secure(true)
                            .size(TEXT_SMALL)
                            .on_input(|p| Message::UpdatePassword(p.into())),
                    ))
                    .push_maybe(self.error.as_ref().map(|e| {
                        container(text(e.as_str()).align_x(Left))
                            .padding(SPACING_LARGE)
                            .style(container::danger)
                            .width(Fill)
                    }))
                    .push(
                        container(
                            button("Login")
                                .on_press_maybe((!self.logging_in).then(|| Message::Login)),
                        )
                        .align_right(Fill),
                    )
                    .spacing(SPACING_LARGE),
            )
            .style(container::bordered_box)
            .height(Shrink)
            .center_x(240)
            .padding(SPACING_MEDIUM),
        )
        .center(Shrink)
        .width(Fill)
        .height(Fill)
        .into()
    }
}

#[derive(Serialize, Deserialize)]
struct LoginInfo {
    homeserver: String,
    user_id: String,
    device_id: String,
    access_token: String,
    refresh_token: Option<String>,
}
