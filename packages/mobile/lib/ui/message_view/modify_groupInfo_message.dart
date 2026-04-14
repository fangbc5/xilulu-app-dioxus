import 'package:wechat_flutter/im/tencent_mocks.dart';
import 'package:flutter/material.dart';



class ModifyGroupInfoMessage extends StatefulWidget {
  const ModifyGroupInfoMessage(this.msg, {super.key});

  final V2TimMessage msg;

  @override
  ModifyGroupInfoMessageState createState() => ModifyGroupInfoMessageState();
}

class ModifyGroupInfoMessageState extends State<ModifyGroupInfoMessage> {
  @override
  Widget build(BuildContext context) {
    final V2TimGroupTipsElem groupTipsElem = widget.msg.groupTipsElem!;

    if (groupTipsElem.groupChangeInfoList?.isNotEmpty ?? false) {
      return Container(
        alignment: Alignment.center,
        margin: const EdgeInsets.symmetric(vertical: 5.0),
        child: Text(
          '${groupTipsElem.opMember.nickName ?? ''} 修改群名称为 ”${groupTipsElem.groupChangeInfoList?.first?.value ?? ""}“',
          style: const TextStyle(
              color: Color.fromRGBO(108, 108, 108, 0.8), fontSize: 11),
        ),
      );
    } else {
      return Container();
    }
  }
}