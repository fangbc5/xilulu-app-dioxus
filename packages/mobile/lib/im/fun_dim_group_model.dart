/// 群组管理 - 自研 SDK 实现
/// TODO: 后续对接 Rust FFI 的群组相关接口
import 'package:wechat_flutter/tools/wechat_flutter.dart';
import 'package:wechat_flutter/im/model/im_models.dart';

class DimGroup {
  static Future<dynamic> inviteGroupMember(List list, String groupId,
      {required Callback callback}) async {
    // TODO: 对接 Rust SDK 邀请群成员
  }

  static Future<dynamic> quitGroupModel(String groupId,
      {required Callback callback}) async {
    // TODO: 对接 Rust SDK 退出群聊
  }

  static Future<dynamic> deleteGroupMemberModel(String groupId, List deleteList,
      {required Callback callback}) async {
    // TODO: 对接 Rust SDK 删除群成员
  }

  static Future<List<XGroupMember?>> getGroupMembersListModelLIST(
      String groupId) async {
    // TODO: 对接 Rust SDK 获取群成员列表
    return [];
  }

  static Future<List<XGroupInfo>> getGroupListModel() async {
    // TODO: 对接 Rust SDK 获取已加入群列表
    return [];
  }

  static Future<List<XGroupInfo>> getGroupInfoListModel(
      List<String> groupID) async {
    // TODO: 对接 Rust SDK 获取群资料
    return [];
  }

  static Future<dynamic> deleteGroupModel(String groupId,
      {required Callback callback}) async {
    // TODO: 对接 Rust SDK 解散群
  }

  static Future<dynamic> modifyGroupNameModel(
      String groupId, String setGroupName,
      {required Callback callback}) async {
    // TODO: 对接 Rust SDK 修改群名称
  }

  static Future<dynamic> modifyGroupIntroductionModel(
      String groupId, String setIntroduction,
      {required Callback callback}) async {
    // TODO: 对接 Rust SDK 修改群简介
  }

  static Future<dynamic> modifyGroupNotificationModel(
      String groupId, String notification, String time,
      {Callback? callback}) async {
    // TODO: 对接 Rust SDK 修改群公告
  }

  static Future<dynamic> setReceiveMessageOptionModel(
      String groupId, String identifier, int type,
      {required Callback callback}) async {
    // TODO: 对接 Rust SDK 修改群消息提醒选项
  }
}