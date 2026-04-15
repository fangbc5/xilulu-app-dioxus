import 'package:wechat_flutter/im/conversation_handle.dart';
import 'package:wechat_flutter/tools/wechat_flutter.dart';
import 'package:wechat_flutter/im/model/im_models.dart';

class ChatListData {
  Future<bool> isNull() async {
    final List<XConversation?> data = await getConversationsListData();
    return !listNoEmpty(data);
  }

  Future<List<XConversation?>> chatListData() async {
    final data = await getConversationsListData();
    return data.reversed.toList();
  }
}