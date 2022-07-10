use std::collections::HashMap;
use std::sync::Arc;

use ricq::client::event::{
    FriendMessageEvent, FriendMessageRecallEvent, GroupLeaveEvent, GroupMessageEvent,
    GroupMessageRecallEvent, GroupMuteEvent, JoinGroupRequestEvent, MemberPermissionChangeEvent,
    NewFriendEvent, NewFriendRequestEvent, NewMemberEvent, SelfInvitedEvent,
};
use ricq::handler::QEvent;
use ricq::structs::GroupMemberPermission;

use crate::bot::Bot;
use crate::idl::pbbot;
use crate::idl::pbbot::MessageReceipt;
use crate::msg::to_proto_chain;
use crate::msg::to_xml::proto_to_xml;

pub async fn to_proto_event(bot: &Arc<Bot>, event: QEvent) -> Option<pbbot::frame::Data> {
    match event {
        QEvent::GroupMessage(e) => {
            tracing::info!(
                "MESSAGE (GROUP={}): {}",
                e.inner.group_code,
                e.inner.elements
            );
            Some(pbbot::frame::Data::GroupMessageEvent(
                to_proto_group_message(bot, e).await,
            ))
        }
        QEvent::FriendMessage(e) => {
            tracing::info!(
                "MESSAGE (FRIEND={}): {}",
                e.inner.from_uin,
                e.inner.elements
            );
            Some(pbbot::frame::Data::PrivateMessageEvent(
                to_proto_private_message(bot, e).await,
            ))
        }
        // QEvent::TempMessage(_) => {}
        QEvent::GroupRequest(e) => {
            tracing::info!(
                "GROUP_REQUEST (GROUP={}): {}",
                e.inner.group_code,
                e.inner.req_uin
            );
            Some(pbbot::frame::Data::GroupRequestEvent(
                to_proto_group_request(bot, e).await,
            ))
        }
        QEvent::SelfInvited(e) => {
            tracing::info!(
                "SELF_INVITED (GROUP={}): {}",
                e.inner.group_code,
                e.inner.invitor_uin
            );
            Some(pbbot::frame::Data::GroupRequestEvent(
                to_proto_self_group_request(bot, e).await,
            ))
        }
        QEvent::NewFriendRequest(e) => {
            tracing::info!(
                "FRIEND_REQUEST (UIN={}): {}",
                e.inner.req_uin,
                e.inner.message
            );
            Some(pbbot::frame::Data::FriendRequestEvent(
                to_proto_friend_request(bot, e).await,
            ))
        }
        QEvent::NewMember(e) => {
            tracing::info!(
                "NEW_MEMBER (GROUP={}): {}",
                e.inner.group_code,
                e.inner.member_uin
            );
            Some(pbbot::frame::Data::GroupIncreaseNoticeEvent(
                to_proto_group_increase(bot, e).await,
            ))
        }
        QEvent::GroupMute(e) => {
            tracing::info!(
                "GROUP_MUTE (GROUP={}): {}",
                e.inner.group_code,
                e.inner.target_uin
            );
            Some(pbbot::frame::Data::GroupBanNoticeEvent(
                to_proto_group_ban(bot, e).await,
            ))
        }
        QEvent::FriendMessageRecall(e) => {
            tracing::info!(
                "FRIEND_RECALL (FRIEND={}): {}",
                e.inner.friend_uin,
                e.inner.msg_seq
            );
            Some(pbbot::frame::Data::FriendRecallNoticeEvent(
                to_proto_friend_recall(bot, e).await,
            ))
        }
        QEvent::GroupMessageRecall(e) => {
            tracing::info!(
                "GROUP_RECALL (GROUP={}): {}",
                e.inner.group_code,
                e.inner.msg_seq
            );
            Some(pbbot::frame::Data::GroupRecallNoticeEvent(
                to_proto_group_recall(bot, e).await,
            ))
        }
        QEvent::NewFriend(e) => {
            tracing::info!("NEW_FRIEND (FRIEND={}): {}", e.inner.uin, e.inner.nick);
            Some(pbbot::frame::Data::FriendAddNoticeEvent(
                to_proto_friend_add(bot, e).await,
            ))
        }
        QEvent::GroupLeave(e) => {
            tracing::info!(
                "GROUP_LEAVE (GROUP={}): {}",
                e.inner.group_code,
                e.inner.member_uin
            );
            Some(pbbot::frame::Data::GroupDecreaseNoticeEvent(
                to_proto_group_decrease(bot, e).await,
            ))
        }
        // QEvent::FriendPoke(_) => {}
        // QEvent::GroupNameUpdate(_) => {}
        // QEvent::DeleteFriend(_) => {}
        QEvent::MemberPermissionChange(e) => {
            tracing::info!(
                "PERMISSION_CHANGE (GROUP={}): {} {:?}",
                e.inner.group_code,
                e.inner.member_uin,
                e.inner.new_permission
            );
            Some(pbbot::frame::Data::GroupAdminNoticeEvent(
                to_proto_group_admin_notice(bot, e).await,
            ))
        }
        _ => None,
    }
}

