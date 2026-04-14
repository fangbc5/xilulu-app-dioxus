import os
import re

MOCK_FILE_CONTENT = """
class V2TimMessage {
  int? status;
  int? elemType;
  dynamic textElem;
  dynamic faceElem;
  dynamic customElem;
  dynamic imageElem;
  dynamic soundElem;
  dynamic videoElem;
  String? msgID;
  int? timestamp;
  bool isSelf = false;
  String? sender;
  String? nickName;
  String? friendRemark;
  String? faceUrl;
  String? nameCard;
  String? groupID;
  String? userID;
}

class V2TimConversation {
  String? conversationID;
  int? type;
  String? userID;
  String? groupID;
  String? showName;
  String? faceUrl;
  int? unreadCount;
  V2TimMessage? lastMessage;
  dynamic draftText;
  int? draftTimestamp;
  bool? isPinned;
}

class V2TimFriendInfo {
  String? userID;
  String? friendRemark;
  String? friendAddSource;
  String? friendAddWording;
  V2TimUserFullInfo? userProfile;
}

class V2TimFriendInfoResult {
  int? resultCode;
  String? resultInfo;
  int? relation;
  V2TimFriendInfo? friendInfo;
}

class V2TimUserFullInfo {
  String? userID;
  String? nickName;
  String? faceUrl;
  String? selfSignature;
  int? gender;
  int? role;
  int? level;
  int? allowType;
}

class V2TimFriendApplication {
  String? userID;
  String? nickName;
  String? faceUrl;
  String? addTime;
  String? addSource;
  String? addWording;
  int? type;
}

class V2TimValueCallback<T> {
  int code = 0;
  String desc = '';
  T? data;
}

class V2TimCallback {
  int code = 0;
  String desc = '';
}

class V2TimMessageReceipt {
  String? userID;
  int? timestamp;
}

class V2TimGroupChangeInfo {
  int? type;
  String? value;
}

class V2TimGroupMemberChangeInfo {
  String? userID;
}

class V2TimGroupMemberInfo {
  String? userID;
  String? nickName;
  String? friendRemark;
  String? nameCard;
  String? faceUrl;
}

class V2TimUserStatus {
  String? userID;
  int? statusType;
  dynamic customStatus;
}
"""

def main():
    root_dir = "/Volumes/fangbc/RustProjects/xilulu-app-dioxus/packages/mobile/lib"
    mock_file_path = os.path.join(root_dir, "im", "tencent_mocks.dart")
    
    with open(mock_file_path, "w") as f:
        f.write(MOCK_FILE_CONTENT)
    
    import_pattern = re.compile(r"import 'package:tencent_cloud_chat_sdk/[^']+';")

    for dirpath, _, filenames in os.walk(root_dir):
        for filename in filenames:
            if not filename.endswith(".dart"):
                continue
                
            filepath = os.path.join(dirpath, filename)
            if filepath == mock_file_path:
                continue
                
            with open(filepath, "r") as f:
                content = f.read()
                
            if "package:tencent_cloud_chat_sdk" in content:
                print(f"Modifying {filepath}")
                # Replace all tencent imports with our single mock import
                new_content = import_pattern.sub("", content)
                # Ensure the mock import is added once
                if "import 'package:wechat_flutter/im/tencent_mocks.dart';" not in new_content:
                  lines = new_content.splitlines()
                  import_idx = 0
                  for i, line in enumerate(lines):
                      if line.startswith("import "):
                          import_idx = i
                  lines.insert(import_idx + 1 if import_idx else 0, "import 'package:wechat_flutter/im/tencent_mocks.dart';")
                  new_content = "\n".join(lines)
                
                with open(filepath, "w") as f:
                    f.write(new_content)

if __name__ == "__main__":
    main()
