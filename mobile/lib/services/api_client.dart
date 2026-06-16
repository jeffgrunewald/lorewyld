// HTTP client for the lorewyld server's JSON API.
//
// All v1 endpoints land here. Authenticated endpoints attach the
// session token via the standard `Authorization: Bearer <token>` header.
// Failures throw `ApiException` carrying the server's error code +
// message; UI layers translate those into friendly strings.

import 'dart:convert';

import 'package:http/http.dart' as http;

import '../types/api_v1.dart';
import '../types/lore_note.dart';
import '../types/setting.dart';
import '../types/tag.dart';
import '../types/user.dart';

class ApiException implements Exception {
  final int statusCode;
  final String code;
  final String message;

  ApiException(this.statusCode, this.code, this.message);

  @override
  String toString() => 'ApiException($statusCode, $code): $message';
}

class ApiClient {
  final Uri baseUri;
  final http.Client _http;
  String? _sessionToken;

  ApiClient({required this.baseUri, http.Client? httpClient})
    : _http = httpClient ?? http.Client();

  void setSessionToken(String? token) {
    _sessionToken = token;
  }

  String? get sessionToken => _sessionToken;

  Map<String, String> _headers({bool json = false, bool auth = true}) {
    final h = <String, String>{};
    if (json) h['Content-Type'] = 'application/json';
    if (auth && _sessionToken != null) {
      h['Authorization'] = 'Bearer $_sessionToken';
    }
    return h;
  }

  Uri _u(String path, [Map<String, String>? query]) {
    // Append to the base URI's path rather than replacing it, so a
    // server URL with a prefix (http://host/lorewyld) keeps working.
    final basePath = baseUri.path.endsWith('/')
        ? baseUri.path.substring(0, baseUri.path.length - 1)
        : baseUri.path;
    return baseUri.replace(path: '$basePath$path', queryParameters: query);
  }

  Future<dynamic> _send(Future<http.Response> Function() send) async {
    final res = await send();
    if (res.statusCode >= 200 && res.statusCode < 300) {
      if (res.body.isEmpty) return null;
      try {
        return jsonDecode(res.body);
      } on FormatException catch (e) {
        // A non-JSON 2xx (proxy page, captive portal) should surface as
        // an ApiException like every other failure, not a raw parse error.
        throw ApiException(
          res.statusCode,
          'parse_error',
          'Server returned a malformed response: ${e.message}',
        );
      }
    }
    String code = 'http_${res.statusCode}';
    String message = res.body;
    try {
      final err = jsonDecode(res.body) as Map<String, dynamic>;
      code = err['code'] as String? ?? code;
      message = err['message'] as String? ?? message;
    } catch (_) {
      // Body wasn't JSON — leave the raw body as the message.
    }
    throw ApiException(res.statusCode, code, message);
  }

  // ── auth ────────────────────────────────────────────────────────────

  Future<AuthResponse> register({
    required String joinCode,
    required String username,
    required String email,
    required String password,
  }) async {
    final body = await _send(
      () => _http.post(
        _u('/api/users/register'),
        headers: _headers(json: true, auth: false),
        body: jsonEncode({
          'join_code': joinCode,
          'username': username,
          'email': email,
          'password': password,
        }),
      ),
    );
    return AuthResponse.fromJson(body as Map<String, dynamic>);
  }

  Future<AuthResponse> login({
    required String username,
    required String password,
  }) async {
    final body = await _send(
      () => _http.post(
        _u('/api/users/login'),
        headers: _headers(json: true, auth: false),
        body: jsonEncode({'username': username, 'password': password}),
      ),
    );
    return AuthResponse.fromJson(body as Map<String, dynamic>);
  }

  /// Revokes the current session server-side. Idempotent on the server;
  /// callers should clear local session state regardless of outcome.
  Future<void> logout() async {
    await _send(() => _http.post(_u('/api/users/logout'), headers: _headers()));
  }

  /// Resolves the current session token to its user — used to validate
  /// a persisted session on app launch.
  Future<User> me() async {
    final body = await _send(
      () => _http.get(_u('/api/users/me'), headers: _headers()),
    );
    return User.fromJson(body as Map<String, dynamic>);
  }

  // ── server info ─────────────────────────────────────────────────────

  Future<ServerInfo> serverInfo() async {
    final body = await _send(
      () => _http.get(_u('/api/server-info'), headers: _headers(auth: false)),
    );
    return ServerInfo.fromJson(body as Map<String, dynamic>);
  }

  // ── tags ────────────────────────────────────────────────────────────

  Future<List<Tag>> listTags({String? query, int limit = 50}) async {
    final qs = <String, String>{'limit': '$limit'};
    if (query != null && query.isNotEmpty) qs['q'] = query;
    final body = await _send(
      () => _http.get(_u('/api/tags', qs), headers: _headers()),
    );
    return (body as List<dynamic>)
        .map((e) => Tag.fromJson(e as Map<String, dynamic>))
        .toList();
  }

  // ── settings ────────────────────────────────────────────────────────

  Future<List<Setting>> listSettings() async {
    final body = await _send(
      () => _http.get(_u('/api/settings'), headers: _headers()),
    );
    return (body as List<dynamic>)
        .map((e) => Setting.fromJson(e as Map<String, dynamic>))
        .toList();
  }

