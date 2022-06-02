use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use cached::Cached;

use crate::bot;
use crate::bot::Bot;
use crate::error::{RCError, RCResult};
use crate::idl::pbbot::frame::Data;
use crate::idl::pbbot::*;
use crate::msg::{to_rq_chain, Contact};

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
        // Data::SetGroupAdminReq(_) => {}
        // Data::SetGroupAnonymousReq(_) => {}
        // Data::SetGroupCardReq(_) => {}
        // Data::SetGroupNameReq(_) => {}
        // Data::SetGroupLeaveReq(_) => {}
        // Data::SetGroupSpecialTitleReq(_) => {}
        // Data::SetFriendAddRequestReq(_) => {}
        // Data::SetGroupAddRequestReq(_) => {}
        // Data::GetLoginInfoReq(_) => {}
        // Data::GetStrangerInfoReq(_) => {}
        // Data::GetFriendListReq(_) => {}
        // Data::GetGroupInfoReq(_) => {}
        // Data::GetGroupListReq(_) => {}
        // Data::GetGroupMemberInfoReq(_) => {}
        // Data::GetGroupMemberListReq(_) => {}
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
    let message_id = bot.message_id.fetch_add(1, Ordering::Relaxed);
    bot.message_cache.write().await.cache_set(
        message_id,
        bot::Message {
            time: receipt.time as i32,
            from_uin: bot.client.uin().await,
            from_group: None,
            elements: chain,
            seqs: receipt.seqs,
            rans: receipt.rands,
        },
    );

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
    let message_id = bot.message_id.fetch_add(1, Ordering::Relaxed);
    bot.message_cache.write().await.cache_set(
        message_id,
        bot::Message {
            time: receipt.time as i32,
            from_uin: bot.client.uin().await,
            from_group: Some(req.group_id),
            elements: chain,
            seqs: receipt.seqs,
            rans: receipt.rands,
        },
    );
    Ok(SendGroupMsgResp { message_id })
}

pub async fn handle_delete_msg(bot: &Arc<Bot>, req: DeleteMsgReq) -> RCResult<DeleteMsgResp> {
    let message = bot
        .message_cache
        .write()
        .await
        .cache_get(&req.message_id)
        .cloned()
        .ok_or(RCError::None("message_id"))?;
    if let Some(group_code) = message.from_group {
        bot.client
            .recall_group_message(group_code, message.seqs, message.rans)
            .await?;
    } else {
        bot.client
            .recall_friend_message(
                message.from_uin,
                message.time as i64,
                message.seqs,
                message.rans,
            )
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
