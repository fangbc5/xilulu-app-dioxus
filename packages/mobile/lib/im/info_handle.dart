/// 用户信息管理 - 自研 SDK 实现
/// TODO: 后续对接 Rust FFI 的用户信息接口
import 'dart:developer';

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'package:wechat_flutter/provider/global_model.dart';
import 'package:wechat_flutter/tools/wechat_flutter.dart';
import 'package:wechat_flutter/im/model/im_models.dart';

/// 获取备注
Future<String?> getRemarkMethod(String id) async {
  // TODO: 从 Rust SDK 获取用户备注
  return "";
}

/// 设置用户资料
/// TODO: 对接 Rust SDK 更新用户资料
Future<bool> setUsersProfileMethod(
  BuildContext context, {
  String nickNameStr = '',
  String avatarStr = '',
}) async {
  final model = Provider.of<GlobalModel>(context, listen: false);
  if (nickNameStr.isNotEmpty) {
    model.nickName = nickNameStr;
  }
  if (avatarStr.isNotEmpty) {
    model.avatar = avatarStr;
  }
  // TODO: 调用 Rust FFI 更新用户资料到服务端
  return true;
}

/// 获取用户资料列表
/// TODO: 对接 Rust SDK 批量获取用户资料
Future<List<XUserInfo>> getUsersProfile(List<String> users) async {
  // 暂时返回空列表
  return [];
}