  Future<Setting> createSetting({required String name}) async {
    final body = await _send(
      () => _http.post(
        _u('/api/settings'),
        headers: _headers(json: true),
        body: jsonEncode({'name': name}),
      ),
    );
    return Setting.fromJson(body as Map<String, dynamic>);
  }

  Future<Setting> updateSetting({required String uuid, String? name}) async {
    final body = await _send(
      () => _http.patch(
        _u('/api/settings/$uuid'),
        headers: _headers(json: true),
        body: jsonEncode({if (name != null) 'name': name}),
      ),
    );
    return Setting.fromJson(body as Map<String, dynamic>);
  }

  Future<void> deleteSetting(String uuid) async {
    await _send(
      () => _http.delete(_u('/api/settings/$uuid'), headers: _headers()),
    );
  }

  // ── lore notes ──────────────────────────────────────────────────────

  Future<List<LoreNoteWithTags>> listLoreNotes({
    NoteScopeKind? scopeKind,
    String? scopeTarget,
    String? tag,
    int limit = 100,
  }) async {
    final qs = <String, String>{'limit': '$limit'};
    if (scopeKind != null) qs['scope_kind'] = scopeKind.wire;
    if (scopeTarget != null) qs['scope_target'] = scopeTarget;
    if (tag != null) qs['tag'] = tag;
    final body = await _send(
      () => _http.get(_u('/api/lore-notes', qs), headers: _headers()),
    );
    return (body as List<dynamic>)
        .map((e) => LoreNoteWithTags.fromJson(e as Map<String, dynamic>))
        .toList();
  }

  Future<LoreNoteWithTags> getLoreNote(String uuid) async {
    final body = await _send(
      () => _http.get(_u('/api/lore-notes/$uuid'), headers: _headers()),
    );
    return LoreNoteWithTags.fromJson(body as Map<String, dynamic>);
  }

  Future<LoreNoteWithTags> createLoreNote({
    required String title,
    required String bodyMarkdown,
    required NoteScope scope,
    NoteVisibility visibility = NoteVisibility.visible,
    List<String> tagSlugs = const [],
  }) async {
    final body = await _send(
      () => _http.post(
        _u('/api/lore-notes'),
        headers: _headers(json: true),
        body: jsonEncode({
          'title': title,
          'body_markdown': bodyMarkdown,
          'scope': scope.toJson(),
          'visibility': visibility.wire,
          'tag_slugs': tagSlugs,
        }),
      ),
    );
    return LoreNoteWithTags.fromJson(body as Map<String, dynamic>);
  }

  Future<LoreNoteWithTags> updateLoreNote({
    required String uuid,
    String? title,
    String? bodyMarkdown,
    NoteVisibility? visibility,
    List<String>? tagSlugs,
  }) async {
    final payload = <String, dynamic>{};
    if (title != null) payload['title'] = title;
    if (bodyMarkdown != null) payload['body_markdown'] = bodyMarkdown;
    if (visibility != null) payload['visibility'] = visibility.wire;
    if (tagSlugs != null) payload['tag_slugs'] = tagSlugs;
    final body = await _send(
      () => _http.patch(
        _u('/api/lore-notes/$uuid'),
        headers: _headers(json: true),
        body: jsonEncode(payload),
      ),
    );
    return LoreNoteWithTags.fromJson(body as Map<String, dynamic>);
  }

  Future<void> deleteLoreNote(String uuid) async {
    await _send(
      () => _http.delete(_u('/api/lore-notes/$uuid'), headers: _headers()),
    );
  }

  // ── search ──────────────────────────────────────────────────────────

  Future<SearchResponse> search({
    String? q,
    NoteScopeKind? scopeKind,
    String? scopeTargetUuid,
    List<String> tagSlugs = const [],
    int limit = 50,
  }) async {
    final payload = <String, dynamic>{
      if (q != null && q.isNotEmpty) 'q': q,
      if (scopeKind != null) 'scope_kind': scopeKind.wire,
      if (scopeTargetUuid != null) 'scope_target_uuid': scopeTargetUuid,
      'tag_slugs': tagSlugs,
      'limit': limit,
    };
    final body = await _send(
      () => _http.post(
        _u('/api/search'),
        headers: _headers(json: true),
        body: jsonEncode(payload),
      ),
    );
    return SearchResponse.fromJson(body as Map<String, dynamic>);
  }

  // ── modules / Promote-to-Module ─────────────────────────────────────

  Future<dynamic> publishModule({
    required String sourceSettingUuid,
    required String name,
    required String slug,
    required String license,
    String? description,
    List<String> authors = const [],
    required String versionString,
    required List<String> selectedNoteUuids,
  }) async {
    return await _send(
      () => _http.post(
        _u('/api/modules'),
        headers: _headers(json: true),
        body: jsonEncode({
          'source_setting_uuid': sourceSettingUuid,
          'name': name,
          'slug': slug,
          'license': license,
          if (description != null) 'description': description,
          'authors': authors,
          'version_string': versionString,
          'selected_note_uuids': selectedNoteUuids,
        }),
      ),
    );
  }
}
