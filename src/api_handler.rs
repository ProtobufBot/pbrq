use std::sync::Arc;
use std::time::Duration;

use ricq::structs::GroupMemberPermission;

use crate::bot::Bot;
use crate::error::{RCError, RCResult};
use crate::idl::pbbot::frame::Data;
use crate::idl::pbbot::*;
use crate::msg::{to_rq_chain, Contact};
use crate::plugin::pb_to_bytes::PbToBytes;

pub async fn handle_api_frame(bot: &Arc<Bot>, req_frame: Frame) -> Frame {
    let bot_id = req_frame.bot_id;
    let echo = req_frame.echo;
    let frame_type = req_frame.frame_type;
    let resp = if let Some(data) = req_frame.data {
        handle_api_data(bot, data).await
    } else {
        None
    };

    Frame {
        bot_id,
        frame_type: frame_type + 100,
        echo,
        ok: true,
        data: resp,
        extra: Default::default(),
    }
}

pub async fn handle_api_data(bot: &Arc<Bot>, data: Data) -> Option<Data> {
    match data {
        Data::SendPrivateMsgReq(req) => handle_send_private_msg(bot, req)
            .await
            .map(Data::SendPrivateMsgResp),
        Data::SendGroupMsgReq(req) => handle_send_group_msg(bot, req)
            .await
            .map(Data::SendGroupMsgResp),
        // Data::SendMsgReq(_) => {}
        Data::DeleteMsgReq(req) => handle_delete_msg(bot, req).await.map(Data::DeleteMsgResp),
        // Data::GetMsgReq(_) => {}
        // Data::GetForwardMsgReq(_) => {}
        Data::SendLikeReq(req) => handle_send_like(bot, req).await.map(Data::SendLikeResp),
        Data::SetGroupKickReq(req) => handle_group_kick(bot, req)
            .await
            .map(Data::SetGroupKickResp),
        Data::SetGroupBanReq(req) => handle_group_ban(bot, req).await.map(Data::SetGroupBanResp),
        // Data::SetGroupAnonymousBanReq(_) => {}
        Data::SetGroupWholeBanReq(req) => handle_group_whole_ban(bot, req)
            .await
            .map(Data::SetGroupWholeBanResp),
        Data::SetGroupAdminReq(req) => handle_set_group_admin(bot, req)
            .await
            .map(Data::SetGroupAdminResp),
        // Data::SetGroupAnonymousReq(_) => {}
        Data::SetGroupCardReq(req) => handle_set_group_card(bot, req)
            .await
            .map(Data::SetGroupCardResp),
        Data::SetGroupNameReq(req) => handle_set_group_name(bot, req)
            .await
            .map(Data::SetGroupNameResp),
        Data::SetGroupLeaveReq(req) => handle_group_leave(bot, req)
            .await
            .map(Data::SetGroupLeaveResp),
        Data::SetGroupSpecialTitleReq(req) => handle_set_group_special_title(bot, req)
            .await
            .map(Data::SetGroupSpecialTitleResp),
        // Data::SetFriendAddRequestReq(_) => {}
        // Data::SetGroupAddRequestReq(_) => {}
        Data::GetLoginInfoReq(req) => handle_get_login_info(bot, req)
            .await
            .map(Data::GetLoginInfoResp),
        Data::GetStrangerInfoReq(req) => handle_get_stranger_info(bot, req)
            .await
            .map(Data::GetStrangerInfoResp),
        Data::GetFriendListReq(req) => handle_get_friend_list(bot, req)
            .await
            .map(Data::GetFriendListResp),
        Data::GetGroupInfoReq(req) => handle_get_group_info(bot, req)
            .await
            .map(Data::GetGroupInfoResp),
        Data::GetGroupListReq(req) => handle_get_group_list(bot, req)
            .await
            .map(Data::GetGroupListResp),
        Data::GetGroupMemberInfoReq(req) => handle_get_group_member_info(bot, req)
            .await
            .map(Data::GetGroupMemberInfoResp),
        Data::GetGroupMemberListReq(req) => handle_get_group_member_list(bot, req)
            .await
            .map(Data::GetGroupMemberListResp),
        // Data::GetGroupHonorInfoReq(_) => {}
        // Data::GetCookiesReq(_) => {}
        // Data::GetCsrfTokenReq(_) => {}
        // Data::GetCredentialsReq(_) => {}
        // Data::GetRecordReq(_) => {}
        // Data::GetImageReq(_) => {}
        // Data::CanSendImageReq(_) => {}
        // Data::CanSendRecordReq(_) => {}
        // Data::GetStatusReq(_) => {}
        // Data::GetVersionInfoReq(_) => {}
        // Data::SetRestartReq(_) => {}
        // Data::CleanCacheReq(_) => {}
        _ => Err(RCError::None("api_req not supported")),
    }
    .ok()
}

