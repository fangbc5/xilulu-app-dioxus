# Xilulu Mobile 迁移：wechat_flutter 前端集成实施文档

## 概述
当前的移动端需求转向 Flutter 方案后，针对现有纯净并高度解耦在 `packages/core` 的 Rust 业务核心代码，经过调研，决定将开源的高完成度 IM UI 库 [`wechat_flutter`](https://github.com/fluttercandies/wechat_flutter) 作为移动端的底层脚手架，以避免重复造轮子快速完成 UI 迁移。

本文档详细介绍了如何将 wechat_flutter 的丰富界面无痕“嫁接”到 Xilulu 基于 Rust 的底层之上。

## 一、可复用的 UI 资产盘点
wechat_flutter 已经非常完整地在纯 Dart/Flutter 侧实现了大量我们需要的视觉和交互层：
1. **主会话列表 (Message List)**：仿微信界面的首页消息流。
2. **通讯录页面 (Contacts)**：包括索引提取、群聊管理菜单等。
3. **聊天详情页 (Chat Detail)**：实现了丰富的高级组件：
   - 包含表情键盘。
   - 语音按住录音、声波播放交互。
   - 发送图片、多媒体库选取交互。
4. **账号体系及其他页面**：登录界面、扫码功能、个人信息设置页面等。

## 二、架构融合：“换心手术”实施策略
**核心理念**：将并保留 wechat_flutter 中所有的 UI 组件层(Widgets)，但抛弃其内置的状态、网络获取和模拟数据管理部分。Xilulu 的核心业务 (`packages/core`) 将通过 `flutter_rust_bridge` 来成为应用真正的大脑。

### 1. 代码初始化与依赖洗牌
* **引入方式**：清空目前 `packages/mobile` 内所有 Dioxus 的包裹代码，在此目录建立新的 Flutter 环境，并将 wechat_flutter 源码的 `lib/`、`assets/`、及关键依赖合入。
* **核心剥离**：全局搜索并清理原项目中用于返回假数据的 mock 函数，以及与其耦合不够清晰的 provider 状态（视代码质量决定保留部分本地状态管理或引入 Riverpod）。

### 2. 状态映射器 (Rust 数据 -> Dart UI Model)
从 Rust `core` 暴露出来的模型（如 UserData, ChatSession, PushMessage）往往与 UI 层的所需模型可能因历史原因有字段差异。
建立一套 `Adapter`（适配层）：
* 将 Rust FFI 生成的类映射到 UI 页面 `State` 期望绑定的对象。
* **举例**：`packages/core::model::User`（FFI层）在收到后被包装转换成 Dart 侧需要的 `UserInfoModel` 喂给 UI 渲染。

### 3. 接管核心管线 (API & WebSocket)
使用 `wechat_flutter` 当作展示层，其触发真正的动作时都要路由进 Rust 桥：

**(1) 认证生命周期（Auth）**：
* 登录点击时，不是走原有的请求，而是调用 Dart 中的 `XiluluCore.doLogin(req)`，这个方法 await C FFI 暴露的 tokio 异步方法。
* Rust 返回成功后，将 Token 和状态交给 Flutter 侧的本地存储/状态库记录，跳转。

**(2) WebSocket 消息推送与刷新（Chat）**：
* wechat_flutter 聊天页通过监听不断吐出的 Stream 进行聊天气泡的上推刷新。
* 在我们要整合的架构中，`XiluluCore.subscribeChatSink()` 将绑定我们在 Rust 端的 `core::ws` 的消息接收通道。所有后端推送的新消息，将在 Rust 侧解析完后直接压入 FFI 生成的 Dart Stream 中，wechat_flutter 的页面层只需要用 `StreamBuilder` 或 Provider 监听并追加给 `ListView` 即可做到真正的响应式开发。

**(3) 多媒体文件操作（Media & Storage）**：
* wechat_flutter 提供的相册拉取、摄像头按压录制，最终输出为本地文件的绝对路径。
* 依然调用 Rust 的上传服务：将文件路径或基于其转换为 `Uint8List` 的内容经由 FFI 传递给 Rust 相应的网络 port 处理（如 AWS S3，阿里 OSS），Rust 层完成后给 Dart 层反馈。

## 三、实施路线图与阶段检验点

* **Phase A: 骨架跑通**
  完成 FFI 构建和 wechat_flutter 在 Xilulu workspace 中的可执行。确保无报错编译。
* **Phase B: 用户链路对齐**
  登录、注册完全切换为 Xilulu 原有基于 Rust `core::api` 的后端协议，能够成功登陆拿到 `app_token`。
* **Phase C: 互动与IM界面接管**
  对接会话列表的加载逻辑。建立 Rust WebSocket长连，打通 Flutter 侧发送消息、收取消息气泡渲染。
* **Phase D: 功能补齐与打磨**
  梳理 wechat_flutter 中多余的页面（如支付展示页）并隐藏；实现群信息变更、通讯录等剩余功能；确保国际化兼容。

---
最后更新于：实施计划第一阶段设计。
