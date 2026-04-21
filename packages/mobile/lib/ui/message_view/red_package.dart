import 'package:flutter/material.dart';
import 'package:wechat_flutter/ui/message_view/msg_avatar.dart';
import 'package:wechat_flutter/ui/message_view/text_item_container.dart';
import '../../provider/global_model.dart';
import 'package:provider/provider.dart';
import 'package:wechat_flutter/im/model/x_message.dart';

/// 红包消息 - 暂时以文本形式展示
class RedPackage extends StatelessWidget {
  final XMessage model;
  const RedPackage(this.model, {super.key});

  @override
  Widget build(BuildContext context) {
    final GlobalModel globalModel = Provider.of<GlobalModel>(context);
    final bool self = model.sender == globalModel.account;
    List<Widget> body = <Widget>[
      MsgAvatar(model: model, globalModel: globalModel),
      TextItemContainer(text: '[红包] ${model.content ?? ''}', action: '', isMyself: self),
      const Spacer(),
    ];
    if (self) body = body.reversed.toList();
    return Container(
      padding: const EdgeInsets.symmetric(vertical: 5.0),
      child: Row(crossAxisAlignment: CrossAxisAlignment.start, children: body),
    );
  }
}