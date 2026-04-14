
class V2TimMessage {
  int? status;
  int elemType = 0;
  dynamic textElem;
  dynamic faceElem;
  dynamic customElem;
  dynamic imageElem;
  dynamic soundElem;
  dynamic videoElem;
  dynamic groupTipsElem;
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
  String? id;
}

class V2TimGroupInfo {
  int? memberCount;
  String? owner;
  String? introduction;
  String? groupName;
  String? notification;
  String? groupID;
  String? faceUrl;
}
class V2TimGroupTipsElem {
  late V2TimGroupMemberInfo opMember;
  List<V2TimGroupChangeInfo>? groupChangeInfoList;
  int? type;
}
class V2TimSoundElem {
  String? url;
  String? path;
}
class V2TimMessageOnlineUrl {
  V2TimVideoElem? videoElem;
}

class V2TimVideoElem {
  String? snapshotUrl;
  String? snapshotPath;
  String? videoUrl;
  String? localVideoUrl;
  int? duration;
}
class ConversationType {
  static const int V2TIM_C2C = 1;
  static const int V2TIM_GROUP = 2;
}
class GroupTipsElemType {
  static const int V2TIM_GROUP_TIPS_TYPE_JOIN = 1;
  static const int V2TIM_GROUP_TIPS_TYPE_INVITE = 2;
  static const int V2TIM_GROUP_TIPS_TYPE_QUIT = 3;
  static const int V2TIM_GROUP_TIPS_TYPE_MEMBER_INFO_CHANGE = 4;
  static const int V2TIM_GROUP_TIPS_TYPE_GROUP_INFO_CHANGE = 5;
}
class GroupMemberFilterTypeEnum {
  static const int V2TIM_GROUP_MEMBER_FILTER_ALL = 0;
}
class FriendTypeEnum {
  static const int V2TIM_FRIEND_TYPE_BOTH = 1;
}
class GroupType {
  static const String Public = "Public";
}
class GroupMemberRoleTypeEnum {
  static const int V2TIM_GROUP_MEMBER_ROLE_MEMBER = 3;
}
class V2TimGroupMember {
  String? userID;
  int? role;
  V2TimGroupMember({this.userID, this.role});
}
class V2TimFriendOperationResult {
  int? resultCode;
}
class V2TimGroupMemberInfoResult {
  List<V2TimGroupMemberFullInfo>? memberInfoList;
}
class HistoryMsgGetTypeEnum {
  static const int V2TIM_GET_CLOUD_OLDER_MSG = 1;
  static const int V2TIM_GET_CLOUD_NEWER_MSG = 2;
}
class MessageElemType {
  static const int V2TIM_ELEM_TYPE_TEXT = 1;
  static const int V2TIM_ELEM_TYPE_CUSTOM = 2;
  static const int V2TIM_ELEM_TYPE_IMAGE = 3;
  static const int V2TIM_ELEM_TYPE_SOUND = 4;
  static const int V2TIM_ELEM_TYPE_VIDEO = 5;
  static const int V2TIM_ELEM_TYPE_GROUP_TIPS = 6;
}
class V2TimImageElem {
  List<V2TimImage>? imageList;
}
class V2TimImage {
  int? height;
  String? url;
}
class V2TimCustomElem {
  String? data;
}
class V2TimMessageListResult {
  List<V2TimMessage>? messageList;
}
class V2TimConversationFilter {
  int? conversationType;
  String? conversationGroup;
  V2TimConversationFilter({this.conversationType, this.conversationGroup});
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
  String userID = '';
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
  Map<String, dynamic> toJson() => {};
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

class OfflinePushInfo {}

class V2TimMsgCreateInfoResult {
  String? id;
}

class TencentImSDKPlugin {
  static final _v2TIMManager = V2TIMManager();
  static V2TIMManager get v2TIMManager => _v2TIMManager;
}

class V2TIMManager {
  Future<V2TimValueCallback<T>> initSDK<T>({required int sdkAppID, required int loglevel, required dynamic listener}) async {
    return V2TimValueCallback<T>()..code = 0;
  }
  
  Future<V2TimCallback> login({required String userID, required String userSig}) async {
    return V2TimCallback()..code = 0;
  }
  
  Future<V2TimCallback> logout() async {
    return V2TimCallback()..code = 0;
  }

