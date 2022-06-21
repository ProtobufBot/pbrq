use std::collections::HashMap;
use std::sync::Arc;

use async_recursion::async_recursion;
use ricq::msg::{elem, MessageChain};
use ricq::Client;

use crate::error::RCResult;
use crate::idl::pbbot;
use crate::msg::from_xml::xml_to_proto;
use crate::util::uri_reader::get_binary;

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
            display = format!("@{}", d);
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
