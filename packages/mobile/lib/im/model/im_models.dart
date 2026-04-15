import 'dart:convert';
import 'package:wechat_flutter/im/model/x_message.dart';

/// 自研会话模型，替代 V2TimConversation
class XConversation {
  final String conversationId;
  final int type; // 1: 单聊, 2: 群聊
  final String? peerId; // 单聊对方 userId / 群聊 groupId
  final String? showName;
  final String? faceUrl;
  final int unreadCount;
  final XMessage? lastMessage;
  final String? draftText;
  final bool isPinned;

  XConversation({
    required this.conversationId,
    required this.type,
    this.peerId,
    this.showName,
    this.faceUrl,
    this.unreadCount = 0,
    this.lastMessage,
    this.draftText,
    this.isPinned = false,
  });

  factory XConversation.fromJson(Map<String, dynamic> json) {
    return XConversation(
      conversationId: json['conversation_id']?.toString() ?? '',
      type: json['type'] ?? 1,
      peerId: json['peer_id']?.toString(),
      showName: json['show_name']?.toString(),
      faceUrl: json['face_url']?.toString(),
      unreadCount: json['unread_count'] ?? 0,
      lastMessage: json['last_message'] != null
          ? XMessage.fromJson(json['last_message'])
          : null,
      draftText: json['draft_text']?.toString(),
      isPinned: json['is_pinned'] ?? false,
    );
  }
}

/// 自研用户资料模型，替代 V2TimUserFullInfo
class XUserInfo {
  String userId;
  String? nickName;
  String? faceUrl;
  String? selfSignature;
  int? gender;

  XUserInfo({
    required this.userId,
    this.nickName,
    this.faceUrl,
    this.selfSignature,
    this.gender,
  });

  factory XUserInfo.fromJson(Map<String, dynamic> json) {
    return XUserInfo(
      userId: json['user_id']?.toString() ?? '',
      nickName: json['nick_name']?.toString(),
      faceUrl: json['face_url']?.toString(),
      selfSignature: json['self_signature']?.toString(),
      gender: json['gender'],
    );
  }
}

/// 自研好友模型，替代 V2TimFriendInfo
class XFriendInfo {
  String userId;
  String? friendRemark;
  String? friendAddSource;
  XUserInfo? userProfile;

  XFriendInfo({
    required this.userId,
    this.friendRemark,
    this.friendAddSource,
    this.userProfile,
  });

  factory XFriendInfo.fromJson(Map<String, dynamic> json) {
    return XFriendInfo(
      userId: json['user_id']?.toString() ?? '',
      friendRemark: json['friend_remark']?.toString(),
      friendAddSource: json['friend_add_source']?.toString(),
      userProfile: json['user_profile'] != null
          ? XUserInfo.fromJson(json['user_profile'])
          : null,
    );
  }
}

/// 自研好友申请模型，替代 V2TimFriendApplication
class XFriendApplication {
  String? userId;
  String? nickName;
  String? faceUrl;
  String? addWording;
  int? type;

  XFriendApplication({
    this.userId,
    this.nickName,
    this.faceUrl,
    this.addWording,
    this.type,
  });
}

/// 自研群组信息模型，替代 V2TimGroupInfo
class XGroupInfo {
  int? memberCount;
  String? owner;
  String? introduction;
  String? groupName;
  String? notification;
  String? groupId;
  String? faceUrl;

  XGroupInfo({
    this.memberCount,
    this.owner,
    this.introduction,
    this.groupName,
    this.notification,
    this.groupId,
    this.faceUrl,
  });

  factory XGroupInfo.fromJson(Map<String, dynamic> json) {
    return XGroupInfo(
      memberCount: json['member_count'],
      owner: json['owner']?.toString(),
      introduction: json['introduction']?.toString(),
      groupName: json['group_name']?.toString(),
      notification: json['notification']?.toString(),
      groupId: json['group_id']?.toString(),
      faceUrl: json['face_url']?.toString(),
    );
  }
}

/// 自研群成员模型，替代 V2TimGroupMemberFullInfo
class XGroupMember {
  String userId;
  String? nickName;
  String? faceUrl;
  int? role;

  XGroupMember({
    required this.userId,
    this.nickName,
    this.faceUrl,
    this.role,
  });
}

/// 会话类型常量
class ConversationType {
  static const int c2c = 1;
  static const int group = 2;
}
