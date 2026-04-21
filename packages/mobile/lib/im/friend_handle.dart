/// 好友管理 - 自研 SDK 实现
import 'dart:convert';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:wechat_flutter/tools/wechat_flutter.dart';
import 'package:wechat_flutter/im/model/im_models.dart';
import 'package:wechat_flutter/src/rust/api/im.dart' as rust_im;

typedef OnSuCc = void Function(bool v);

/// 添加好友
Future<dynamic> addFriend(String userName, BuildContext context,
    {String? message, OnSuCc? suCc}) async {
  try {
    // userName 可能是 user_id 字符串，需要转换为 u64
    final targetUid = int.tryParse(userName);
    if (targetUid == null) {
      showToast('用户 ID 格式错误');
      return;
    }

    await rust_im.coreAddFriend(targetUid: targetUid, message: message ?? '你好，我想加你为好友');
    showToast('好友申请已发送');
    if (suCc == null) {
      popToHomePage(context);
    } else {
      suCc(true);
    }
  } catch (e) {
    debugPrint('添加好友失败: $e');
    showToast('添加好友失败');
  }
}

/// 删除好友
Future<dynamic> delFriend(String userName, BuildContext context,
    {OnSuCc? suCc}) async {
  try {
    final targetUid = int.tryParse(userName);
    if (targetUid == null) {
      showToast('用户 ID 格式错误');
      return;
    }

    await rust_im.coreDeleteFriend(targetUid: targetUid);
    showToast('删除成功');
    if (suCc == null) {
      popToHomePage(context);
    } else {
      suCc(true);
    }
  } catch (e) {
    debugPrint('删除好友失败: $e');
    showToast('删除好友失败');
  }
}

/// 获取好友列表
Future<List<XFriendInfo>> getContactsFriends(String userName) async {
  try {
    final jsonStr = await rust_im.coreListFriends();
    final List<dynamic> friendsJson = json.decode(jsonStr) as List<dynamic>;

    return friendsJson.map((friendData) {
      final userId = friendData['friend_uid']?.toString() ?? '';
      final nickName = friendData['nick_name']?.toString();
      final faceUrl = friendData['avatar']?.toString();
      final friendRemark = friendData['remark']?.toString();

      return XFriendInfo(
        userId: userId,
        friendRemark: friendRemark,
        userProfile: XUserInfo(
          userId: userId,
          nickName: nickName,
          faceUrl: faceUrl,
        ),
      );
    }).toList();
  } catch (e) {
    debugPrint('获取好友列表失败: $e');
    return [];
  }
}

/// 搜索用户
Future<List<XUserInfo>> searchUser(String keyword) async {
  try {
    final jsonStr = await rust_im.coreSearchUser(keyword: keyword);
    final List<dynamic> usersJson = json.decode(jsonStr) as List<dynamic>;

    return usersJson.map((userData) {
      return XUserInfo(
        userId: userData['id']?.toString() ?? '',
        nickName: userData['nick_name']?.toString(),
        faceUrl: userData['avatar']?.toString(),
      );
    }).toList();
  } catch (e) {
    debugPrint('搜索用户失败: $e');
    return [];
  }
}

/// 获取好友申请列表
Future<List<XFriendApplication>> getFriendApplications() async {
  try {
    final jsonStr = await rust_im.coreListFriendApplies();
    final List<dynamic> appliesJson = json.decode(jsonStr) as List<dynamic>;

    return appliesJson.map((applyData) {
      return XFriendApplication(
        id: applyData['id'] as int?,
        userId: applyData['uid']?.toString(),
        nickName: applyData['nick_name']?.toString(),
        faceUrl: applyData['avatar']?.toString(),
        addWording: applyData['msg']?.toString(),
        type: applyData['status'] as int?,
      );
    }).toList();
  } catch (e) {
    debugPrint('获取好友申请列表失败: $e');
    return [];
  }
}

/// 同意好友申请
Future<bool> approveFriendApplication(int applyId) async {
  try {
    await rust_im.coreApproveFriendApply(applyId: applyId);
    return true;
  } catch (e) {
    debugPrint('同意好友申请失败: $e');
    return false;
  }
}

/// 拒绝好友申请
Future<bool> rejectFriendApplication(int applyId) async {
  try {
    await rust_im.coreRejectFriendApply(applyId: applyId);
    return true;
  } catch (e) {
    debugPrint('拒绝好友申请失败: $e');
    return false;
  }
}

/// 创建群聊
Future<bool> createGroupChat(List<String> personList, {String? name}) async {
  try {
    final memberUids = personList
        .map((id) => int.tryParse(id))
        .where((id) => id != null)
        .cast<int>()
        .toList();

    await rust_im.coreCreateGroup(
      name: name ?? '群聊',
      memberUids: Int64List.fromList(memberUids),
    );
    showToast('群组创建成功');
    return true;
  } catch (e) {
    debugPrint('创建群组失败: $e');
    showToast('创建群组失败');
    return false;
  }
}