/// 用户信息管理 - 自研 SDK 实现
import 'dart:convert';
import 'dart:developer';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'package:wechat_flutter/provider/global_model.dart';
import 'package:wechat_flutter/tools/wechat_flutter.dart';
import 'package:wechat_flutter/im/model/im_models.dart';
import 'package:wechat_flutter/src/rust/api/im.dart' as rust_im;

/// 获取备注
Future<String?> getRemarkMethod(String id) async {
  // TODO: 从 Rust SDK 获取用户备注
  return "";
}

/// 设置用户资料
Future<bool> setUsersProfileMethod(
  BuildContext context, {
  String nickNameStr = '',
  String avatarStr = '',
}) async {
  try {
    final model = Provider.of<GlobalModel>(context, listen: false);

    // 获取当前用户 ID
    final accountStr = model.account ?? '0'; // GlobalModel uses account, not userId
    final userId = int.tryParse(accountStr);
    if (userId == null || userId == 0) {
      debugPrint('用户 ID 无效');
      return false;
    }

    // 调用 Rust FFI 更新用户资料到服务端
    final jsonStr = await rust_im.coreUpdateUserInfo(
      userId: userId,
      nickName: nickNameStr.isNotEmpty ? nickNameStr : null,
      faceUrl: avatarStr.isNotEmpty ? avatarStr : null,
      selfSignature: null,
      gender: null,
    );

    // 解析返回的用户信息并更新本地状态
    final userInfo = json.decode(jsonStr);
    if (nickNameStr.isNotEmpty) {
      model.nickName = (userInfo['nick_name'] as String?) ?? nickNameStr;
    }
    if (avatarStr.isNotEmpty) {
      model.avatar = (userInfo['face_url'] as String?) ?? avatarStr;
    }

    return true;
  } catch (e) {
    debugPrint('更新用户资料失败: $e');
    return false;
  }
}

/// 获取单个用户资料
Future<XUserInfo?> getUserProfile(String userId) async {
  try {
    final uid = int.tryParse(userId);
    if (uid == null) {
      return null;
    }

    final jsonStr = await rust_im.coreGetUserInfo(userId: uid);
    final Map<String, dynamic> userData = json.decode(jsonStr) as Map<String, dynamic>;

    return XUserInfo(
      userId: userData['id']?.toString() ?? userId,
      nickName: userData['nick_name']?.toString(),
      faceUrl: userData['face_url']?.toString(),
      selfSignature: userData['self_signature']?.toString(),
      gender: userData['gender'] as int?,
    );
  } catch (e) {
    debugPrint('获取用户资料失败: $e');
    return null;
  }
}

/// 获取用户资料列表
Future<List<XUserInfo>> getUsersProfile(List<String> users) async {
  try {
    // 将字符串 ID 转换为 u64
    final userIds = Int64List.fromList(users
        .map((id) => int.tryParse(id) ?? 0)
        .where((id) => id > 0)
        .toList());

    if (userIds.isEmpty) {
      return [];
    }

    final jsonStr = await rust_im.coreGetUsersInfo(userIds: userIds);
    final List<dynamic> usersJson = json.decode(jsonStr) as List<dynamic>;

    return usersJson.map((userData) {
      return XUserInfo(
        userId: userData['id']?.toString() ?? '',
        nickName: userData['nick_name']?.toString(),
        faceUrl: userData['face_url']?.toString(),
        selfSignature: userData['self_signature']?.toString(),
        gender: userData['gender'] as int?,
      );
    }).toList();
  } catch (e) {
    debugPrint('批量获取用户资料失败: $e');
    return [];
  }
}