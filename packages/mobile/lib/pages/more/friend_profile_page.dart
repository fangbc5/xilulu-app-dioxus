import 'package:flutter/material.dart';
import 'package:wechat_flutter/config/const.dart';
import 'package:wechat_flutter/tools/wechat_flutter.dart';
import 'package:wechat_flutter/ui/orther/label_row.dart';

class FriendProfilePage extends StatefulWidget {
  final String imUser;
  final String nickName;

  FriendProfilePage({required this.imUser, required this.nickName});

  @override
  _FriendProfilePageState createState() => _FriendProfilePageState();
}

class _FriendProfilePageState extends State<FriendProfilePage> {
  Widget _buildSectionHeader(String title) {
    return Container(
      width: double.infinity,
      color: appBarColor,
      padding: EdgeInsets.only(left: 15.0, top: 12.0, bottom: 8.0),
      child: Text(
        title,
        style: TextStyle(color: mainTextColor, fontSize: 13.0),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final nameStr = strNoEmpty(widget.nickName) ? widget.nickName : widget.imUser;

    var content = [
      _buildSectionHeader('备注'),
      new LabelRow(
        label: '备注名',
        isLine: true,
        rValue: nameStr,
        onPressed: () {},
      ),
      new LabelRow(
        label: '标签',
        isLine: true,
        onPressed: () {},
      ),
      new LabelRow(
        label: '备忘',
        isLine: true,
        onPressed: () {},
      ),
      new LabelRow(
        label: '照片',
        onPressed: () {},
      ),
      _buildSectionHeader('更多信息'),
      new LabelRow(
        label: '手机',
        rValue: widget.imUser,
        isLine: true,
        isRight: false,
      ),
      new LabelRow(
        label: '来源',
        rValue: '来自手机号搜索',
        isRight: false,
      ),
    ];

    return Scaffold(
      backgroundColor: appBarColor,
      appBar: new ComMomBar(
          title: '朋友资料', backgroundColor: appBarColor, centerTitle: true),
      body: new SingleChildScrollView(
        child: new Column(children: content),
      ),
    );
  }
}
