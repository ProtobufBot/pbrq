# pbbot-rq

基于 [ricq](https://github.com/lz1998/ricq) 的机器人框架，使用 websocket + protobuf 通信。


## 使用方法

1. 下载 [Release](https://github.com/ProtobufBot/pbbot-rq/releases) 版本。
2. 下载 [PBRQ-UI-Release](https://github.com/ProtobufBot/pbrq-react-ui/releases)，并解压 static.zip。
3. 如果是 Linux/MacOS 需要执行 `chmod +x pbbot-rq` 添加权限。
4. 执行 `./pbbot-rq --help` 查看帮助。
5. 执行 `./pbbot-rq --bind-addr 0.0.0.0:9000 --static-dir static` 启动程序，可以自己添加参数开启 跨域、HTTP-BASIC登录 等功能。
6. 打开浏览器访问 `http://localhost:9000` 管理机器人。
7. 首次运行后生成 `plugins` 文件夹，默认连接地址 `ws://localhost:8081/ws/rq/`，修改后重启生效。


```text
├── pbbot-rq.exe
└── static
    ├── asset-manifest.json
    ├── favicon.ico
    ├── index.html
    ├── logo192.png
    ├── logo512.png
    ├── manifest.json
    ├── robots.txt
    └── static
        ├── css
        │   └── main.a14a9148.css
        └── js
            ├── 27.af432e68.chunk.js
            ├── main.989aee2b.js
            └── main.989aee2b.js.LICENSE.txt
```




## API

- [x] SendPrivateMsg
- [x] SendGroupMsg
- [x] DeleteMsg
- [x] SetGroupKick
- [x] SetGroupBan
- [x] SetGroupWholeBan
- [x] SetGroupAdmin
- [x] SetGroupCard
- [x] SetGroupName
- [x] SetGroupLeave
- [x] SetGroupSpecialTitle
- [x] SetFriendAddRequest
- [x] SetGroupAddRequest
- [x] GetLoginInfo
- [x] GetStrangerInfo
- [x] GetFriendList
- [x] GetGroupInfo
- [x] GetGroupList
- [x] GetGroupMemberInfo
- [x] GetGroupMemberList

## Event

- [x] GroupMessageEvent
- [x] PrivateMessageEvent
- [x] GroupRequestEvent
- [x] GroupRequestEvent
- [x] FriendRequestEvent
- [x] GroupIncreaseNoticeEvent
- [x] GroupBanNoticeEvent
- [x] FriendRecallNoticeEvent
- [x] GroupRecallNoticeEvent
- [x] FriendAddNoticeEvent
- [x] GroupDecreaseNoticeEvent
- [x] GroupAdminNoticeEvent

## 消息类型

- [x] text
- [x] face
- [x] at
- [x] image
- [ ] video
- [ ] music

## 编译

环境要求：使用 [rustup](https://rustup.rs/) 安装的 Rust 环境。

如果速度较慢可以使用 [rsproxy](https://rsproxy.cn/)。

```bash
 # 更新rust工具链到最新
rustup update

# 拉取最新代码
git pull

# 更新依赖
cargo update

# 清理之前的产物
cargo clean

# 编译
cargo +nightly build --release

# 运行
./target/release/main
```
