import 'package:flutter/material.dart';
import 'package:wechat_flutter/im/model/x_message.dart';

/// 语音消息 - TODO: 实现语音消息渲染
class SoundMsg extends StatelessWidget {
  final XMessage model;
  const SoundMsg(this.model, {super.key});

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(vertical: 5.0),
      child: Text('[语音消息] - 待实现'),
    );
  }
}