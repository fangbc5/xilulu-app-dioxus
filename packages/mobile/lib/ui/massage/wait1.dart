import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart';

import 'package:wechat_flutter/ui/message_view/text_msg.dart';
import 'package:wechat_flutter/im/model/x_message.dart';

/// 消息渲染路由器：根据 XMessage.msgType 分发到不同的渲染组件
class SendMessageView extends StatefulWidget {
  const SendMessageView(this.model, {super.key});

  final XMessage model;

  @override
  State<SendMessageView> createState() => _SendMessageViewState();
}

class _SendMessageViewState extends State<SendMessageView> {
  @override
  Widget build(BuildContext context) {
    final XMessage msg = widget.model;

    // 当前阶段只实现文本消息渲染，后续逐步添加图片、语音、视频等
    switch (msg.msgType) {
      case 0: // 文本消息
        return TextMsg(msg.content, widget.model);
      // TODO: case 1 图片消息
      // TODO: case 2 语音消息
      // TODO: case 3 视频消息
      default:
        return TextMsg(msg.content.isEmpty ? '未知消息类型' : msg.content, widget.model);
    }
  }
}