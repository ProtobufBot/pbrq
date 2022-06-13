# pbbot-rq

基于 [ricq](https://github.com/lz1998/ricq) 的机器人框架，使用 websocket + protobuf 通信。

建议直接下载 [Release](https://github.com/ProtobufBot/pbbot-rq/releases) 版本。

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