pub async fn to_proto_group_message(
    bot: &Arc<Bot>,
    event: GroupMessageEvent,
) -> pbbot::GroupMessageEvent {
    let client = event.client;
    let message = event.inner;
    let role = bot
        .cached_group_role(message.group_code, message.from_uin)
        .await
        .unwrap_or_default();
    let message_id = MessageReceipt {
        sender_id: message.from_uin,
        time: message.time as i64,
        seqs: message.seqs,
        rands: message.rands,
        group_id: message.group_code,
    };
    let proto_message = to_proto_chain(&client, message.elements);
    let raw_message = proto_to_xml(proto_message.clone());
    pbbot::GroupMessageEvent {
        time: message.time as i64,
        self_id: client.uin().await,
        post_type: "message".to_string(),
        message_type: "group".to_string(),
        sub_type: "normal".to_string(),
        message_id: Some(message_id),
        group_id: message.group_code,
        user_id: message.from_uin,
        anonymous: None, // TODO
        raw_message,
        message: proto_message,
        sender: Some(pbbot::group_message_event::Sender {
            user_id: message.from_uin,
            card: message.group_card,
            role: match role {
                GroupMemberPermission::Owner => "owner",
                GroupMemberPermission::Administrator => "admin",
                GroupMemberPermission::Member => "member",
            }
            .into(),
            ..Default::default()
        }),
        font: 0,
        extra: Default::default(),
    }
}

