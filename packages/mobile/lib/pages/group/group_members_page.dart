import 'package:flutter/material.dart';
import 'package:get/get.dart';

import 'package:wechat_flutter/im/fun_dim_group_model.dart';
import 'package:wechat_flutter/pages/group/select_members_page.dart';
import 'package:wechat_flutter/tools/wechat_flutter.dart';

import '../../im/info_handle.dart';
import 'package:wechat_flutter/im/model/im_models.dart';

class GroupMembersPage extends StatefulWidget {
  final String groupId;

  GroupMembersPage(this.groupId);

  @override
  _GroupMembersPageState createState() => _GroupMembersPageState();
}

class _GroupMembersPageState extends State<GroupMembersPage> {
  late Future<void> _futureBuilderFuture;
  List<XGroupMember?> memberList = <XGroupMember?>[
    XGroupMember(userId: '+'),
  ];

  @override
  void initState() {
    _futureBuilderFuture = _gerData();
    super.initState();
  }

  Future<void> handle(String? uId) async {
    if (!strNoEmpty(uId)) {
      Get.to<void>(SelectMembersPage());
    } else {
      showToast('敬请期待');
    }
  }

  Widget memberItem(XGroupMember? item) {
    if (item == null) {
      return Container();
    }
    if (item.userId == '+' || item.userId == '-') {
      return InkWell(
        child: SizedBox(
          width: (Get.width - 60) / 5,
          child: Image.asset(
            'assets/images/group/${item.userId}.png',
            height: 48.0,
            width: 48.0,
          ),
        ),
        onTap: () => handle(null),
      );
    }

    return FutureBuilder<List<XUserInfo>>(
      future: getUsersProfile(<String>[item.userId]),
      builder:
          (BuildContext context, AsyncSnapshot<List<XUserInfo>> snap) {
        if (snap.connectionState != ConnectionState.done) {
          return Container();
        }
        final XUserInfo currentUser =
            List<XUserInfo>.from(snap.data!).first;
        return SizedBox(
          width: (Get.width - 60) / 5,
          child: TextButton(
            onPressed: () => handle(currentUser.userId),
            style: const ButtonStyle(
              padding: WidgetStatePropertyAll<EdgeInsets>(EdgeInsets.zero),
            ),
            child: Column(
              children: <Widget>[
                ClipRRect(
                  borderRadius: const BorderRadius.all(Radius.circular(5)),
                  child: !strNoEmpty(currentUser.faceUrl)
                      ? Image.asset(
                          defIcon,
                          height: 48.0,
                          width: 48.0,
                          fit: BoxFit.cover,
                        )
                      : CachedNetworkImage(
                          imageUrl: currentUser.faceUrl!,
                          height: 48.0,
                          width: 48.0,
                          cacheManager: cacheManager,
                          fit: BoxFit.cover,
                        ),
                ),
                const SizedBox(height: 2),
                Container(
                  alignment: Alignment.center,
                  height: 20.0,
                  width: 50,
                  child: Text(
                    currentUser.nickName == null || currentUser.nickName == ''
                        ? '默认昵称'
                        : currentUser.nickName!.length > 5
                            ? '${currentUser.nickName!.substring(0, 3)}...'
                            : currentUser.nickName!,
                    style: const TextStyle(fontSize: 12.0),
                  ),
                ),
              ],
            ),
          ),
        );
      },
    );
  }

  Future<void> _gerData() async {
    final List<XGroupMember?> result =
        await DimGroup.getGroupMembersListModelLIST(
      widget.groupId,
    );

    setState(() {
      memberList.insertAll(0, result.toSet());
    });
  }

  Widget titleWidget() {
    return FutureBuilder<void>(
      future: _futureBuilderFuture,
      builder: (BuildContext context, AsyncSnapshot<void> snap) {
        return Text(
          '聊天成员(${memberList.isNotEmpty ? memberList.length - 1 : 0})',
          style: const TextStyle(
              color: Colors.black, fontSize: 17.0, fontWeight: FontWeight.w600),
        );
      },
    );
  }

  @override
  Widget build(BuildContext context) {
    if (!listNoEmpty(memberList)) {
      return Container();
    }

    return Scaffold(
      appBar: ComMomBar(titleW: titleWidget()),
      body: ListView(
        padding: const EdgeInsets.all(10),
        children: <Widget>[
          Wrap(
            runSpacing: 20.0,
            spacing: 10,
            children: memberList.map(memberItem).toList(),
          ),
        ],
      ),
    );
  }
}