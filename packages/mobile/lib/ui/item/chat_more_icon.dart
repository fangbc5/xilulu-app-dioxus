import 'package:flutter/material.dart';
import 'package:flutter/cupertino.dart';
import 'package:wechat_flutter/tools/wechat_flutter.dart';

class ChatMoreIcon extends StatelessWidget {
  final bool isMore;
  final String? value;
  final VoidCallback? onTap;
  final GestureTapCallback? moreTap;

  ChatMoreIcon({
    this.isMore = false,
    this.value,
    this.onTap,
    this.moreTap,
  });

  @override
  Widget build(BuildContext context) {
    return strNoEmpty(value)
        ? ComMomButton(
      text: '发送',
      style: TextStyle(color: Colors.white),
      width: 45.0,
      margin: EdgeInsets.all(10.0),
      radius: 4.0,
      onTap: onTap ?? () {},
    )
        : InkWell(
      child: Container(
        child: const Icon(
          CupertinoIcons.add_circled,
          size: 28,
          color: Color(0xff111111),
        ),
      ),
      onTap: moreTap,
    );
  }
}