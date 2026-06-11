// Manages the OPTIONAL server connection: base URL + session token +
// cached user identity. The app is fully usable offline; connecting
// only unlocks server features (modules, publish, push/pull). Persists
// across launches via SharedPreferences.

import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../types/user.dart';
import 'api_client.dart';

class ServerConnection extends ChangeNotifier {
  static const _kServerUrl = 'lw.server_url';
  static const _kSessionToken = 'lw.session_token';
  static const _kUserJson = 'lw.user_json';

  // Pre-unification session keys — cleared on load; those sessions are
  // invalid against the new server anyway.
  static const _kLegacyUserUuid = 'lw.user_uuid';
  static const _kLegacyDisplayName = 'lw.display_name';
  static const _kLegacyServerUuid = 'lw.server_uuid';

  String? _serverUrl;
  ApiClient? _api;
  User? _user;

  String? get serverUrl => _serverUrl;
  ApiClient? get api => _api;
  User? get user => _user;
  bool get isLoggedIn => _api != null && _api!.sessionToken != null;

  Future<void> load() async {
    final prefs = await SharedPreferences.getInstance();

    if (prefs.containsKey(_kLegacyDisplayName)) {
      await prefs.remove(_kLegacyUserUuid);
      await prefs.remove(_kLegacyDisplayName);
      await prefs.remove(_kLegacyServerUuid);
      await prefs.remove(_kSessionToken);
    }

    final url = prefs.getString(_kServerUrl);
    final token = prefs.getString(_kSessionToken);
    final userJson = prefs.getString(_kUserJson);

    if (url == null) return;
    _serverUrl = url;
    _api = ApiClient(baseUri: Uri.parse(url))..setSessionToken(token);
    if (userJson != null) {
      try {
        _user = User.fromJson(jsonDecode(userJson) as Map<String, dynamic>);
      } catch (_) {
        _user = null;
      }
    }
    notifyListeners();

    if (token != null) {
      // Fire-and-forget probe: a 401 means the session was revoked
      // server-side, so drop it. Network errors are ignored — the
      // device may simply be offline, and the token may still be good.
      _probeSession();
    }
  }

  Future<void> _probeSession() async {
    final api = _api;
    if (api == null) return;
    try {
      final me = await api.me();
      await _storeUser(me);
      _user = me;
      notifyListeners();
    } on ApiException catch (e) {
      if (e.statusCode == 401) {
        await _clearSession();
      }
    } catch (_) {
      // Offline / unreachable — keep the persisted session.
    }
  }

  Future<void> login({
    required String serverUrl,
    required String username,
    required String password,
  }) async {
    final api = ApiClient(baseUri: Uri.parse(serverUrl));
    final auth = await api.login(username: username, password: password);
    await _adoptSession(serverUrl, api, auth.sessionToken, auth.user);
  }

  Future<void> register({
    required String serverUrl,
    required String joinCode,
    required String username,
    required String email,
    required String password,
  }) async {
    final api = ApiClient(baseUri: Uri.parse(serverUrl));
    final auth = await api.register(
      joinCode: joinCode,
      username: username,
      email: email,
      password: password,
    );
    await _adoptSession(serverUrl, api, auth.sessionToken, auth.user);
  }

  /// Revokes the session server-side (best-effort) and discards it
  /// locally. The server URL is kept so the next login pre-fills it.
  Future<void> logout() async {
    final api = _api;
    if (api != null && api.sessionToken != null) {
      try {
        await api.logout();
      } catch (_) {
        // Revocation is best-effort; the local session goes regardless.
      }
    }
    await _clearSession();
  }

  Future<void> _adoptSession(
    String serverUrl,
    ApiClient api,
    String token,
    User user,
  ) async {
    api.setSessionToken(token);
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(_kServerUrl, serverUrl);
    await prefs.setString(_kSessionToken, token);
    _serverUrl = serverUrl;
    _api = api;
    _user = user;
    await _storeUser(user);
    notifyListeners();
  }

  Future<void> _storeUser(User user) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(_kUserJson, jsonEncode(user.toJson()));
  }

  Future<void> _clearSession() async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.remove(_kSessionToken);
    await prefs.remove(_kUserJson);
    _api?.setSessionToken(null);
    _user = null;
    notifyListeners();
  }
}
