import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:wechat_flutter/tools/wechat_flutter.dart';
import '../tools/event/im_event.dart';
import 'package:wechat_flutter/im/tencent_mocks.dart';
import 'package:wechat_flutter/src/rust/api/im.dart';

typedef CallbackMsg = void Function(V2TimMessage messageInfo);

Future<void> sendTextMsg(String targetId, int type, String context, {CallbackMsg? call}) async {
  try {
    int parsedRoomId = int.tryParse(targetId) ?? 0;
    // For now, hardcode sender_uid to 1 or pull from user config
    int senderUid = 1; 

    debugPrint('开始调用 Rust FFI: coreSendTextMessage($parsedRoomId, $context, $senderUid)');
    
    // Call our newly built Rust SDK!
    final rustResponseJson = await coreSendTextMessage(
        roomId: BigInt.from(parsedRoomId), 
        content: context, 
        senderUid: BigInt.from(senderUid)
    );
    
    debugPrint('Rust FFI 消息发送完成: $rustResponseJson');

    if (call != null) {
      // Mocking the return object so the remaining UI doesn't crash
      final fakeTencentMsg = V2TimMessage()
        ..msgID = "rust_msg_id"
        ..textElem = context;
      call(fakeTencentMsg);
    }

    /// Notice to refresh conversation list.
    eventBusNewMsg.value = EventBusNewMsg(targetId);
  } catch (e, s) {
    debugPrint('发送消息失败, $s');
    showToast('发送消息失败: $e');
  }
}