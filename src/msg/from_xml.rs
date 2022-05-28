use std::collections::HashMap;

use xml::reader::XmlEvent;

use crate::idl::pbbot;

pub fn xml_to_proto(s: String) -> Vec<pbbot::Message> {
    let msg = format!("<a>{}</a>", s);
    let reader = xml::reader::EventReader::from_str(&msg);
    let mut output = Vec::new();
    for elem in reader {
        if elem.is_err() {
            continue;
        }
        let elem = elem.unwrap();
        match elem {
            XmlEvent::StartElement {
                name, attributes, ..
            } => {
                if name.local_name == "a" {
                    continue;
                }
                output.push(pbbot::Message {
                    r#type: name.local_name,
                    data: attributes
                        .into_iter()
                        .map(|attr| (attr.name.local_name, attr.value))
                        .collect(),
                });
            }
            XmlEvent::Characters(s) => {
                output.push(pbbot::Message {
                    r#type: "text".into(),
                    data: HashMap::from([("text".to_string(), s)]),
                });
            }
            _ => continue,
        }
    }
    output
}

#[cfg(test)]
mod tests {
    extern crate xml;

    use crate::msg::from_xml::xml_to_proto;

    #[test]
    fn test_xml_to_proto() {
        let msg = "<a>as d  <face id=\"1\"/></a>".to_string();
        let p = xml_to_proto(msg);
        println!("{:?}", p)
    }
}
