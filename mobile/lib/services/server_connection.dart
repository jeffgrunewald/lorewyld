// Manages the user's current server connection: base URL + session
// token + cached user identity. Persists across launches via
// SharedPreferences so the user only sees the connect screen once.

import 'package:flutter/foundation.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../types/app_user.dart';
import 'api_client.dart';

class ServerConnection extends ChangeNotifier {
  static const _kServerUrl = 'lw.server_url';
  static const _kSessionToken = 'lw.session_token';
  static const _kUserUuid = 'lw.user_uuid';
  static const _kDisplayName = 'lw.display_name';
  static const _kServerUuid = 'lw.server_uuid';

  String? _serverUrl;
  ApiClient? _api;
  AppUser? _user;

  String? get serverUrl => _serverUrl;
  ApiClient? get api => _api;
  AppUser? get user => _user;
  bool get isConnected => _api != null && _api!.sessionToken != null;

  Future<void> load() async {
    final prefs = await SharedPreferences.getInstance();
    final url = prefs.getString(_kServerUrl);
    final token = prefs.getString(_kSessionToken);
    final userUuid = prefs.getString(_kUserUuid);
    final displayName = prefs.getString(_kDisplayName);
    final serverUuid = prefs.getString(_kServerUuid);

    if (url == null) return;
    _serverUrl = url;
    _api = ApiClient(baseUri: Uri.parse(url))..setSessionToken(token);
    if (userUuid != null && displayName != null && serverUuid != null) {
      _user = AppUser(
        uuid: userUuid,
        serverUuid: serverUuid,
        displayName: displayName,
        // Best-effort restore — exact creation timestamp isn't critical
        // for UI; refreshed on next login.
        createdAt: DateTime.now(),
      );
    }
    notifyListeners();
  }

  Future<void> connect({
    required String serverUrl,
    required String joinCode,
    required String displayName,
  }) async {
    final api = ApiClient(baseUri: Uri.parse(serverUrl));
    final auth = await api.register(
      joinCode: joinCode,
      displayName: displayName,
    );
    api.setSessionToken(auth.sessionToken);

    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(_kServerUrl, serverUrl);
    await prefs.setString(_kSessionToken, auth.sessionToken);
    await prefs.setString(_kUserUuid, auth.user.uuid);
    await prefs.setString(_kDisplayName, auth.user.displayName);
    await prefs.setString(_kServerUuid, auth.user.serverUuid);

    _serverUrl = serverUrl;
    _api = api;
    _user = auth.user;
    notifyListeners();
  }

  Future<void> reLogin({required String displayName}) async {
    final api = _api;
    if (api == null) {
      throw StateError('no server URL configured; connect() first');
    }
    final auth = await api.login(displayName: displayName);
    api.setSessionToken(auth.sessionToken);
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(_kSessionToken, auth.sessionToken);
    await prefs.setString(_kUserUuid, auth.user.uuid);
    await prefs.setString(_kDisplayName, auth.user.displayName);
    await prefs.setString(_kServerUuid, auth.user.serverUuid);
    _user = auth.user;
    notifyListeners();
  }

  Future<void> disconnect() async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.remove(_kServerUrl);
    await prefs.remove(_kSessionToken);
    await prefs.remove(_kUserUuid);
    await prefs.remove(_kDisplayName);
    await prefs.remove(_kServerUuid);
    _serverUrl = null;
    _api = null;
    _user = null;
    notifyListeners();
  }
}
