import 'package:flutter/material.dart';
import 'package:get/get.dart';

import 'package:wechat_flutter/im/conversation_handle.dart';
import 'package:wechat_flutter/im/model/chat_list.dart';
import 'package:wechat_flutter/pages/chat/chat_page.dart';
import 'package:wechat_flutter/tools/wechat_flutter.dart';
import 'package:wechat_flutter/ui/chat/my_conversation_view.dart';
import 'package:wechat_flutter/ui/edit/text_span_builder.dart';
import 'package:wechat_flutter/ui/view/indicator_page_view.dart';
import 'package:wechat_flutter/ui/view/pop_view.dart';
import 'package:wechat_flutter/ui/view/connection_status_bar.dart';

import '../../tools/event/im_event.dart';
import 'package:wechat_flutter/im/model/im_models.dart';
import 'package:wechat_flutter/im/model/x_message.dart';

class HomePage extends StatefulWidget {
  @override
  _HomePageState createState() => _HomePageState();
}

class _HomePageState extends State<HomePage>
    with AutomaticKeepAliveClientMixin {
  List<XConversation?> _chatData = [];
  String _connectionStatus = 'connected';

  Offset? tapPos;
  TextSpanBuilder _builder = TextSpanBuilder();
  StreamSubscription<dynamic>? _msgStreamSubs;

  @override
  void initState() {
    super.initState();
    initPlatformState();
    getChatData();
  }

  Future<void> getChatData() async {
    final List<XConversation?> listChat =
        await ChatListData().chatListData();
    if (!listNoEmpty(listChat)) {
      return;
    }
    _chatData.clear();
    _chatData.addAll(listChat.reversed.toList());
    if (mounted) {
      setState(() {});
    }
  }

  void _showMenu(BuildContext context, Offset tapPos, int type, String id) {
    final RenderBox overlay =
        Overlay.of(context).context.findRenderObject()! as RenderBox;
    final RelativeRect position = RelativeRect.fromLTRB(tapPos.dx, tapPos.dy,
        overlay.size.width - tapPos.dx, overlay.size.height - tapPos.dy);
    showMenu<String>(
        context: context,
        position: position,
        items: <MyPopupMenuItem<String>>[
          const MyPopupMenuItem(value: '标为已读', child: Text('标为已读')),
          const MyPopupMenuItem(value: '置顶聊天', child: Text('置顶聊天')),
          const MyPopupMenuItem(value: '删除该聊天', child: Text('删除该聊天')),
        ]).then<void>((String? selected) async {
      switch (selected) {
        case '删除该聊天':
          deleteConversationAndLocalMsgModel(id, type);
          getChatData();
          break;
        case '标为已读':
          final num = await getUnreadMessageNumModel(type, id);
          if (num > 0) {
            setReadMessageModel(type, id);
            setState(() {});
          }
          break;
      }
    });
  }

  void canCelListener() {
    if (_msgStreamSubs != null) {
      _msgStreamSubs!.cancel();
    }
  }

  Future<void> initPlatformState() async {
    if (!mounted) {
      return;
    }

    Notice.addListener(WeChatActions.msg(), (v) {
        if (v is Map && v['type'] == 'LATEST_UPDATE') {
           final roomId = v['room_id'];
           final content = v['content'];
           final time = v['time'];
           
           final index = _chatData.indexWhere((m) {
             if (m == null) return false;
             return m.conversationId == roomId || m.peerId == roomId;
           });
           
           if (index != -1) {
              setState(() {
                 _chatData[index]!.lastMessage = XMessage(
                  msgId: '',
                  roomId: int.tryParse(roomId.toString()) ?? 0,
                  fromUid: 0,
                  content: content,
                  type: 1,
                  localStatus: 0,
                  createdAt: time
               );
                 // 置顶（让有新消息的顶上来）
                 final moved = _chatData.removeAt(index);
                 _chatData.insert(0, moved);
              });
           }
        }
    });

    _msgStreamSubs ??= eventBusNewMsg.listen((EventBusNewMsg onData) {
      if (onData.covId.startsWith('LATEST_UPDATE_')) {
         // 静默刷新已由 Notice 接管
      } else {
         getChatData();
      }
    });

    // 监听 WS 连接状态变更
    Notice.addListener(WeChatActions.connectionStatus(), (v) {
      if (v is String && mounted) {
        setState(() {
          _connectionStatus = v;
        });
      }
    });
  }

  bool modelIsGroup(XConversation m) {
    return m.type == 2 || m.type == ConversationType.group;
  }

  @override
  bool get wantKeepAlive => true;

  Widget timeView(int time) {
    final DateTime dateTime = DateTime.fromMillisecondsSinceEpoch(time * 1000);

    final String hourParse = "0${dateTime.hour}";
    final String minuteParse = "0${dateTime.minute}";

    final String hour = dateTime.hour.toString().length == 1
        ? hourParse
        : dateTime.hour.toString();
    final String minute = dateTime.minute.toString().length == 1
        ? minuteParse
        : dateTime.minute.toString();

    final String timeStr = '$hour:$minute';

    return Text(
      timeStr,
      maxLines: 1,
      overflow: TextOverflow.ellipsis,
      style: const TextStyle(color: mainTextColor, fontSize: 14.0),
    );
  }

  @override
  Widget build(BuildContext context) {
    super.build(context);
    if (!listNoEmpty(_chatData)) {
      return HomeNullView();
    }
    return Container(
      color: const Color(AppColors.BackgroundColor),
      child: Column(
        children: [
          ConnectionStatusBar(status: _connectionStatus),
          Expanded(
            child: ScrollConfiguration(
              behavior: MyBehavior(),
              child: ListView.builder(
                itemBuilder: (BuildContext context, int index) {
                  final XConversation? model = _chatData[index];
                  if (model == null) {
                    return Container();
                  }

                  return InkWell(
                    onTap: () {
                      Get.to<void>(ChatPage(
                          id: model.peerId ?? model.conversationId,
                          title: model.showName ?? model.conversationId,
                          type: model.type));
                    },
                    onTapDown: (TapDownDetails details) {
                      tapPos = details.globalPosition;
                    },
                    onLongPress: () {
                      _showMenu(
                        context,
                        tapPos!,
                        model.type == ConversationType.group ? 2 : 1,
                        model.conversationId,
                      );
                    },
                    child: MyConversationView(
                      imageUrl: model.faceUrl,
                      title: model.showName ?? '',
                      content: model.lastMessage?.content,
                      time: timeView(model.lastMessage?.timestamp ?? 0),
                      isBorder: model.showName != _chatData[0]?.showName,
                    ),
                  );
                },
                itemCount: _chatData.length ?? 1,
              ),
            ),
          ),
        ],
      ),
    );
  }

  @override
  void dispose() {
    Notice.removeListenerByEvent(WeChatActions.msg());
    Notice.removeListenerByEvent(WeChatActions.connectionStatus());
    super.dispose();
    canCelListener();
  }
}