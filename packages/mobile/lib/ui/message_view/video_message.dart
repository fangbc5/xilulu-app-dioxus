import 'package:flutter/material.dart';
import 'package:wechat_flutter/im/model/x_message.dart';

/// 视频消息 - TODO: 实现视频消息渲染
class VideoMessage extends StatelessWidget {
  final XMessage model;
  const VideoMessage(this.model, {super.key});

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(vertical: 5.0),
      child: Text('[视频消息] - 待实现'),
    );
  }
}