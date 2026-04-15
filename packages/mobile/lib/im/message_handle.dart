import 'dart:convert';
import 'dart:developer';

import 'package:flutter/material.dart';
import 'package:wechat_flutter/im/model/x_message.dart';
import 'package:wechat_flutter/src/rust/api/im.dart' as rust_api;

/// 从本地 SQLite 获取指定房间的历史消息
Future<List<XMessage>> getDimMessages(String id,
    {required int type, Function? callback, int num = 50}) async {
  try {
    int roomId = int.tryParse(id) ?? 0;
    if (roomId == 0) return [];

    final String jsonStr = await rust_api.coreGetHistoryMessages(
      roomId: BigInt.from(roomId),
      limit: num,
    );

    final List<dynamic> jsonList = jsonDecode(jsonStr);
    final List<XMessage> messages =
        jsonList.map((e) => XMessage.fromJson(e as Map<String, dynamic>)).toList();

    return messages;
  } catch (e) {
    debugPrint('获取历史消息失败: $e');
    return [];
  }
}