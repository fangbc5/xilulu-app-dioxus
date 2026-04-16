import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:wechat_flutter/tools/wechat_flutter.dart';
import '../tools/event/im_event.dart';
import 'package:wechat_flutter/im/model/x_message.dart';
import 'package:wechat_flutter/src/rust/api/im.dart';

typedef CallbackMsg = void Function(XMessage messageInfo);

Future<void> sendTextMsg(String targetId, int type, String context, {CallbackMsg? call}) async {
  try {
    int parsedRoomId = int.tryParse(targetId) ?? 0;
    // TODO: 从用户配置中获取当前登录用户 UID
    int senderUid = 1;

    debugPrint('开始调用 Rust FFI: coreSendTextMessage($parsedRoomId, $context, $senderUid)');

    // 调用自研 Rust SDK 发送消息
    final rustResponseJson = await coreSendTextMessage(
        roomId: parsedRoomId,
        content: context,
        senderUid: senderUid);

    debugPrint('Rust FFI 消息发送完成: $rustResponseJson');

    if (call != null) {
      final Map<String, dynamic> json = jsonDecode(rustResponseJson) as Map<String, dynamic>;
      final xmsg = XMessage.fromJson(json);
      call(xmsg);
    }

    /// 通知刷新会话列表
    eventBusNewMsg.value = EventBusNewMsg(targetId);
  } catch (e, s) {
    debugPrint('发送消息失败, $s');
    showToast('发送消息失败: $e');
  }
}

Future<void> sendImageMsg(String targetId, int type, {required dynamic file, CallbackMsg? callback, dynamic source}) async {
  debugPrint('Mock sendImageMsg called');
  showToast('图片发送尚未在 Rust 层实现');
}