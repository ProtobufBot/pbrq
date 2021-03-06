use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use async_recursion::async_recursion;
use ricq::msg::{elem, MessageChain};
use ricq::Client;
use ricq_core::hex::encode_hex;

use crate::error::RCResult;
use crate::idl::pbbot;
use crate::msg::from_xml::xml_to_proto;
use crate::util::uri_reader::{get_binary, read_binary_file};

#[derive(Clone, Debug)]
pub enum Contact {
    Group(i64),
    Friend(i64),
}

#[async_recursion]
pub async fn to_rq_chain(
    client: &Arc<Client>,
    message: Vec<pbbot::Message>,
    contact: Contact,
    auto_escape: bool,
) -> MessageChain {
    let mut chain = MessageChain::default();
    for mut element in message {
        match element.r#type.as_str() {
            "text" => {
                if auto_escape {
                    append_text(&mut chain, element.data)
                } else {
                    let text = element.data.remove("text").unwrap_or_default();
                    if text.is_empty() {
                        continue;
                    }
                    let ccc = to_rq_chain(client, xml_to_proto(text), contact.clone(), true).await;
                    chain.0.extend(ccc.0);
                }
            }
            "at" => append_at(&mut chain, element.data),
            "face" => append_face(&mut chain, element.data),
            "image" => {
                if let Err(e) =
                    append_image(client, &mut chain, element.data, contact.clone()).await
                {
                    tracing::error!("failed to append image: {}", e)
                }
            }
            "video" => {
                if let Err(e) =
                    append_video(client, &mut chain, element.data, contact.clone()).await
                {
                    tracing::error!("failed to append video: {}", e)
                }
            }
            _ => {
                println!("{} not supported", element.r#type)
            }
        }
    }
    chain
}

pub fn append_text(chain: &mut MessageChain, mut data: HashMap<String, String>) {
    chain.push(elem::Text::new(data.remove("text").unwrap_or_default()))
}

pub fn append_at(chain: &mut MessageChain, mut data: HashMap<String, String>) {
    let target = data
        .remove("qq")
        .unwrap_or_default()
        .parse()
        .unwrap_or_default();
    let mut display = format!("@{}", target);
    if let Some(d) = data.remove("display") {
        if !d.is_empty() {
            display = d;
        }
    }
    chain.push(elem::At { target, display })
}

pub fn append_face(chain: &mut MessageChain, mut data: HashMap<String, String>) {
    chain.push(elem::Face::new(
        data.remove("id")
            .unwrap_or_default()
            .parse()
            .unwrap_or_default(),
    ))
}

pub async fn append_image(
    client: &Arc<Client>,
    chain: &mut MessageChain,
    mut data: HashMap<String, String>,
    contact: Contact,
) -> RCResult<()> {
    let url = data.remove("url").unwrap_or_default();
    let data = get_binary(&url).await?;
    match contact {
        Contact::Group(code) => chain.push(client.upload_group_image(code, data).await?),
        Contact::Friend(uin) => chain.push(client.upload_friend_image(uin, data).await?),
    }
    Ok(())
}

pub async fn append_video(
    client: &Arc<Client>,
    chain: &mut MessageChain,
    mut data: HashMap<String, String>,
    contact: Contact,
) -> RCResult<()> {
    let cover_url = data.remove("cover").unwrap_or_default();
    let cover_data = get_binary(&cover_url).await?;
    let video_url = data.remove("url").unwrap_or_default();
    let video_cache_path = format!(
        "video/{}.mp4",
        encode_hex(&md5::compute(&video_url).to_vec())
    );
    let use_cache = Some("1".to_string()) == data.remove("cache");
    if use_cache && Path::new(&video_cache_path).exists() {
        if let Ok(video_data) = read_binary_file(&video_cache_path).await {
            chain.push(
                client
                    .upload_group_short_video(
                        match contact {
                            Contact::Group(code) => code,
                            Contact::Friend(uin) => uin,
                        },
                        video_data,
                        cover_data,
                    )
                    .await?,
            );
            return Ok(());
        }
    }
    let video_data = get_binary(&video_url).await?;
    if use_cache {
        if !Path::new("video").exists() {
            tokio::fs::create_dir_all("video").await.ok();
        }
        if let Err(err) = tokio::fs::write(video_cache_path, &video_data).await {
            tracing::error!("failed to write video cache {}", err)
        }
    }
    chain.push(
        client
            .upload_group_short_video(
                match contact {
                    Contact::Group(code) => code,
                    Contact::Friend(uin) => uin,
                },
                video_data,
                cover_data,
            )
            .await?,
    );
    Ok(())
}
