import 'package:flutter/material.dart';
import 'package:wechat_flutter/im/model/x_message.dart';

/// 图片消息 - TODO: 实现图片消息渲染
class ImgMsg extends StatelessWidget {
  final XMessage model;
  const ImgMsg(this.model, {super.key});

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(vertical: 5.0),
      child: Text('[图片消息] - 待实现'),
    );
  }
}