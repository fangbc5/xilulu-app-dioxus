import 'package:flutter/material.dart';
import 'package:wechat_flutter/im/model/x_message.dart';

/// 修改群公告系统消息
class ModifyNotificationMessage extends StatelessWidget {
  final XMessage model;
  const ModifyNotificationMessage(this.model, {super.key});

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(vertical: 5.0),
      alignment: Alignment.center,
      child: Text('[系统消息] 群公告变更', style: TextStyle(color: Colors.grey, fontSize: 12)),
    );
  }
}