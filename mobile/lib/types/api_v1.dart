// Wire types for the v1 HTTP API. Mirror of
// `lorewyld_types::api_v1::*`.

import 'app_user.dart';
import 'content_module.dart';
import 'lore_note.dart';

class AuthResponse {
  final AppUser user;
  final String sessionToken;

  const AuthResponse({required this.user, required this.sessionToken});

  factory AuthResponse.fromJson(Map<String, dynamic> json) => AuthResponse(
        user: AppUser.fromJson(json['user'] as Map<String, dynamic>),
        sessionToken: json['session_token'] as String,
      );
}

class GameServerSummary {
  final String uuid;
  final String name;
  final String version;

  const GameServerSummary({
    required this.uuid,
    required this.name,
    required this.version,
  });

  factory GameServerSummary.fromJson(Map<String, dynamic> json) =>
      GameServerSummary(
        uuid: json['uuid'] as String,
        name: json['name'] as String,
        version: json['version'] as String,
      );
}

class ServerInfo {
  final GameServerSummary server;
  final List<ContentModule> modules;

  const ServerInfo({required this.server, required this.modules});

  factory ServerInfo.fromJson(Map<String, dynamic> json) => ServerInfo(
        server: GameServerSummary.fromJson(
            json['server'] as Map<String, dynamic>),
        modules: (json['modules'] as List<dynamic>? ?? const [])
            .map((e) => ContentModule.fromJson(e as Map<String, dynamic>))
            .toList(),
      );
}

class SearchResponse {
  final List<LoreNoteWithTags> notes;

  const SearchResponse({required this.notes});

  factory SearchResponse.fromJson(Map<String, dynamic> json) => SearchResponse(
        notes: (json['notes'] as List<dynamic>? ?? const [])
            .map((e) => LoreNoteWithTags.fromJson(e as Map<String, dynamic>))
            .toList(),
      );
}
