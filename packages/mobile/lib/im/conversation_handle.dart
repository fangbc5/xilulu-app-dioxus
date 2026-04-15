/// 会话管理 - 自研 SDK 实现
/// TODO: 后续对接 Rust FFI 的会话查询/删除等接口
import 'package:wechat_flutter/tools/wechat_flutter.dart';
import 'package:wechat_flutter/im/model/im_models.dart';

/// 获取会话列表
/// TODO: 对接 Rust core_get_conversations FFI
Future<List<XConversation?>> getConversationsListData() async {
  // 暂时返回一个硬编码的测试会话，后续对接 Rust SDK
  final fakeConv = XConversation(
    conversationId: "c2c_1",
    peerId: "1",
    showName: "测试机器人 (Bot)",
    type: 1,
    unreadCount: 0,
  );
  return [fakeConv];
}

/// 删除会话及本地消息
Future<dynamic> deleteConversationAndLocalMsgModel(String id, int type) async {
  // TODO: 对接 Rust SDK 删除会话
}

/// 删除本地消息
Future<dynamic> delLocalMsg(String identifier, int type) async {
  // TODO: 对接 Rust SDK 删除消息
  return true;
}

/// 删除会话
Future<dynamic> delConversationModel(String identifier, int type) async {
  // TODO: 对接 Rust SDK 删除会话
  return true;
}

/// 获取未读消息数量
Future<int> getUnreadMessageNumModel(int type, String id) async {
  // TODO: 对接 Rust SDK 查询未读数
  return 0;
}

/// 设置消息为已读
Future<void> setReadMessageModel(int type, String id) async {
  // TODO: 对接 Rust SDK 标记已读
}