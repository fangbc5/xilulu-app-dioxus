import 'package:flutter/material.dart';
import 'package:wechat_flutter/im/model/x_message.dart';

/// 修改群资料系统消息
class ModifyGroupInfoMessage extends StatelessWidget {
  final XMessage model;
  const ModifyGroupInfoMessage(this.model, {super.key});

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(vertical: 5.0),
      alignment: Alignment.center,
      child: Text('[系统消息] 群资料变更', style: TextStyle(color: Colors.grey, fontSize: 12)),
    );
  }
}