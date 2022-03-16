use std::collections::HashMap;
use std::sync::Arc;

use rs_qq::msg::elem::RQElem;
use rs_qq::msg::{elem, MessageChain};
use rs_qq::Client;

use crate::idl::pbbot;

pub fn to_proto_chain(_client: &Arc<Client>, message: MessageChain) -> Vec<pbbot::Message> {
    let mut chain = Vec::new();
    for element in message.into_iter() {
        match element {
            RQElem::At(element) => append_at(&mut chain, element),
            RQElem::Text(element) => append_text(&mut chain, element),
            RQElem::Face(element) => append_face(&mut chain, element),
            // RQElem::FriendImage(element) => append_friend_image(&mut chain, element),
            // RQElem::GroupImage(element) => append_group_image(&mut chain, element),
            // RQElem::Other(element) => {
            // tracing::trace!("other elem {:?}", element)
            // }
            _ => {
                // tracing::warn!("elem not supported {:?}", element)
            }
        }
    }
    chain
}

pub fn append_text(chain: &mut Vec<pbbot::Message>, element: elem::Text) {
    chain.push(pbbot::Message {
        r#type: "text".into(),
        data: HashMap::from([("text".into(), element.content)]),
    })
}

pub fn append_at(chain: &mut Vec<pbbot::Message>, element: elem::At) {
    chain.push(pbbot::Message {
        r#type: "at".into(),
        data: HashMap::from([(
            "qq".into(),
            if element.target != 0 {
                element.target.to_string()
            } else {
                "all".into()
            },
        )]),
    })
}

pub fn append_face(chain: &mut Vec<pbbot::Message>, element: elem::Face) {
    chain.push(pbbot::Message {
        r#type: "face".into(),
        data: HashMap::from([("id".into(), element.index.to_string())]),
    })
}

// pub fn append_friend_image(chain: &mut Vec<pbbot::Message>, element: elem::FriendImage) {
//     chain.push(pbbot::Message {
//         r#type: "image".into(),
//         data: HashMap::from([("url".into(), element.url())]),
//     })
// }
//
// pub fn append_group_image(chain: &mut Vec<pbbot::Message>, element: elem::GroupImage) {
//     chain.push(pbbot::Message {
//         r#type: "image".into(),
//         data: HashMap::from([("url".into(), element.url())]),
//     })
// }
