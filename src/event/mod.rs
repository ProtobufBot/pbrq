use std::sync::atomic::Ordering;
use std::sync::Arc;

use cached::Cached;
use ricq::client::event::{FriendMessageEvent, GroupMessageEvent};
use ricq::handler::QEvent;

use crate::bot;
use crate::bot::Bot;
use crate::idl::pbbot;
use crate::msg::to_proto_chain;
use crate::msg::to_xml::proto_to_xml;

pub async fn to_proto_event(bot: &Arc<Bot>, event: QEvent) -> Option<pbbot::frame::Data> {
    match event {
        QEvent::GroupMessage(e) => {
            tracing::info!(
                "MESSAGE (GROUP={}): {}",
                e.message.group_code,
                e.message.elements
            );
            Some(pbbot::frame::Data::GroupMessageEvent(
                to_proto_group_message(bot, e).await,
            ))
        }
        QEvent::FriendMessage(e) => {
            tracing::info!(
                "MESSAGE (FRIEND={}): {}",
                e.message.from_uin,
                e.message.elements
            );
            Some(pbbot::frame::Data::PrivateMessageEvent(
                to_proto_private_message(bot, e).await,
            ))
        }
        // QEvent::SelfGroupMessage(_) => {}
        // QEvent::TempMessage(_) => {}
        // QEvent::GroupRequest(_) => {}
        // QEvent::SelfInvited(_) => {}
        // QEvent::FriendRequest(_) => {}
        // QEvent::NewMember(_) => {}
        // QEvent::GroupMute(_) => {}
        // QEvent::FriendMessageRecall(_) => {}
        // QEvent::GroupMessageRecall(_) => {}
        // QEvent::NewFriend(_) => {}
        // QEvent::GroupLeave(_) => {}
        // QEvent::FriendPoke(_) => {}
        // QEvent::GroupNameUpdate(_) => {}
        // QEvent::DeleteFriend(_) => {}
        // QEvent::MemberPermissionChange(_) => {}
        _ => None,
    }
}

pub async fn to_proto_group_message(
    bot: &Arc<Bot>,
    event: GroupMessageEvent,
) -> pbbot::GroupMessageEvent {
    let client = event.client;
    let message = event.message;
    let message_id = bot.message_id.fetch_add(1, Ordering::Relaxed);
    bot.message_cache.write().await.cache_set(
        message_id,
        bot::Message {
            time: message.time,
            from_uin: message.from_uin,
            from_group: Some(message.group_code),
            elements: message.elements.clone(),
            seqs: message.seqs.clone(),
            rans: message.rands.clone(),
        },
    );
    let proto_message = to_proto_chain(&client, message.elements);
    let raw_message = proto_to_xml(proto_message.clone());
    pbbot::GroupMessageEvent {
        time: message.time as i64,
        self_id: client.uin().await,
        post_type: "message".to_string(),
        message_type: "group".to_string(),
        sub_type: "normal".to_string(),
        message_id,
        group_id: message.group_code,
        user_id: message.from_uin,
        anonymous: None, // TODO
        raw_message,
        message: proto_message,
        sender: Some(pbbot::group_message_event::Sender {
            user_id: message.from_uin,
            card: message.group_card,
            ..Default::default()
        }),
        font: 0,
        extra: Default::default(),
    }
}

pub async fn to_proto_private_message(
    bot: &Arc<Bot>,
    event: FriendMessageEvent,
) -> pbbot::PrivateMessageEvent {
    let client = event.client;
    let message = event.message;
    let message_id = bot.message_id.fetch_add(1, Ordering::Relaxed);
    bot.message_cache.write().await.cache_set(
        message_id,
        bot::Message {
            time: message.time,
            from_uin: message.from_uin,
            from_group: None,
            elements: message.elements.clone(),
            seqs: message.seqs.clone(),
            rans: message.rands.clone(),
        },
    );
    let proto_message = to_proto_chain(&client, message.elements);
    let raw_message = proto_to_xml(proto_message.clone());
    pbbot::PrivateMessageEvent {
        time: message.time as i64,
        self_id: client.uin().await,
        post_type: "message".to_string(),
        message_type: "private".to_string(),
        sub_type: "normal".to_string(),
        message_id,
        user_id: message.from_uin,
        raw_message,
        message: proto_message,
        sender: Some(pbbot::private_message_event::Sender {
            user_id: message.from_uin,
            nickname: message.from_nick,
            ..Default::default()
        }),
        font: 0,
        extra: Default::default(),
    }
}
