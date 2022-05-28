use crate::idl::pbbot;

pub fn proto_to_xml(s: Vec<pbbot::Message>) -> String {
    let elems: Vec<String> = s
        .into_iter()
        .map(|mut elem| {
            if elem.r#type == "text" {
                return elem.data.remove("text").unwrap_or_default();
            }
            let attrs = elem
                .data
                .into_iter()
                .map(|(k, v)| format!("{}=\"{}\"", k, xml::escape::escape_str_attribute(v)))
                .collect::<Vec<String>>()
                .join(" ");
            format!("<{} {}/>", elem.r#type, attrs)
        })
        .collect();
    elems.join("")
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::idl::pbbot;
    use crate::msg::to_xml::proto_to_xml;

    #[test]
    fn test_proto_to_xml() {
        let p = vec![
            pbbot::Message {
                r#type: "text".into(),
                data: HashMap::from([("text".to_string(), "xxx".to_string())]),
            },
            pbbot::Message {
                r#type: "at".into(),
                data: HashMap::from([("qq".to_string(), "123".to_string())]),
            },
        ];
        let x = proto_to_xml(p);
        println!("{}", x);
    }
}
