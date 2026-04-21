import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart';

import 'package:wechat_flutter/ui/message_view/text_msg.dart';
import 'package:wechat_flutter/im/model/x_message.dart';

/// 消息渲染路由器：根据 XMessage.type 分发到不同的渲染组件
/// 服务端 type 定义：1文本 2图片 3文件 4语音 5视频 6撤回 7系统
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
    final String displayContent = msg.content ?? '';

    switch (msg.type) {
      case 1: // 文本消息
        return TextMsg(displayContent, widget.model);
      case 6: // 撤回消息
        return TextMsg('[消息已撤回]', widget.model);
      // TODO: case 2 图片消息
      // TODO: case 3 文件消息
      // TODO: case 4 语音消息
      // TODO: case 5 视频消息
      default:
        return TextMsg(displayContent.isEmpty ? '未知消息类型' : displayContent, widget.model);
    }
  }
}