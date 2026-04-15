import 'package:extended_text/extended_text.dart';
import 'package:flutter/material.dart';

import 'package:wechat_flutter/tools/wechat_flutter.dart';
import 'package:wechat_flutter/ui/edit/text_span_builder.dart';

/// 会话列表中的最后一条消息内容预览
class ContentMsg extends StatefulWidget {
  final String? msg;

  ContentMsg(this.msg);

  @override
  _ContentMsgState createState() => _ContentMsgState();
}

class _ContentMsgState extends State<ContentMsg> {
  TextStyle _style = TextStyle(color: mainTextColor, fontSize: 14.0);

  @override
  Widget build(BuildContext context) {
    final str = widget.msg ?? '[未知消息]';

    return ExtendedText(
      str,
      specialTextSpanBuilder: TextSpanBuilder(showAtBackground: true),
      maxLines: 1,
      overflow: TextOverflow.ellipsis,
      style: _style,
    );
  }
}