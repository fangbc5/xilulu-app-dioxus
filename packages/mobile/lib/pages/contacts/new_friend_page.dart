import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart';
import 'package:get/get.dart';

import 'package:wechat_flutter/im/info_handle.dart';
import 'package:wechat_flutter/pages/more/add_friend_details.dart';
import 'package:wechat_flutter/pages/more/add_friend_page.dart';
import 'package:wechat_flutter/im/friend_handle.dart';
import 'package:wechat_flutter/tools/wechat_flutter.dart';
import 'package:wechat_flutter/ui/orther/label_row.dart';
import 'package:wechat_flutter/ui/view/search_main_view.dart';
import 'package:wechat_flutter/ui/view/search_tile_view.dart';
import 'package:wechat_flutter/im/model/im_models.dart';

class NewFriendPage extends StatefulWidget {
  const NewFriendPage({super.key});

  @override
  State<NewFriendPage> createState() => _NewFriendPageState();
}

class _NewFriendPageState extends State<NewFriendPage> {
  bool isSearch = false;
  bool showBtn = false;
  bool isResult = false;

  FocusNode searchF = FocusNode();
  TextEditingController searchC = TextEditingController();

  List<XFriendApplication> _applies = [];

  @override
  void initState() {
    super.initState();
    _fetchApplies();
  }

  Future<void> _fetchApplies() async {
    final list = await getFriendApplications();
    if (mounted) {
      setState(() {
        _applies = list;
      });
    }
  }

  Future<void> _approveApply(int applyId) async {
    final success = await approveFriendApplication(applyId);
    if (success) {
      showToast('已同意');
      _fetchApplies();
    } else {
      showToast('操作失败');
    }
  }

