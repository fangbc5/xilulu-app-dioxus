import 'package:flutter/material.dart';
import 'package:flutter/cupertino.dart';
import 'package:get/get.dart';
import 'package:wechat_flutter/pages/more/verification_page.dart';
import 'package:wechat_flutter/tools/wechat_flutter.dart';
import 'package:wechat_flutter/ui/orther/button_row.dart';
import 'package:wechat_flutter/ui/orther/label_row.dart';
import 'package:wechat_flutter/ui/orther/person_card.dart';
import 'package:wechat_flutter/pages/more/friend_profile_page.dart';

class AddFriendsDetails extends StatefulWidget {
  final String type;
  final String imUser;
  final String avatarImg;
  final String nickName;
  final int gender;

  AddFriendsDetails(
      this.type, this.imUser, this.avatarImg, this.nickName, this.gender);

  @override
  _AddFriendsDetailsState createState() => _AddFriendsDetailsState();
}

class _AddFriendsDetailsState extends State<AddFriendsDetails> {
  Widget body() {
    var content = [
      new Container(
        color: Colors.white,
        child: Column(
          children: [
            new PersonCard(
                imageUrl: widget.avatarImg,
                name: strNoEmpty(widget.nickName) ? widget.nickName : widget.imUser,
            ),
            new Container(
              padding: EdgeInsets.only(left: 20.0),
              child: new HorizontalLine(height: 0.5),
            ),
            new LabelRow(
              label: '朋友资料',
              onPressed: () => Get.to<void>(() => FriendProfilePage(
                imUser: widget.imUser,
                nickName: widget.nickName,
              )),
              isLine: false,
              padding: EdgeInsets.only(top: 16.0, bottom: 10.0, right: 15.0),
            ),
            new LabelRow(
              label: '来源',
              isRight: false,
              labelWidth: 64.0, // Fixed width to align with standard WeChat spacing
              value: '来自手机号搜索',
              labelTextColor: Color(0xFF999999),
              valueTextColor: Color(0xFF999999), // value color should match label color for source
              padding: EdgeInsets.only(top: 4.0, bottom: 22.0, right: 15.0),
            ),
          ],
        )
      ),
      new Space(),
      new ButtonRow(
        text: '添加到通讯录',
        style: TextStyle(
          color: Color(0xFF576B95),
          fontWeight: FontWeight.w600,
          fontSize: 16
        ),
        onPressed: () => Get.to<void>(
            () => VerificationPage(nickName: widget.nickName, id: widget.imUser)),
      ),
    ];

    return new Column(children: content);
  }

  @override
  Widget build(BuildContext context) {
    var rWidget = [
      new InkWell(
        child: new Container(
          padding: EdgeInsets.symmetric(horizontal: 15.0),
          child: new Image.asset('assets/images/right_more.png'),
        ),
        onTap: () {},
      )
    ];

    return Scaffold(
      backgroundColor: appBarColor,
      appBar: new ComMomBar(
          title: '', backgroundColor: Colors.white, rightDMActions: rWidget),
      body: new SingleChildScrollView(child: body()),
    );
  }
}
