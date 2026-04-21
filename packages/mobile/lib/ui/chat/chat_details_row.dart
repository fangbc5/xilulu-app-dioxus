import 'package:flutter/material.dart';
import 'package:flutter/cupertino.dart';
import 'package:wechat_flutter/im/message_handle.dart';
import 'package:wechat_flutter/tools/wechat_flutter.dart';
import 'package:wechat_flutter/ui/item/chat_voice.dart';

class ChatDetailsRow extends StatefulWidget {
  final GestureTapCallback? voiceOnTap;
  final bool isVoice;
  final LayoutWidgetBuilder edit;
  final VoidCallback onEmojio;
  final Widget more;
  final String id;
  final int type;

  ChatDetailsRow({
    this.voiceOnTap,
    required this.isVoice,
    required this.edit,
    required this.more,
    required this.id,
    required this.type,
    required this.onEmojio,
  });

  ChatDetailsRowState createState() => ChatDetailsRowState();
}

class ChatDetailsRowState extends State<ChatDetailsRow> {
  String? path;

  @override
  void initState() {
    super.initState();

    Notice.addListener(WeChatActions.voiceImg(), (v) {
      if (v is! bool) {
        return;
      }
      if (!v) return;
      if (!strNoEmpty(path)) return;
      debugPrint("Mock sendSoundMessages: path=$path, targetId=${widget.id}");
    });
  }

  @override
  Widget build(BuildContext context) {
    return new GestureDetector(
      child: new Container(
        constraints: BoxConstraints(minHeight: 56.0), // Standard WeChat bottom bar height
        padding: EdgeInsets.symmetric(horizontal: 10.0), // Outer horizontal space
        decoration: BoxDecoration(
          color: const Color(AppColors.ChatBoxBg),
          border: Border(top: BorderSide(color: const Color(0xffE5E5E5), width: Constants.DividerWidth * 2.5)),
        ),
        child: new Row(
          crossAxisAlignment: CrossAxisAlignment.end,
          children: <Widget>[
            new InkWell(
              child: new Container(
                padding: const EdgeInsets.only(top: 12.0, bottom: 16.0),
                margin: const EdgeInsets.only(right: 8.0),
                child: Image.asset('assets/images/chat/ic_voice.webp',
                    width: 28, fit: BoxFit.contain, color: const Color(0xff111111)),
              ),
              onTap: () {
                if (widget.voiceOnTap != null) {
                  widget.voiceOnTap!();
                }
              },
            ),
            new Expanded(
              child: new Container(
                margin: const EdgeInsets.only(top: 10.0, bottom: 10.0, right: 8.0),
                constraints: const BoxConstraints(minHeight: 38.0),
                decoration: BoxDecoration(
                    color: Colors.white,
                    borderRadius: BorderRadius.circular(6.0)),
                child: widget.isVoice
                    ? new ChatVoice(
                        voiceFile: (path) {
                          setState(() => this.path = path);
                        },
                      )
                    : new LayoutBuilder(builder: widget.edit),
              ),
            ),
            new InkWell(
              child: new Container(
                padding: const EdgeInsets.symmetric(vertical: 14.0),
                margin: const EdgeInsets.only(right: 8.0),
                child: const Icon(CupertinoIcons.smiley, size: 28, color: Color(0xff111111)),
              ),
              onTap: () {
                widget.onEmojio();
              },
            ),
            new Container(
              padding: const EdgeInsets.symmetric(vertical: 14.0),
              child: widget.more,
            ),
          ],
        ),
      ),
      onTap: () {},
    );
  }
}