  Future<V2TimValueCallback<String>> getLoginUser() async {
    return V2TimValueCallback<String>()..code = 0..data = 'mock_user_id';
  }

  Future<V2TimCallback> setSelfInfo({required dynamic userFullInfo}) async {
    return V2TimCallback()..code = 0;
  }

  V2TIMMessageManager getMessageManager() => V2TIMMessageManager();
  V2TIMFriendshipManager getFriendshipManager() => V2TIMFriendshipManager();
  V2TIMConversationManager getConversationManager() => V2TIMConversationManager();
  V2TIMGroupManager getGroupManager() => V2TIMGroupManager();
  dynamic getCommunityManager() => _DynamicMock();

  Future<V2TimValueCallback<List<V2TimUserFullInfo>>> getUsersInfo({required List<String> userIDList}) async {
    return V2TimValueCallback<List<V2TimUserFullInfo>>()..code = 0..data = [];
  }
}

class V2TIMGroupManager {
  Future<V2TimValueCallback<V2TimGroupMemberInfoResult>> getGroupMemberList({required String groupID, required int filter, required String nextSeq}) async {
    return V2TimValueCallback<V2TimGroupMemberInfoResult>()..code = 0..data = V2TimGroupMemberInfoResult();
  }
  Future<V2TimValueCallback<List<V2TimGroupInfoResult>>> getGroupsInfo({required List<String> groupIDList}) async {
    return V2TimValueCallback<List<V2TimGroupInfoResult>>()..code = 0..data = [];
  }
  Future<V2TimValueCallback<List<V2TimGroupInfo>>> getJoinedGroupList() async {
    return V2TimValueCallback<List<V2TimGroupInfo>>()..code = 0..data = [];
  }
  Future<V2TimValueCallback<String>> createGroup({required String groupType, required String groupName, String? groupID, List<V2TimGroupMember>? memberList}) async {
    return V2TimValueCallback<String>()..code = 0..data = '';
  }
}

class V2TimGroupMemberFullInfo {
  String userID = '';
  V2TimGroupMemberFullInfo({this.userID = ''});
}
class V2TimGroupInfoResult {
  String? groupID;
  V2TimGroupInfo? groupInfo;
}

class V2TIMConversationManager {
  Future<V2TimValueCallback<V2TimConversationResult>> getConversationList({required String nextSeq, required int count}) async {
    final fakeConv = V2TimConversation()
      ..conversationID = "c2c_1"
      ..userID = "1"
      ..showName = "测试机器人 (Bot)"
      ..type = 1
      ..unreadCount = 0;
    return V2TimValueCallback<V2TimConversationResult>()..code = 0..data = V2TimConversationResult(nextSeq: "0", isFinished: true, conversationList: [fakeConv]);
  }
  Future<V2TimValueCallback<V2TimConversationResult>> getConversationListByFilter({required dynamic filter, required String nextSeq, required int count}) async {
    return V2TimValueCallback<V2TimConversationResult>()..code = 0..data = V2TimConversationResult(nextSeq: "0", isFinished: true, conversationList: []);
  }
  Future<V2TimCallback> deleteConversation({required String conversationID}) async {
    return V2TimCallback()..code = 0;
  }
  Future<V2TimValueCallback<int>> getUnreadMessageCountByFilter({required V2TimConversationFilter filter}) async {
    return V2TimValueCallback<int>()..code = 0..data = 0;
  }
  Future<V2TimCallback> cleanConversationUnreadMessageCount({
    required String conversationID, 
    int? conversationType,
    int? cleanTimestamp,
    int? cleanSequence
  }) async {
    return V2TimCallback()..code = 0;
  }
}

class V2TimConversationResult {
  String? nextSeq;
  bool? isFinished;
  List<V2TimConversation>? conversationList;
  V2TimConversationResult({this.nextSeq, this.isFinished, this.conversationList});
}

class V2TIMFriendshipManager {
  Future<V2TimValueCallback<List<V2TimFriendApplication>>> getFriendApplicationList() async {
    return V2TimValueCallback<List<V2TimFriendApplication>>()..code = 0..data = [];
  }
  Future<V2TimValueCallback<List<V2TimFriendInfo>>> getFriendList() async {
    final fakeFriend = V2TimFriendInfo()
        ..userID = "1"
        ..friendRemark = "测试机器人 (Bot)"
        ..userProfile = (V2TimUserFullInfo()..nickName = "机器人")
        ..friendAddSource = "system"
        ..friendAddWording = "Local Mock Data";
    return V2TimValueCallback<List<V2TimFriendInfo>>()..code = 0..data = [fakeFriend];
  }
  Future<V2TimValueCallback<List<V2TimFriendInfoResult>>> checkFriend({required List<String> userIDList, required int checkType}) async {
    return V2TimValueCallback<List<V2TimFriendInfoResult>>()..code = 0..data = [];
  }
  Future<V2TimValueCallback<V2TimFriendOperationResult>> addFriend({required String userID, required int addType}) async {
    return V2TimValueCallback<V2TimFriendOperationResult>()..code = 0..data = V2TimFriendOperationResult();
  }
  Future<V2TimValueCallback<List<V2TimFriendOperationResult>>> deleteFromFriendList({required List<String> userIDList, required int deleteType}) async {
    return V2TimValueCallback<List<V2TimFriendOperationResult>>()..code = 0..data = [];
  }
}

class V2TIMMessageManager {
  Future<V2TimValueCallback<V2TimMsgCreateInfoResult>> createTextMessage({required String text}) async {
    return V2TimValueCallback<V2TimMsgCreateInfoResult>()..code = 0..data = (V2TimMsgCreateInfoResult()..id = 'dummy');
  }
  