pub async fn handle_send_private_msg(
    bot: &Arc<Bot>,
    req: SendPrivateMsgReq,
) -> RCResult<SendPrivateMsgResp> {
    let chain = to_rq_chain(
        &bot.client,
        req.message,
        Contact::Friend(req.user_id),
        req.auto_escape,
    )
    .await;
    let receipt = bot
        .client
        .send_friend_message(req.user_id, chain.clone())
        .await?;
    let message_id = MessageReceipt {
        sender_id: bot.client.uin().await,
        time: receipt.time,
        seqs: receipt.seqs,
        rands: receipt.rands,
        group_id: 0,
    }
    .to_bytes();
    Ok(SendPrivateMsgResp { message_id })
}

pub async fn handle_send_group_msg(
    bot: &Arc<Bot>,
    req: SendGroupMsgReq,
) -> RCResult<SendGroupMsgResp> {
    let chain = to_rq_chain(
        &bot.client,
        req.message,
        Contact::Group(req.group_id),
        req.auto_escape,
    )
    .await;
    let receipt = bot
        .client
        .send_group_message(req.group_id, chain.clone())
        .await?;
    let message_id = MessageReceipt {
        sender_id: bot.client.uin().await,
        time: receipt.time,
        seqs: receipt.seqs,
        rands: receipt.rands,
        group_id: req.group_id,
    }
    .to_bytes();
    Ok(SendGroupMsgResp { message_id })
}

pub async fn handle_delete_msg(bot: &Arc<Bot>, req: DeleteMsgReq) -> RCResult<DeleteMsgResp> {
    let receipt = MessageReceipt::from_bytes(&req.message_id)?;
    if receipt.group_id != 0 {
        bot.client
            .recall_group_message(receipt.group_id, receipt.seqs, receipt.rands)
            .await?;
    } else {
        bot.client
            .recall_friend_message(receipt.sender_id, receipt.time, receipt.seqs, receipt.rands)
            .await?;
    }
    Ok(DeleteMsgResp {})
}

pub async fn handle_send_like(bot: &Arc<Bot>, req: SendLikeReq) -> RCResult<SendLikeResp> {
    let summary = bot.client.get_summary_info(req.user_id).await?;
    bot.client
        .send_like(req.user_id, req.times, 1, summary.cookie)
        .await?;
    Ok(SendLikeResp {})
}

pub async fn handle_group_kick(bot: &Arc<Bot>, req: SetGroupKickReq) -> RCResult<SetGroupKickResp> {
    bot.client
        .group_kick(req.group_id, vec![req.user_id], "", req.reject_add_request)
        .await?;
    Ok(SetGroupKickResp {})
}

pub async fn handle_group_ban(bot: &Arc<Bot>, req: SetGroupBanReq) -> RCResult<SetGroupBanResp> {
    bot.client
        .group_mute(
            req.group_id,
            req.user_id,
            Duration::from_secs(req.duration as u64),
        )
        .await?;
    Ok(SetGroupBanResp {})
}

pub async fn handle_group_whole_ban(
    bot: &Arc<Bot>,
    req: SetGroupWholeBanReq,
) -> RCResult<SetGroupWholeBanResp> {
    bot.client.group_mute_all(req.group_id, req.enable).await?;
    Ok(SetGroupWholeBanResp {})
}

pub async fn handle_set_group_admin(
    bot: &Arc<Bot>,
    req: SetGroupAdminReq,
) -> RCResult<SetGroupAdminResp> {
    bot.client
        .group_set_admin(req.group_id, req.user_id, req.enable)
        .await?;
    Ok(SetGroupAdminResp {})
}

pub async fn handle_set_group_card(
    bot: &Arc<Bot>,
    req: SetGroupCardReq,
) -> RCResult<SetGroupCardResp> {
    bot.client
        .edit_group_member_card(req.group_id, req.user_id, req.card)
        .await?;
    Ok(SetGroupCardResp {})
}
pub async fn handle_set_group_name(
    bot: &Arc<Bot>,
    req: SetGroupNameReq,
) -> RCResult<SetGroupNameResp> {
    bot.client
        .update_group_name(req.group_id, req.group_name)
        .await?;
    Ok(SetGroupNameResp {})
}

pub async fn handle_group_leave(
    bot: &Arc<Bot>,
    req: SetGroupLeaveReq,
) -> RCResult<SetGroupLeaveResp> {
    bot.client.group_quit(req.group_id).await?;
    Ok(SetGroupLeaveResp {})
}

