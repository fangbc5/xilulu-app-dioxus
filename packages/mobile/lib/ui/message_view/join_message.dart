import 'package:flutter/material.dart';
import 'package:wechat_flutter/im/model/x_message.dart';

/// 加入群聊系统消息
class JoinMessage extends StatelessWidget {
  final XMessage model;
  const JoinMessage(this.model, {super.key});

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(vertical: 5.0),
      alignment: Alignment.center,
      child: Text('[系统消息] 新成员加入', style: TextStyle(color: Colors.grey, fontSize: 12)),
    );
  }
}