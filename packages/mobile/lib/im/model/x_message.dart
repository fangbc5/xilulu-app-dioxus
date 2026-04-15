import 'dart:convert';

/// 原生的自研 IM 消息结构，用于替代 V2TimMessage
class XMessage {
  final String msgId;
  final int roomId;
  final int senderUid;
  final int msgType;
  final String content;
  final int localStatus;
  final int createdAt;

  // 为旧代码做兼容的别名方法
  String get id => msgId;
  int get timestamp => createdAt;
  String get sender => senderUid.toString();
  String get groupID => roomId.toString();
  int get elemType => msgType;

  // 快捷提取文本内容 (适配旧 textElem.text)
  String get textContent {
    if (msgType == 1) { // 假设 1 代表普通文本消息
      return content;
    }
    return '';
  }

  XMessage({
    required this.msgId,
    required this.roomId,
    required this.senderUid,
    required this.msgType,
    required this.content,
    required this.localStatus,
    required this.createdAt,
  });

  factory XMessage.fromJson(Map<String, dynamic> json) {
    return XMessage(
      msgId: json['msg_id']?.toString() ?? '',
      roomId: int.tryParse(json['room_id']?.toString() ?? '0') ?? 0,
      senderUid: int.tryParse(json['sender_uid']?.toString() ?? '0') ?? 0,
      msgType: int.tryParse(json['msg_type']?.toString() ?? '0') ?? 0,
      content: json['content']?.toString() ?? '',
      localStatus: int.tryParse(json['local_status']?.toString() ?? '0') ?? 0,
      createdAt: int.tryParse(json['created_at']?.toString() ?? '0') ?? 0,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'msg_id': msgId,
      'room_id': roomId,
      'sender_uid': senderUid,
      'msg_type': msgType,
      'content': content,
      'local_status': localStatus,
      'created_at': createdAt,
    };
  }
}
