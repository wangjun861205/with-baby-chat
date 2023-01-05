use crate::models::{Channel, FriendApplication, JoinApplication, User};
use crate::{websocket::WS, Author};
use actix::{Addr, Message};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Login {
    pub phone: String,
    pub password: String,
}

impl Message for Login {
    type Result = ();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub phone: String,
    pub token: String,
    pub err: String,
}

pub struct RepeatLoginWarning;

impl Message for RepeatLoginWarning {
    type Result = ();
}

impl Message for LoginResponse {
    type Result = ();
}

#[derive(Debug, Serialize, Deserialize)]
pub enum NotifyLevel {
    Notify,
    Warning,
    Error,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Input {
    FindUser { phone: String },
    AddFriend { phone: String },
    FindChannel { q: String },
    JoinChannel { cid: i32 },
    FriendApplications { applications: Vec<FriendApplication> },
    JoinApplications { applications: Vec<JoinApplication> },
    ApproveFriend { phone: i32 },
    RejectFriend { phone: i32 },
    ApproveJoin { cid: i32 },
    RejectJoin { cid: i32 },
}

impl Message for Input {
    type Result = ();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InputMessage {
    pub token: String,
    pub input: Input,
}

impl Message for InputMessage {
    type Result = ();
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Result {
    Approved,
    Rejected,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Output {
    LoginResponse { token: String },
    FindUserResponse { uid: i32, username: String },
    AddFriendResponse { user: Option<User> },
    FindChannelResponse { channels: Vec<Channel> },
    JoinChannelResponse { cid: i32, name: String },
    AddFriendResult { uid: i32, result: Result },
    JoinChannelResult { uid: i32, result: Result },
    Notify { level: NotifyLevel, content: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OutputMessage {
    pub output: Output,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Command {
    pub from: String,
    pub input: Input,
}

impl Message for Command {
    type Result = ();
}
