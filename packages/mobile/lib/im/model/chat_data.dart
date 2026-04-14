

import '../message_handle.dart';
import 'package:wechat_flutter/im/tencent_mocks.dart';

class ChatDataRep {
  Future<List<V2TimMessage>> repData(String id, int type) async {
    return getDimMessages(id, type: type);
  }
}