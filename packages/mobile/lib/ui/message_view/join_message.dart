import 'package:flutter/material.dart';
import 'package:provider/provider.dart';


import 'package:wechat_flutter/provider/global_model.dart';
import 'package:wechat_flutter/im/tencent_mocks.dart';

class JoinMessage extends StatelessWidget {
  const JoinMessage(this.data, {super.key});

  final V2TimMessage data;

  @override
  Widget build(BuildContext context) {
    final GlobalModel globalModel = Provider.of<GlobalModel>(context);
    final V2TimGroupTipsElem groupTipsElem = data.groupTipsElem!;

    return Container(
      alignment: Alignment.center,
      margin: const EdgeInsets.symmetric(vertical: 5.0),
      child: Text(
        groupTipsElem.opMember.userID == globalModel.account
            ? '你 加入了群聊'
            : '${groupTipsElem.opMember.nickName ?? ""} 加入了群聊',
        style: const TextStyle(
            color: Color.fromRGBO(108, 108, 108, 0.8), fontSize: 11),
      ),
    );
  }
}