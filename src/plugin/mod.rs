use serde::{Deserialize, Serialize};

pub mod conn;
pub mod pb_to_bytes;
pub mod storage;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Plugin {
    #[serde(skip)]
    pub name: String,
    pub disabled: bool,
    pub urls: Vec<String>,
    // TODO
    // 	EventFilter  []int32             `json:"event_filter"`  // 事件过滤
    // 	ApiFilter    []int32             `json:"api_filter"`    // API过滤
    // 	RegexFilter  string              `json:"regex_filter"`  // 正则过滤
    // 	RegexReplace string              `json:"regex_replace"` // 正则替换
    // 	ExtraHeader  map[string][]string `json:"extra_header"`  // 自定义请求头
}

impl Default for Plugin {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            disabled: false,
            urls: vec!["http://localhost:8081/ws/rq/".into()],
        }
    }
}