pub async fn to_proto_private_message(
    _: &Arc<Bot>,
    event: FriendMessageEvent,
) -> pbbot::PrivateMessageEvent {
    let client = event.client;
    let message = event.inner;
    let message_id = MessageReceipt {
        sender_id: message.from_uin,
        time: message.time as i64,
        seqs: message.seqs,
        rands: message.rands,
        group_id: 0,
    };
    let proto_message = to_proto_chain(&client, message.elements);
    let raw_message = proto_to_xml(proto_message.clone());
    pbbot::PrivateMessageEvent {
        time: message.time as i64,
        self_id: client.uin().await,
        post_type: "message".to_string(),
        message_type: "private".to_string(),
        sub_type: "normal".to_string(),
        message_id: Some(message_id),
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

pub async fn to_proto_group_decrease(
    _: &Arc<Bot>,
    event: GroupLeaveEvent,
) -> pbbot::GroupDecreaseNoticeEvent {
    let client = event.client;
    let leave = event.inner;
    pbbot::GroupDecreaseNoticeEvent {
        time: chrono::Utc::now().timestamp(),
        self_id: client.uin().await,
        post_type: "notice".to_string(),
        notice_type: "group_decrease".to_string(),
        sub_type: if leave.operator_uin.is_some() {
            "kick"
        } else {
            "leave"
        }
        .to_string(),
        group_id: leave.group_code,
        operator_id: leave.operator_uin.unwrap_or_default(),
        user_id: leave.member_uin,
        extra: Default::default(),
    }
}

pub async fn to_proto_group_increase(
    _: &Arc<Bot>,
    event: NewMemberEvent,
) -> pbbot::GroupIncreaseNoticeEvent {
    let client = event.client;
    let new_mem = event.inner;
    pbbot::GroupIncreaseNoticeEvent {
        time: chrono::Utc::now().timestamp(),
        self_id: client.uin().await,
        post_type: "notice".to_string(),
        notice_type: "group_increase".to_string(),
        sub_type: "".into(),
        group_id: new_mem.group_code,
        operator_id: 0,
        user_id: new_mem.member_uin,
        extra: Default::default(),
    }
}
pub async fn to_proto_group_ban(_: &Arc<Bot>, event: GroupMuteEvent) -> pbbot::GroupBanNoticeEvent {
    let client = event.client;
    let mute = event.inner;
    pbbot::GroupBanNoticeEvent {
        time: chrono::Utc::now().timestamp(),
        self_id: client.uin().await,
        post_type: "notice".to_string(),
        notice_type: "group_ban".to_string(),
        sub_type: "".into(),
        group_id: mute.group_code,
        operator_id: mute.operator_uin,
        user_id: mute.target_uin,
        duration: mute.duration.as_secs() as i64,
        extra: Default::default(),
    }
}

pub async fn to_proto_friend_recall(
    _: &Arc<Bot>,
    event: FriendMessageRecallEvent,
) -> pbbot::FriendRecallNoticeEvent {
    let client = event.client;
    let recall = event.inner;
    let message_id = MessageReceipt {
        sender_id: recall.friend_uin,
        time: recall.time as i64,
        seqs: vec![recall.msg_seq],
        rands: vec![],
        group_id: 0,
    };
    pbbot::FriendRecallNoticeEvent {
        time: chrono::Utc::now().timestamp(),
        self_id: client.uin().await,
        post_type: "notice".to_string(),
        notice_type: "friend_recall".to_string(),
        user_id: recall.friend_uin,
        message_id: Some(message_id),
        extra: Default::default(),
    }
}

pub async fn to_proto_group_recall(
    _: &Arc<Bot>,
    event: GroupMessageRecallEvent,
) -> pbbot::GroupRecallNoticeEvent {
    let client = event.client;
    let recall = event.inner;
    let message_id = MessageReceipt {
        sender_id: recall.author_uin,
        time: recall.time as i64,
        seqs: vec![recall.msg_seq],
        rands: vec![],
        group_id: recall.group_code,
    };
    pbbot::GroupRecallNoticeEvent {
        time: chrono::Utc::now().timestamp(),
        self_id: client.uin().await,
        post_type: "notice".to_string(),
        notice_type: "group_recall".to_string(),
        group_id: recall.group_code,
        user_id: recall.author_uin,
        operator_id: recall.operator_uin,
        message_id: Some(message_id),
        extra: Default::default(),
    }
}

pub async fn to_proto_friend_add(
    _: &Arc<Bot>,
    event: NewFriendEvent,
) -> pbbot::FriendAddNoticeEvent {
    let client = event.client;
    pbbot::FriendAddNoticeEvent {
        time: chrono::Utc::now().timestamp(),
        self_id: client.uin().await,
        post_type: "notice".to_string(),
        notice_type: "friend_add".to_string(),
        user_id: event.inner.uin,
        extra: Default::default(),
    }
}

pub async fn to_proto_group_request(
    _: &Arc<Bot>,
    event: JoinGroupRequestEvent,
) -> pbbot::GroupRequestEvent {
    let client = event.client;
    let request = event.inner;
    let flag = format!(
        "{}:{}:{}",
        request.group_code, request.req_uin, request.msg_seq
    );
    let sub_type = format!(
        "{}{}",
        if request.invitor_uin.is_some() {
            "is_invite,"
        } else {
            ""
        },
        if request.suspicious {
            "suspicious,"
        } else {
            ""
        }
    );

    pbbot::GroupRequestEvent {
        time: chrono::Utc::now().timestamp(),
        self_id: client.uin().await,
        post_type: "request".to_string(),
        request_type: "group".to_string(),
        sub_type,
        group_id: request.group_code,
        user_id: request.req_uin,
        comment: request.message,
        flag,
        extra: Default::default(),
    }
}

pub async fn to_proto_self_group_request(
    _: &Arc<Bot>,
    event: SelfInvitedEvent,
) -> pbbot::GroupRequestEvent {
    let client = event.client;
    let request = event.inner;
    let flag = format!(
        "{}:{}:{}",
        request.group_code,
        client.uin().await,
        request.msg_seq
    );

    pbbot::GroupRequestEvent {
        time: chrono::Utc::now().timestamp(),
        self_id: client.uin().await,
        post_type: "request".to_string(),
        request_type: "group".to_string(),
        sub_type: "is_invite".into(),
        group_id: request.group_code,
        user_id: client.uin().await,
        comment: "".into(),
        flag,
        extra: HashMap::from([("invitor_uin".to_string(), request.invitor_uin.to_string())]),
    }
}

pub async fn to_proto_friend_request(
    _: &Arc<Bot>,
    event: NewFriendRequestEvent,
) -> pbbot::FriendRequestEvent {
    let client = event.client;
    let request = event.inner;
    let flag = format!("{}:{}", request.req_uin, request.msg_seq);

    pbbot::FriendRequestEvent {
        time: chrono::Utc::now().timestamp(),
        self_id: client.uin().await,
        post_type: "request".to_string(),
        request_type: "friend".to_string(),
        user_id: request.req_uin,
        comment: request.message,
        flag,
        extra: Default::default(),
    }
}

pub async fn to_proto_group_admin_notice(
    _: &Arc<Bot>,
    event: MemberPermissionChangeEvent,
) -> pbbot::GroupAdminNoticeEvent {
    let client = event.client;
    let change = event.inner;

    pbbot::GroupAdminNoticeEvent {
        time: chrono::Utc::now().timestamp(),
        self_id: client.uin().await,
        post_type: "notice".to_string(),
        notice_type: "group_admin".to_string(),
        sub_type: if matches!(change.new_permission, GroupMemberPermission::Administrator) {
            "set"
        } else {
            "unset"
        }
        .to_string(),
        group_id: change.group_code,
        user_id: change.member_uin,
        extra: Default::default(),
    }
}
