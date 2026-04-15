import '../message_handle.dart';
import 'package:wechat_flutter/im/model/x_message.dart';

class ChatDataRep {
  Future<List<XMessage>> repData(String id, int type) async {
    return getDimMessages(id, type: type);
  }
}