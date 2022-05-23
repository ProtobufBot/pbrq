use std::collections::HashMap;
use std::sync::Arc;

use ricq::msg::{elem, MessageChain};
use ricq::Client;

use crate::error::{RCError, RCResult};
use crate::idl::pbbot;

#[derive(Clone, Debug)]
pub enum Contact {
    Group(i64),
    Friend(i64),
}

pub async fn to_rq_chain(
    client: &Arc<Client>,
    message: Vec<pbbot::Message>,
    contact: Contact,
) -> MessageChain {
    let mut chain = MessageChain::default();
    for element in message {
        match element.r#type.as_str() {
            "text" => append_text(&mut chain, element.data),
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
    chain.push(elem::At::new(
        data.remove("qq")
            .unwrap_or_default()
            .parse()
            .unwrap_or_default(),
    ))
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
    let data = reqwest::Client::new()
        .get(url)
        .send()
        .await
        .map_err(RCError::Reqwest)?
        .bytes()
        .await
        .map_err(RCError::Reqwest)?
        .to_vec();
    match contact {
        Contact::Group(code) => chain.push(client.upload_group_image(code, data).await?),
        Contact::Friend(uin) => chain.push(client.upload_friend_image(uin, data).await?),
    }
    Ok(())
}
