/// 好友管理 - 自研 SDK 实现
/// TODO: 后续对接 Rust FFI 的好友相关接口
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:wechat_flutter/tools/wechat_flutter.dart';
import 'package:wechat_flutter/im/model/im_models.dart';

typedef OnSuCc = void Function(bool v);

/// 添加好友
/// TODO: 对接 Rust SDK 添加好友
Future<dynamic> addFriend(String userName, BuildContext context,
    {OnSuCc? suCc}) async {
  try {
    // TODO: 调用 Rust FFI 添加好友
    showToast('添加成功');
    if (suCc == null) {
      popToHomePage(context);
    } else {
      suCc(true);
    }
  } on PlatformException {
    debugPrint('添加好友失败');
  }
}

/// 删除好友
/// TODO: 对接 Rust SDK 删除好友
Future<dynamic> delFriend(String userName, BuildContext context,
    {OnSuCc? suCc}) async {
  try {
    // TODO: 调用 Rust FFI 删除好友
    showToast('删除成功');
    if (suCc == null) {
      popToHomePage(context);
    } else {
      suCc(true);
    }
  } on PlatformException {
    debugPrint('删除好友失败');
  }
}

/// 获取好友列表
/// TODO: 对接 Rust SDK 获取好友列表
Future<List<XFriendInfo>> getContactsFriends(String userName) async {
  // 暂时返回一个硬编码的好友数据，后续对接 Rust SDK
  final fakeFriend = XFriendInfo(
    userId: "1",
    friendRemark: "测试机器人 (Bot)",
    userProfile: XUserInfo(userId: "1", nickName: "机器人"),
    friendAddSource: "system",
  );
  return [fakeFriend];
}

/// 创建群聊
/// TODO: 对接 Rust SDK 创建群聊
Future<bool> createGroupChat(List<String> personList, {String? name}) async {
  // TODO: 调用 Rust FFI 创建群聊
  return true;
}