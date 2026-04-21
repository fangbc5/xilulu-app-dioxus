import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:provider/provider.dart';

import 'package:wechat_flutter/im/info_handle.dart';
import 'package:wechat_flutter/pages/mine/code_page.dart';
import 'package:wechat_flutter/pages/more/add_friend_details.dart';
import 'package:wechat_flutter/pages/root/user_page.dart';
import 'package:wechat_flutter/provider/global_model.dart';
import 'package:wechat_flutter/tools/wechat_flutter.dart';
import 'package:wechat_flutter/ui/view/list_tile_view.dart';
import 'package:wechat_flutter/ui/view/list_tile_view_modern.dart';
import 'package:wechat_flutter/ui/view/search_main_view.dart';
import 'package:wechat_flutter/ui/view/search_tile_view.dart';
import 'package:wechat_flutter/im/model/im_models.dart';

class AddFriendPage extends StatefulWidget {
  @override
  _AddFriendPageState createState() => new _AddFriendPageState();
}

class _AddFriendPageState extends State<AddFriendPage> {
  bool isSearch = false;
  bool showBtn = false;
  bool isResult = false;

  String? currentUser;

  FocusNode searchF = new FocusNode();
  TextEditingController searchC = new TextEditingController();

  Widget buildItem(Map<String, String> item) {
    return new ListTileViewModern(
      border: item['title'] == '扫一扫'
          ? null
          : Border(top: BorderSide(color: Color(0xFFE5E5E5), width: 0.5)),
      title: item['title']!,
      label: item['label'],
      icon: strNoEmpty(item['icon'])
          ? item['icon']!
          : 'assets/images/favorite.webp',
      fit: BoxFit.cover,
      onPressed: () => Get.to<void>(new UserPage()),
    );
  }

  Widget body() {
    final model = Provider.of<GlobalModel>(context);

    List<Map<String, String>> data = [
      {
        'icon': contactAssets + 'ic_scanqr.webp',
        'title': '扫一扫',
        'label': '扫描二维码名片',
      },
      {
        'icon': contactAssets + 'ic_new_friend.webp',
        'title': '手机联系人',
        'label': '添加通讯录中的朋友',
      },
      {
        'icon': contactAssets + 'ic_reda.webp',
        'title': '雷达',
        'label': '添加身边的朋友',
      },
      {
        'icon': contactAssets + 'ic_search_wework.webp',
        'title': '企业微信联系人',
        'label': '通过手机号搜索企业微信用户',
      },
      {
        'icon': contactAssets + 'ic_group.webp',
        'title': '面对面建群',
        'label': '与身边的朋友进入同一个群聊'
      },
      {
        'icon': contactAssets + 'ic_offical.webp',
        'title': '公众号',
        'label': '获取更多资讯',
      },
      {
        'icon': contactAssets + 'ic_no_public.webp',
        'title': '服务号',
        'label': '获取更多购物信息和服务',
      },
    ];
    
    var searchBar = InkWell(
      onTap: () {
        isSearch = true;
        setState(() {});
        searchF.requestFocus();
      },
      child: Container(
        margin: EdgeInsets.only(left: 15.0, right: 15.0, top: 10.0, bottom: 20.0),
        height: 38.0,
        decoration: BoxDecoration(
          color: Color(0xFFF3F3F3),
          borderRadius: BorderRadius.circular(6.0),
        ),
        child: Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.search, color: mainTextColor.withOpacity(0.5), size: 20.0),
            SizedBox(width: 6.0),
            Text('账号/手机号', style: TextStyle(color: mainTextColor.withOpacity(0.5), fontSize: 16.0)),
          ],
        ),
      ),
    );

    var content = [
      searchBar,
      new Column(children: data.map(buildItem).toList()),
      new Spacer(),
      new Padding(
        padding: EdgeInsets.only(bottom: 40.0),
        child: new Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: <Widget>[
            // 占位用的大尺寸二维码图标
            new Icon(CupertinoIcons.qrcode, size: 160.0, color: Colors.green),
          ],
        ),
      ),
    ];

    return new Column(children: content);
  }

  List<Widget> searchBody() {
    if (isResult) {
      return [
        new Container(
          color: Colors.white,
          width: Get.width,
          height: 110.0,
          alignment: Alignment.center,
          child: new Text(
            '该用户不存在',
            style: TextStyle(color: mainTextColor),
          ),
        ),
        new SizedBox(height: mainSpace),
        new SearchTileView(searchC.text, type: 1),
        new Container(
          color: Colors.white,
          width: Get.width,
          height: (Get.height - 185 * 1.38),
        )
      ];
    } else {
      return [
        new SearchTileView(
          searchC.text,
          onPressed: () => search(searchC.text),
        ),
        new Container(
          color: strNoEmpty(searchC.text) ? Colors.white : appBarColor,
          width: Get.width,
          height: strNoEmpty(searchC.text)
              ? (Get.height - 65 * 2.1) - winKeyHeight(context)
              : Get.height,
        )
      ];
    }
  }

  @override
  void initState() {
    super.initState();
    getUser();
  }

  getUser() async {
    // if (Platform.isAndroid) {
    //   currentUser = await im.getCurrentLoginUser();
    // } else {
    //   currentUser = null;
    // }
    setState(() {});
  }

  unFocusMethod() {
    searchF.unfocus();
    isSearch = false;
    if (isResult) isResult = !isResult;
    setState(() {});
  }

  // 搜索好友
  Future search(String userName) async {
    final List<XUserInfo> data = await getUsersProfile([userName]);
    if (data.isEmpty) {
      showToast('该用户不存在【可搜"188"或"18888"试试】');
      return;
    }
    setState(() {
      if (Platform.isIOS) {
        XUserInfo model = data[0];
        if (model.nickName != null) {
          Get.to<void>(new AddFriendsDetails('search', model.userId,
              model.faceUrl ?? '', model.nickName ?? '', model.gender ?? 0));
        } else {
          isResult = true;
        }
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    var leading = new InkWell(
      child: new Container(
        width: 15,
        height: 28,
        child: new Icon(CupertinoIcons.back, color: Colors.black),
      ),
      onTap: () => unFocusMethod(),
    );

    // ignore: unused_element
    List<Widget> searchView() {
      return [
        new Expanded(
          child: new TextField(
            focusNode: searchF,
            controller: searchC,
            style: TextStyle(textBaseline: TextBaseline.alphabetic),
            decoration:
                InputDecoration(hintText: '微信号/手机号', border: InputBorder.none),
            onChanged: (txt) {
              if (strNoEmpty(searchC.text))
                showBtn = true;
              else
                showBtn = false;
              if (isResult) isResult = false;

              setState(() {});
            },
            textInputAction: TextInputAction.search,
            onSubmitted: (txt) => search(txt),
          ),
        ),
        strNoEmpty(searchC.text)
            ? new InkWell(
                child: new Image.asset('assets/images/ic_delete.webp'),
                onTap: () {
                  searchC.text = '';
                  setState(() {});
                },
              )
            : new Container()
      ];
    }

    var bodyView = isSearch
        ? new SingleChildScrollView(
            child: new GestureDetector(
              child: new Column(children: searchBody()),
              onTap: () => unFocusMethod(),
            ),
          )
        : body();

    return WillPopScope(
      child: new Scaffold(
        backgroundColor: Colors.white,
        appBar: new ComMomBar(
          backgroundColor: Colors.white,
          centerTitle: true,
          leadingW: isSearch ? leading : null,
          title: '添加朋友',
          titleW: isSearch ? new Row(children: searchView()) : null,
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