pub async fn handle_set_group_special_title(
    bot: &Arc<Bot>,
    req: SetGroupSpecialTitleReq,
) -> RCResult<SetGroupSpecialTitleResp> {
    // TODO duration 无效
    bot.client
        .group_edit_special_title(req.group_id, req.group_id, req.special_title)
        .await?;
    Ok(SetGroupSpecialTitleResp {})
}

pub async fn handle_get_login_info(
    bot: &Arc<Bot>,
    _: GetLoginInfoReq,
) -> RCResult<GetLoginInfoResp> {
    Ok(GetLoginInfoResp {
        user_id: bot.client.uin().await,
        nickname: bot.client.account_info.read().await.nickname.clone(),
    })
}

pub async fn handle_get_stranger_info(
    bot: &Arc<Bot>,
    req: GetStrangerInfoReq,
) -> RCResult<GetStrangerInfoResp> {
    let info = bot.client.get_summary_info(req.user_id).await?;
    Ok(GetStrangerInfoResp {
        user_id: info.uin,
        nickname: info.nickname,
        sex: info.sex.to_string(), // TODO ?
        age: info.age as i32,
        level: info.level,
        login_days: info.login_days,
    })
}

pub async fn handle_get_friend_list(
    bot: &Arc<Bot>,
    _: GetFriendListReq,
) -> RCResult<GetFriendListResp> {
    Ok(GetFriendListResp {
        friend: bot
            .client
            .get_friend_list()
            .await?
            .friends
            .into_iter()
            .map(|f| get_friend_list_resp::Friend {
                user_id: f.uin,
                nickname: f.nick,
                remark: f.remark,
            })
            .collect(),
    })
}

pub async fn handle_get_group_info(
    bot: &Arc<Bot>,
    req: GetGroupInfoReq,
) -> RCResult<GetGroupInfoResp> {
    let group = bot
        .client
        .get_group_info(req.group_id)
        .await?
        .ok_or_else(|| RCError::None("group"))?;
    Ok(GetGroupInfoResp {
        group_id: group.code,
        group_name: group.name,
        member_count: group.member_count as i32,
        max_member_count: group.max_member_count as i32,
    })
}

pub async fn handle_get_group_list(
    bot: &Arc<Bot>,
    _: GetGroupListReq,
) -> RCResult<GetGroupListResp> {
    Ok(GetGroupListResp {
        group: bot
            .client
            .get_group_list()
            .await?
            .into_iter()
            .map(|g| get_group_list_resp::Group {
                group_id: g.code,
                group_name: g.name,
                member_count: g.member_count as i32,
                max_member_count: g.max_member_count as i32,
            })
            .collect(),
    })
}

pub async fn handle_get_group_member_info(
    bot: &Arc<Bot>,
    req: GetGroupMemberInfoReq,
) -> RCResult<GetGroupMemberInfoResp> {
    let member = bot
        .client
        .get_group_member_info(req.group_id, req.user_id)
        .await?;
    Ok(GetGroupMemberInfoResp {
        group_id: member.group_code,
        user_id: member.uin,
        nickname: member.nickname,
        card: member.card_name,
        sex: "".into(), // TODO
        age: 0,
        area: "".into(),
        join_time: member.join_time,
        last_sent_time: member.last_speak_time,
        level: member.level.to_string(),
        role: match member.permission {
            GroupMemberPermission::Owner => "owner",
            GroupMemberPermission::Administrator => "admin",
            GroupMemberPermission::Member => "member",
        }
        .to_string(),
        unfriendly: false,
        title: member.special_title,
        title_expire_time: member.special_title_expire_time,
        card_changeable: false,
    })
}

pub async fn handle_get_group_member_list(
    bot: &Arc<Bot>,
    req: GetGroupMemberListReq,
) -> RCResult<GetGroupMemberListResp> {
    let group = bot
        .client
        .get_group_info(req.group_id)
        .await?
        .ok_or_else(|| RCError::None("group"))?;
    let members = bot
        .client
        .get_group_member_list(req.group_id, group.owner_uin)
        .await?;
    Ok(GetGroupMemberListResp {
        group_member: members
            .into_iter()
            .map(|member| get_group_member_list_resp::GroupMember {
                group_id: member.group_code,
                user_id: member.uin,
                nickname: member.nickname,
                card: member.card_name,
                sex: "".into(), // TODO
                age: 0,
                area: "".into(),
                join_time: member.join_time,
                last_sent_time: member.last_speak_time,
                level: member.level.to_string(),
                role: match member.permission {
                    GroupMemberPermission::Owner => "owner",
                    GroupMemberPermission::Administrator => "admin",
                    GroupMemberPermission::Member => "member",
                }
                .to_string(),
                unfriendly: false,
                title: member.special_title,
                title_expire_time: member.special_title_expire_time,
                card_changeable: false,
            })
            .collect(),
    })
}
