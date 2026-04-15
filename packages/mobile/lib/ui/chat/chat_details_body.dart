import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';

import 'package:wechat_flutter/ui/massage/wait1.dart';
import 'package:wechat_flutter/ui/view/indicator_page_view.dart';
import 'package:wechat_flutter/im/model/x_message.dart';

class ChatDetailsBody extends StatelessWidget {
  final ScrollController sC;
  final List<XMessage> chatData;

  const ChatDetailsBody({
    Key? key,
    required this.sC,
    required this.chatData,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Flexible(
      child: ScrollConfiguration(
        behavior: MyBehavior(),
        child: ListView.builder(
          controller: sC,
          padding: EdgeInsets.all(8.0),
          reverse: true,
          itemBuilder: (context, int index) {
            final XMessage model = chatData[index];
            return SendMessageView(model);
          },
          itemCount: chatData.length,
          dragStartBehavior: DragStartBehavior.down,
        ),
      ),
    );
  }
}