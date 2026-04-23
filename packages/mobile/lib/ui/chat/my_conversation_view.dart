import 'package:flutter/material.dart';
import 'package:get/get.dart';

import 'package:wechat_flutter/tools/wechat_flutter.dart';
import 'package:wechat_flutter/ui/message_view/content_msg.dart';

class MyConversationView extends StatefulWidget {
  final String? imageUrl;
  final String? title;
  final String? content;
  final Widget? time;
  final bool isBorder;
  final int unreadCount;

  const MyConversationView({
    Key? key,
    this.imageUrl,
    this.title,
    this.content,
    this.time,
    this.isBorder = true,
    this.unreadCount = 0,
  }) : super(key: key);

  @override
  _MyConversationViewState createState() => _MyConversationViewState();
}

class _MyConversationViewState extends State<MyConversationView> {
  @override
  Widget build(BuildContext context) {
    var row = Row(
      children: <Widget>[
        SizedBox(width: mainSpace),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: <Widget>[
              Text(
                widget.title ?? '',
                style: TextStyle(fontSize: 17.0, fontWeight: FontWeight.normal),
              ),
              SizedBox(height: 2.0),
              ContentMsg(widget.content),
            ],
          ),
        ),
        SizedBox(width: mainSpace),
        Column(
          children: [
            widget.time ?? SizedBox.shrink(),
            Icon(Icons.flag, color: Colors.transparent),
          ],
        )
      ],
    );

    return Container(
      padding: EdgeInsets.only(left: 18.0),
      color: Colors.white,
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          Stack(
            clipBehavior: Clip.none,
            children: [
              ImageView(
                  img: widget.imageUrl ?? "",
                  height: 50.0,
                  width: 50.0,
                  fit: BoxFit.cover),
              if (widget.unreadCount > 0)
                Positioned(
                  right: -4,
                  top: -4,
                  child: Container(
                    padding: EdgeInsets.symmetric(
                        horizontal: widget.unreadCount > 9 ? 4.0 : 0.0, 
                        vertical: 2.0),
                    decoration: BoxDecoration(
                      color: Colors.red,
                      borderRadius: BorderRadius.circular(10.0),
                      border: Border.all(color: Colors.white, width: 1.0),
                    ),
                    constraints: BoxConstraints(
                      minWidth: 18,
                      minHeight: 18,
                    ),
                    alignment: Alignment.center,
                    child: Text(
                      widget.unreadCount > 99 ? '···' : '${widget.unreadCount}',
                      style: TextStyle(
                        color: Colors.white,
                        fontSize: 11,
                        fontWeight: FontWeight.bold,
                        height: 1.0,
                      ),
                      textAlign: TextAlign.center,
                    ),
                  ),
                ),
            ],
          ),
          Container(
            padding: EdgeInsets.only(right: 18.0, top: 12.0, bottom: 12.0),
            width: Get.width - 68,
            decoration: BoxDecoration(
              border: widget.isBorder
                  ? Border(
                      top: BorderSide(color: lineColor, width: 0.2),
                    )
                  : null,
            ),
            child: row,
          )
        ],
      ),
    );
  }
}