  Future<V2TimValueCallback<V2TimMsgCreateInfoResult>> createSoundMessage({required String soundPath, required int duration}) async {
    return V2TimValueCallback<V2TimMsgCreateInfoResult>()..code = 0..data = (V2TimMsgCreateInfoResult()..id = 'dummy');
  }

  Future<V2TimValueCallback<V2TimMsgCreateInfoResult>> createImageMessage({required String imagePath}) async {
     return V2TimValueCallback<V2TimMsgCreateInfoResult>()..code = 0..data = (V2TimMsgCreateInfoResult()..id = 'dummy');
  }

  Future<V2TimValueCallback<V2TimMsgCreateInfoResult>> createCustomMessage({required String data, String? desc}) async {
     return V2TimValueCallback<V2TimMsgCreateInfoResult>()..code = 0..data = (V2TimMsgCreateInfoResult()..id = 'dummy');
  }

  Future<V2TimValueCallback<V2TimMessageOnlineUrl>> getMessageOnlineUrl({required String msgID}) async {
    return V2TimValueCallback<V2TimMessageOnlineUrl>()..code = 0..data = V2TimMessageOnlineUrl();
  }

  Future<V2TimValueCallback<V2TimMessage>> sendMessage({
    required String id,
    required String receiver,
    required String groupID,
    int? priority,
    bool? onlineUserOnly,
    bool? needReadReceipt,
    bool? isSupportMessageExtension,
    OfflinePushInfo? offlinePushInfo,
  }) async {
    return V2TimValueCallback<V2TimMessage>()..code = 0..data = V2TimMessage();
  }

  Future<V2TimValueCallback<V2TimMessageListResult>> getHistoryMessageList({
    required int count,
    String? userID,
    String? groupID,
    int? getType,
    String? lastMsgID,
  }) async {
    return V2TimValueCallback<V2TimMessageListResult>()..code = 0..data = V2TimMessageListResult();
  }

  Future<V2TimValueCallback<V2TimMessageListResult>> getHistoryMessageListV2({
    required int count,
    String? userID,
    String? groupID,
    int? getType,
    String? lastMsgID,
  }) async {
    return V2TimValueCallback<V2TimMessageListResult>()..code = 0..data = V2TimMessageListResult();
  }
  
  Future<void> addAdvancedMsgListener({required dynamic listener}) async {}
  Future<void> removeAdvancedMsgListener({required dynamic listener}) async {}
  
  Future<V2TimCallback> deleteMessages({required List<String> msgIDs}) async {
    return V2TimCallback()..code = 0;
  }
}

class _DynamicMock {
  dynamic noSuchMethod(Invocation invocation) => _DynamicMock();
}

class LogLevelEnum {
  static const int V2TIM_LOG_ALL = 0;
}
