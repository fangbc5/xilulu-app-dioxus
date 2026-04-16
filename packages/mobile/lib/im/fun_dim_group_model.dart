/// 群组管理 - 自研 SDK 实现
import 'dart:convert';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:flutter/material.dart';
import 'package:wechat_flutter/tools/wechat_flutter.dart';
import 'package:wechat_flutter/im/model/im_models.dart';
import 'package:wechat_flutter/src/rust/api/im.dart' as rust_im;
import 'package:wechat_flutter/im/conversation_handle.dart';

class DimGroup {
  static Future<dynamic> inviteGroupMember(List list, String groupId,
      {required Callback callback}) async {
    try {
      final gid = int.tryParse(groupId) ?? 0;
      final uids = list.map((id) => int.tryParse(id.toString()) ?? 0).where((id) => id > 0).toList();
      if (gid > 0 && uids.isNotEmpty) {
        await rust_im.coreInviteMembers(groupId: gid, memberUids: Int64List.fromList(uids));
      }
      callback(true);
    } catch (e) {
      debugPrint('邀请群成员失败: $e');
      callback(false);
    }
  }

  static Future<dynamic> quitGroupModel(String groupId,
      {required Callback callback}) async {
    try {
      final gid = int.tryParse(groupId) ?? 0;
      if (gid > 0) {
        await rust_im.coreQuitGroup(groupId: gid);
      }
      callback(true);
    } catch (e) {
      debugPrint('退出群聊失败: $e');
      callback(false);
    }
  }

  static Future<dynamic> deleteGroupMemberModel(String groupId, List deleteList,
      {required Callback callback}) async {
    try {
      final gid = int.tryParse(groupId) ?? 0;
      if (gid > 0) {
        for (final item in deleteList) {
          final uid = int.tryParse(item.toString()) ?? 0;
          if (uid > 0) {
            await rust_im.coreKickMember(groupId: gid, userId: uid);
          }
        }
      }
      callback(true);
    } catch (e) {
      debugPrint('删除群成员失败: $e');
      callback(false);
    }
  }

  static Future<List<XGroupMember?>> getGroupMembersListModelLIST(
      String groupId) async {
    try {
      final gid = int.tryParse(groupId) ?? 0;
      if (gid == 0) return [];
      final jsonStr = await rust_im.coreListGroupMembers(groupId: gid);
      final List<dynamic> memberJson = json.decode(jsonStr) as List<dynamic>;
      return memberJson.map((data) {
        return XGroupMember(
          userId: data['user_id']?.toString() ?? '',
          role: data['role'] as int? ?? 0,
          nickName: data['nick_name']?.toString(),
          faceUrl: data['face_url']?.toString(),
        );
      }).toList();
    } catch (e) {
      debugPrint('获取群成员列表失败: $e');
      return [];
    }
  }

  static Future<List<XGroupInfo>> getGroupListModel() async {
    try {
      // 临时使用会话列表过滤出群组会话
      final contacts = await getConversationsListData();
      return contacts
          .where((c) => c?.type == 2)
          .map((c) => XGroupInfo(
                groupId: c!.peerId,
                groupName: c.showName,
              ))
          .toList();
    } catch (e) {
      debugPrint('获取群列表失败: $e');
      return [];
    }
  }

  static Future<List<XGroupInfo>> getGroupInfoListModel(
      List<String> groupID) async {
    List<XGroupInfo> result = [];
    for (String id in groupID) {
      try {
        final gid = int.tryParse(id) ?? 0;
        if (gid > 0) {
          final jsonStr = await rust_im.coreGetGroupInfo(groupId: gid);
          final data = json.decode(jsonStr) as Map<String, dynamic>;
          result.add(XGroupInfo(
            groupId: data['id']?.toString() ?? id,
            groupName: data['name'] as String?,
            faceUrl: data['face_url'] as String?,
            introduction: data['introduction'] as String?,
            notification: data['notification'] as String?,
            owner: data['owner_uid']?.toString(),
            memberCount: data['member_count'] as int?,
          ));
        }
      } catch (e) {
        debugPrint('获取群资料失败: $e');
      }
    }
    return result;
  }

  static Future<dynamic> deleteGroupModel(String groupId,
      {required Callback callback}) async {
    try {
      final gid = int.tryParse(groupId) ?? 0;
      if (gid > 0) {
        await rust_im.coreDismissGroup(groupId: gid);
      }
      callback(true);
    } catch (e) {
      debugPrint('解散群失败: $e');
      callback(false);
    }
  }

  static Future<dynamic> modifyGroupNameModel(
      String groupId, String setGroupName,
      {required Callback callback}) async {
    try {
      final gid = int.tryParse(groupId) ?? 0;
      if (gid > 0) {
        await rust_im.coreUpdateGroupInfo(groupId: gid, name: setGroupName);
      }
      callback(true);
    } catch (e) {
      debugPrint('修改群名称失败: $e');
      callback(false);
    }
  }

  static Future<dynamic> modifyGroupIntroductionModel(
      String groupId, String setIntroduction,
      {required Callback callback}) async {
    try {
      final gid = int.tryParse(groupId) ?? 0;
      if (gid > 0) {
        await rust_im.coreUpdateGroupInfo(groupId: gid, introduction: setIntroduction);
      }
      callback(true);
    } catch (e) {
      debugPrint('修改群简介失败: $e');
      callback(false);
    }
  }

  static Future<dynamic> modifyGroupNotificationModel(
      String groupId, String notification, String time,
      {Callback? callback}) async {
    try {
      final gid = int.tryParse(groupId) ?? 0;
      if (gid > 0) {
        await rust_im.coreUpdateGroupInfo(groupId: gid, notification: notification);
      }
      if (callback != null) callback(true);
    } catch (e) {
      debugPrint('修改群公告失败: $e');
      if (callback != null) callback(false);
    }
  }

  static Future<dynamic> setReceiveMessageOptionModel(
      String groupId, String identifier, int type,
      {required Callback callback}) async {
    // 暂未实现此功能（免打扰）
    callback(true);
  }
}