  Widget _buildApplyItem(XFriendApplication apply) {
    bool isPending = apply.type == 0;
    bool isApproved = apply.type == 1;

    return Container(
      color: Colors.white,
      padding: EdgeInsets.symmetric(horizontal: 15.0, vertical: 10.0),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          ClipRRect(
            borderRadius: BorderRadius.circular(6.0),
            child: ImageView(
              img: apply.faceUrl ?? defIcon,
              width: 50.0,
              height: 50.0,
              fit: BoxFit.cover,
            ),
          ),
          SizedBox(width: 15.0),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                Text(
                  strNoEmpty(apply.nickName) ? apply.nickName! : (apply.userId ?? ''),
                  style: TextStyle(fontSize: 16.0, color: Colors.black, fontWeight: FontWeight.w500),
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                ),
                if (strNoEmpty(apply.addWording))
                  Padding(
                    padding: EdgeInsets.only(top: 4.0),
                    child: Text(
                      apply.addWording!,
                      style: TextStyle(fontSize: 13.0, color: Color(0xFF999999)),
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                    ),
                  ),
              ],
            ),
          ),
          SizedBox(width: 10.0),
          if (isPending && apply.id != null)
            ElevatedButton(
              onPressed: () => _approveApply(apply.id!),
              child: Text('接受', style: TextStyle(color: Colors.white, fontSize: 14.0)),
              style: ElevatedButton.styleFrom(
                backgroundColor: Color(0xFF07C160), // WeChat Green
                elevation: 0,
                padding: EdgeInsets.symmetric(horizontal: 15.0, vertical: 0.0),
                minimumSize: Size(60, 30),
                shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(4.0)),
              ),
            )
          else if (isApproved)
            Text('已添加', style: TextStyle(fontSize: 14.0, color: Color(0xFF999999)))
          else
            Text('已拒绝', style: TextStyle(fontSize: 14.0, color: Color(0xFF999999))),
        ],
      ),
    );
  }

  Widget body() {
    final List<Widget> content = <Widget>[
      SearchMainView(
        text: '微信号/手机号',
        isBorder: true,
        onTap: () {
          isSearch = true;
          setState(() {});
          searchF.requestFocus();
        },
      ),
      LabelRow(
        headW: Padding(
          padding: const EdgeInsets.only(right: 15.0),
          child: Image.asset('assets/images/contact/ic_voice.png',
              width: 25, fit: BoxFit.cover),
        ),
        label: '添加手机联系人',
      )
    ];

    if (_applies.isNotEmpty) {
      content.add(Padding(
        padding: EdgeInsets.only(left: 15.0, top: 20.0, bottom: 5.0),
        child: Text('好友申请', style: TextStyle(fontSize: 13.0, color: Color(0xFF999999))),
      ));
      
      for (int i = 0; i < _applies.length; i++) {
        content.add(_buildApplyItem(_applies[i]));
        if (i < _applies.length - 1) {
          content.add(Container(
            color: Colors.white,
            padding: EdgeInsets.only(left: 80.0),
            child: HorizontalLine(height: 0.5),
          ));
        }
      }
    }

    return Column(crossAxisAlignment: CrossAxisAlignment.start, children: content);
  }

  List<Widget> searchBody() {
    if (isResult) {
      return <Widget>[
        Container(
          color: Colors.white,
          width: Get.width,
          height: 110.0,
          alignment: Alignment.center,
          child: const Text(
            '该用户不存在',
            style: TextStyle(color: mainTextColor),
          ),
        ),
        const SizedBox(height: mainSpace),
        SearchTileView(searchC.text, type: 1),
        Container(
          color: Colors.white,
          width: Get.width,
          height: Get.height - 185 * 1.38,
        )
      ];
    } else {
      return <Widget>[
        SearchTileView(
          searchC.text,
          onPressed: () => search(searchC.text),
        ),
        Container(
          color: strNoEmpty(searchC.text) ? Colors.white : appBarColor,
          width: Get.width,
          height: strNoEmpty(searchC.text)
              ? (Get.height - 65 * 2.1) - winKeyHeight(context)
              : Get.height,
        )
      ];
    }
  }



  void unFocusMethod() {
    searchF.unfocus();
    isSearch = false;
    if (isResult) isResult = !isResult;
    setState(() {});
  }

  /// 搜索好友
  Future search(String userName) async {
    final List<XUserInfo> data =
        await searchUser(userName);
    if (data.isEmpty) {
      isResult = true;
      setState(() {});
      return;
    }
    final XUserInfo model = data[0];
    if (model.nickName != null) {
      Get.to<void>(() => AddFriendsDetails('search', model.userId ?? '',
          model.faceUrl ?? '', model.nickName ?? '', model.gender ?? 0));
    } else {
      isResult = true;
      setState(() {});
    }
  }

  @override
  Widget build(BuildContext context) {
    final InkWell leading = InkWell(
      child: Container(
        width: 15,
        height: 28,
        child: const Icon(CupertinoIcons.back, color: Colors.black),
      ),
      onTap: () => unFocusMethod(),
    );

    // ignore: unused_element
    List<Widget> searchView() {
      return <Widget>[
        Expanded(
          child: TextField(
            style: const TextStyle(textBaseline: TextBaseline.alphabetic),
            focusNode: searchF,
            controller: searchC,
            decoration: const InputDecoration(
                hintText: '微信号/手机号', border: InputBorder.none),
            onChanged: (String txt) {
              if (strNoEmpty(searchC.text)) {
                showBtn = true;
              } else {
                showBtn = false;
              }
              if (isResult) isResult = false;

              setState(() {});
            },
            textInputAction: TextInputAction.search,
            onSubmitted: (String txt) => search(txt),
          ),
        ),
        if (strNoEmpty(searchC.text))
          InkWell(
            child: Image.asset('assets/images/ic_delete.webp'),
            onTap: () {
              searchC.text = '';
              setState(() {});
            },
          )
        else
          Container()
      ];
    }

    final SingleChildScrollView bodyView = SingleChildScrollView(
      child: isSearch
          ? GestureDetector(
              child: Column(children: searchBody()),
              onTap: () => unFocusMethod(),
            )
          : body(),
    );

    final TextButton rWidget = TextButton(
      onPressed: () => Get.to<void>(AddFriendPage()),
      child: const Text('添加朋友'),
    );

    return WillPopScope(
      child: Scaffold(
        backgroundColor: appBarColor,
        appBar: ComMomBar(
          leadingW: isSearch ? leading : null,
          title: '新的朋友',
          titleW: isSearch ? Row(children: searchView()) : null,
          rightDMActions: !isSearch ? <Widget>[rWidget] : <Widget>[],
        ),
        body: bodyView,
      ),
      onWillPop: () async {
        if (isSearch) {
          unFocusMethod();
        } else {
          Navigator.pop(context);
        }
        return true;
      },
    );
  }
}