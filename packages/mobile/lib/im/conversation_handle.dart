/// 会话管理 - 自研 SDK 实现
import 'dart:convert';
import 'package:wechat_flutter/tools/wechat_flutter.dart';
import 'package:wechat_flutter/im/model/im_models.dart';
import 'package:wechat_flutter/im/model/x_message.dart';
import 'package:wechat_flutter/src/rust/api/im.dart' as rust_im;

/// 获取会话列表
Future<List<XConversation?>> getConversationsListData() async {
  try {
    final jsonStr = await rust_im.coreGetContacts();
    final List<dynamic> contactsJson = json.decode(jsonStr) as List<dynamic>;

    return contactsJson.map((contactData) {
      // Rust 返回的是 XContact 结构
      final roomId = contactData['room_id'] as int;
      final roomType = (contactData['room_type'] as int?) ?? 1;
      final unreadCount = (contactData['unread_count'] as int?) ?? 0;

      XMessage? lastMsg;
      if (contactData['last_message'] != null) {
        lastMsg = XMessage.fromJson(contactData['last_message'] as Map<String, dynamic>);
      }

      final showName = (contactData['show_name'] as String?) ?? (roomType == 1 ? '用户 $roomId' : '群聊 $roomId');
      final faceUrl = (contactData['face_url'] as String?) ?? '';

      return XConversation(
        conversationId: 'room_$roomId',
        type: roomType,
        peerId: roomId.toString(),
        showName: showName,
        faceUrl: faceUrl,
        unreadCount: unreadCount,
        lastMessage: lastMsg,
      );
    }).toList();
  } catch (e) {
    print('获取会话列表失败: $e');
    return [];
  }
}

/// 删除会话及本地消息
Future<dynamic> deleteConversationAndLocalMsgModel(String id, int type) async {
  try {
    // id 格式为 "room_123" 或直接是 room_id
    final roomId = id.startsWith('room_')
        ? int.parse(id.substring(5))
        : int.parse(id);

    await rust_im.coreDeleteContact(roomId: roomId);
    return true;
  } catch (e) {
    print('删除会话失败: $e');
    return false;
  }
}

/// 删除本地消息
Future<dynamic> delLocalMsg(String identifier, int type) async {
  // TODO: 后续实现删除本地消息功能
  return true;
}

/// 删除会话
Future<dynamic> delConversationModel(String identifier, int type) async {
  return deleteConversationAndLocalMsgModel(identifier, type);
}

/// 获取未读消息数量
Future<int> getUnreadMessageNumModel(int type, String id) async {
  // TODO: 从本地 DB 查询未读数
  return 0;
}

/// 设置消息为已读
Future<void> setReadMessageModel(int type, String id) async {
  try {
    final roomId = id.startsWith('room_')
        ? int.parse(id.substring(5))
        : int.parse(id);

    await rust_im.coreMarkRead(roomId: roomId);
  } catch (e) {
    print('标记已读失败: $e');
  }
}