import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:provider/provider.dart';
import 'package:wechat_flutter/im/all_im.dart';
import 'package:wechat_flutter/pages/mine/personal_info_page.dart';
import 'package:wechat_flutter/pages/settings/language_page.dart';
import 'package:wechat_flutter/pages/wallet/pay_home_page.dart';
import 'package:wechat_flutter/provider/global_model.dart';
import 'package:wechat_flutter/tools/wechat_flutter.dart';
import 'package:wechat_flutter/ui/view/list_tile_view.dart';
import '../../src/rust/api/auth.dart' as rust_api;

class MinePage extends StatefulWidget {
  @override
  _MinePageState createState() => new _MinePageState();
}

class _MinePageState extends State<MinePage> {
  void action(name) {
    switch (name) {
      case '设置':
        rust_api.coreLogout().then((_) {
          ImLoginManager.loginOut(context);
        }).catchError((e) {
          debugPrint('coreLogout failed: $e');
          ImLoginManager.loginOut(context);
        });
        break;
      case '服务':
        Get.to<void>(new PayHomePage());
        break;
      default:
        Get.to<void>(new LanguagePage());
        break;
    }
  }

  Widget buildContent(Map<String, String> item) {
    bool isGapAfter = item['label'] == '服务' || item['label'] == '表情' || item['label'] == '设置';
    return new ListTileView(
      border: isGapAfter ? null : Border(bottom: BorderSide(color: lineColor, width: 0.2)),
      title: item['label']!,
      titleStyle: TextStyle(fontSize: 16.0, color: Color(0xFF333333)),
      isLabel: false,
      padding: EdgeInsets.symmetric(vertical: 16.0),
      icon: item['icon']!,
      margin: EdgeInsets.only(bottom: isGapAfter ? 10.0 : 0.0),
      onPressed: () => action(item['label']),
      width: 25.0,
      fit: BoxFit.cover,
      horizontal: 15.0,
    );
  }

  Widget dynamicAvatar(String avatar, {double? size}) {
    return new ImageView(
        img: avatar,
        width: size,
        height: size,
        fit: BoxFit.fill);
  }

  Widget body(GlobalModel model) {
    List<Map<String, String>> data = [
      {'label': '服务', 'icon': 'assets/images/mine/ic_pay.png'},
      {'label': '收藏', 'icon': 'assets/images/favorite.webp'},
      {'label': '朋友圈', 'icon': 'assets/images/discover/ff_Icon_album.webp'},
      {'label': '视频号与公众号', 'icon': 'assets/images/discover/ff_Icon_browse.webp'},
      {'label': '小店与卡包', 'icon': 'assets/images/mine/ic_card_package.png'},
      {'label': '表情', 'icon': 'assets/images/mine/ic_emoji.png'},
      {'label': '设置', 'icon': 'assets/images/mine/ic_setting.png'},
    ];

    var topRow = new Row(
      crossAxisAlignment: CrossAxisAlignment.center,
      children: [
        new SizedBox(
          width: 68.0,
          height: 68.0,
          child: new ClipRRect(
            borderRadius: BorderRadius.all(Radius.circular(8.0)),
            child: strNoEmpty(model.avatar)
                ? dynamicAvatar(model.avatar)
                : new Image.asset(defIcon, fit: BoxFit.cover),
          ),
        ),
        SizedBox(width: 15.0),
        new Expanded(
          child: new Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            mainAxisAlignment: MainAxisAlignment.center,
            children: <Widget>[
              new Row(
                crossAxisAlignment: CrossAxisAlignment.center,
                children: [
                  new Expanded(
                    child: new Text(
                      model.nickName ?? model.account,
                      style: TextStyle(
                          color: Colors.black,
                          fontSize: 22.0,
                          fontWeight: FontWeight.w600),
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                    ),
                  ),
                  new Container(
                    width: 14.0,
                    margin: EdgeInsets.only(right: 0.0),
                    child: new Image.asset('assets/images/mine/ic_small_code.png',
                        color: mainTextColor.withOpacity(0.5), fit: BoxFit.cover),
                  ),
                ],
              ),
              SizedBox(height: 5),
              new Row(
                crossAxisAlignment: CrossAxisAlignment.center,
                children: [
                  new Expanded(
                    child: new Text(
                      '微信号：' + model.account,
                      style: TextStyle(color: mainTextColor, fontSize: 14),
                    ),
                  ),
                  new Image.asset('assets/images/ic_right_arrow_grey.webp',
                      width: 7.0, 
                      color: mainTextColor.withOpacity(0.5), // Match the exact color of bottom list items
                      fit: BoxFit.cover),
                ],
              ),
            ],
          ),
        ),
      ],
    );

    var statusRow = new Row(
      children: [
        SizedBox(width: 68.0 + 15.0), // Align with text left margin
        new Container(
          padding: EdgeInsets.symmetric(horizontal: 8.0, vertical: 2.0),
          decoration: BoxDecoration(
            border: Border.all(color: Colors.grey.withOpacity(0.5)),
            borderRadius: BorderRadius.circular(15.0),
          ),
          child: Row(
            children: [
              Icon(Icons.add, size: 12, color: Colors.grey),
              SizedBox(width: 2),
              Text("状态", style: TextStyle(color: Colors.grey, fontSize: 12)),
            ],
          ),
        ),
        SizedBox(width: 10),
        Icon(Icons.panorama_fish_eye, size: 16, color: Colors.grey.withOpacity(0.6)),
      ],
    );

    return new Column(
      children: <Widget>[
        new InkWell(
          child: new Container(
            color: Colors.white,
            padding: EdgeInsets.only(left: 20.0, right: 20.0, top: topBarHeight(context) - 5.0, bottom: 25.0),
            child: new Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                topRow,
                SizedBox(height: 3.0),
                statusRow,
              ],
            ),
          ),
          onTap: () => Get.to<void>(new PersonalInfoPage()),
        ),
        new SizedBox(height: 10.0),
        new Column(children: data.map(buildContent).toList()),
      ],
    );
  }

  @override
  Widget build(BuildContext context) {
    final model = Provider.of<GlobalModel>(context);

    return new Container(
      color: appBarColor,
      child: new SingleChildScrollView(child: body(model)),
    );
  }
}
