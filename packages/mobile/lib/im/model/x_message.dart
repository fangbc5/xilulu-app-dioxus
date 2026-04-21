/// 原生自研 IM 消息结构（字段名与服务端 message 实体 100% 对齐）
class XMessage {
  final String msgId;
  final int roomId;
  /// 对应服务端 from_uid（原 senderUid 已废弃，保留 getter 做兼容）
  final int fromUid;
  /// 消息内容（可为 null，例如撤回消息）
  final String? content;
  /// 对应服务端 type：1文本 2图片 3文件 4语音 5视频 6撤回 7系统
  final int type;
  /// 回复的消息 ID（整型）
  final int? replyMsgId;
  /// 对应服务端 status：0正常 1撤回
  final int status;
  /// JSON 扩展字段
  final String? extra;
  /// 仅客户端：本地发送状态。0已完成 1发送中 2失败
  final int localStatus;
  final int createdAt;

  // ── 旧代码兼容 Getter ────────────────────────────────
  String get id => msgId;
  int get timestamp => createdAt;
  /// 兼容旧 senderUid 引用
  int get senderUid => fromUid;
  String get sender => fromUid.toString();
  String get groupID => roomId.toString();
  int get elemType => type;

  /// 快捷提取文本内容（适配旧 textElem.text）
  String get textContent => type == 1 ? (content ?? '') : '';

  XMessage({
    required this.msgId,
    required this.roomId,
    required this.fromUid,
    this.content,
    required this.type,
    this.replyMsgId,
    this.status = 0,
    this.extra,
    required this.localStatus,
    required this.createdAt,
  });

  factory XMessage.fromJson(Map<String, dynamic> json) {
    return XMessage(
      msgId: json['msg_id']?.toString() ?? '',
      roomId: _parseInt(json['room_id']),
      // 优先取 from_uid，向下兼容旧 sender_uid
      fromUid: _parseInt(json['from_uid'] ?? json['sender_uid']),
      content: json['content']?.toString(),
      // 优先取 type，向下兼容旧 msg_type
      type: _parseInt(json['type'] ?? json['msg_type']),
      replyMsgId: json['reply_msg_id'] != null ? _parseInt(json['reply_msg_id']) : null,
      status: _parseInt(json['status']),
      extra: json['extra']?.toString(),
      localStatus: _parseInt(json['local_status']),
      createdAt: _parseInt(json['created_at']),
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'msg_id': msgId,
      'room_id': roomId,
      'from_uid': fromUid,
      'content': content,
      'type': type,
      'reply_msg_id': replyMsgId,
      'status': status,
      'extra': extra,
      'local_status': localStatus,
      'created_at': createdAt,
    };
  }

  static int _parseInt(dynamic v) {
    if (v == null) return 0;
    if (v is int) return v;
    return int.tryParse(v.toString()) ?? 0;
  }
}
