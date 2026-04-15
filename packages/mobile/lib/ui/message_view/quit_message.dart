import 'package:flutter/material.dart';
import 'package:wechat_flutter/im/model/x_message.dart';

/// 退出群聊系统消息
class QuitMessage extends StatelessWidget {
  final XMessage model;
  const QuitMessage(this.model, {super.key});

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(vertical: 5.0),
      alignment: Alignment.center,
      child: Text('[系统消息] 有成员退出', style: TextStyle(color: Colors.grey, fontSize: 12)),
    );
  }
}