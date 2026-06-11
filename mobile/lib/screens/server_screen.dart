// Optional server connection — mirrors the web app's auth interface:
// log in (username + password, the prominent action), register
// (username, email, password + confirmation with visibility toggles,
// join code), and log out (revokes the session). The app works fully
// offline; connecting unlocks modules, publishing, and push/pull.

import 'package:flutter/material.dart';

import '../services/api_client.dart';
import '../services/server_connection.dart';

class ServerScreen extends StatefulWidget {
  const ServerScreen({super.key, required this.connection});

  final ServerConnection connection;

  @override
  State<ServerScreen> createState() => _ServerScreenState();
}

enum _Mode { login, register }

class _ServerScreenState extends State<ServerScreen> {
  _Mode _mode = _Mode.login;
  bool _busy = false;
  String? _error;

  late final TextEditingController _urlCtl;
  final _usernameCtl = TextEditingController();
  final _emailCtl = TextEditingController();
  final _passwordCtl = TextEditingController();
  final _confirmCtl = TextEditingController();
  final _joinCodeCtl = TextEditingController();
  bool _passwordVisible = false;
  bool _confirmVisible = false;

  @override
  void initState() {
    super.initState();
    _urlCtl = TextEditingController(
      text: widget.connection.serverUrl ?? 'http://10.0.2.2:8080',
    );
    widget.connection.addListener(_onConnectionChanged);
  }

  @override
  void dispose() {
    widget.connection.removeListener(_onConnectionChanged);
    _urlCtl.dispose();
    _usernameCtl.dispose();
    _emailCtl.dispose();
    _passwordCtl.dispose();
    _confirmCtl.dispose();
    _joinCodeCtl.dispose();
    super.dispose();
  }

  void _onConnectionChanged() {
    if (mounted) setState(() {});
  }

  Future<void> _submit() async {
    final url = _urlCtl.text.trim();
    final username = _usernameCtl.text.trim();
    final password = _passwordCtl.text;
    if (url.isEmpty || username.isEmpty || password.isEmpty) {
      setState(() => _error = 'Server URL, username, and password are required.');
      return;
    }
    if (_mode == _Mode.register) {
      if (_emailCtl.text.trim().isEmpty || _joinCodeCtl.text.trim().isEmpty) {
        setState(() => _error = 'Email and join code are required to register.');
        return;
      }
      if (password != _confirmCtl.text) {
        setState(() => _error = 'Passwords do not match.');
        return;
      }
      if (password.length < 8) {
        setState(() => _error = 'Password must be at least 8 characters.');
        return;
      }
    }

    setState(() {
      _busy = true;
      _error = null;
    });
    try {
      if (_mode == _Mode.login) {
        await widget.connection.login(
          serverUrl: url,
          username: username,
          password: password,
        );
      } else {
        await widget.connection.register(
          serverUrl: url,
          joinCode: _joinCodeCtl.text.trim(),
          username: username,
          email: _emailCtl.text.trim(),
          password: password,
        );
      }
      if (!mounted) return;
      _passwordCtl.clear();
      _confirmCtl.clear();
    } on ApiException catch (e) {
      setState(() => _error = e.message);
    } catch (e) {
      setState(() => _error = 'Connection failed: $e');
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  Future<void> _logout() async {
    setState(() => _busy = true);
    try {
      await widget.connection.logout();
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final connection = widget.connection;
    return Scaffold(
      appBar: AppBar(title: const Text('Server')),
      body: SafeArea(
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(24),
          child: Center(
            child: ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 480),
              child: connection.isLoggedIn
                  ? _loggedInView(connection)
                  : _authForm(),
            ),
          ),
        ),
      ),
    );
  }

  Widget _loggedInView(ServerConnection connection) {
    final user = connection.user;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Icon(Icons.cloud_done_outlined,
            size: 48, color: Theme.of(context).colorScheme.primary),
        const SizedBox(height: 16),
        Text(
          user != null ? 'Logged in as ${user.username}' : 'Logged in',
          style: Theme.of(context).textTheme.titleLarge,
          textAlign: TextAlign.center,
        ),
        if (user != null)
          Padding(
            padding: const EdgeInsets.only(top: 4),
            child: Text(
              user.email + (user.admin ? ' · admin' : ''),
              style: Theme.of(context).textTheme.bodyMedium,
              textAlign: TextAlign.center,
            ),
          ),
        if (connection.serverUrl != null)
          Padding(
            padding: const EdgeInsets.only(top: 4),
            child: Text(
              connection.serverUrl!,
              style: Theme.of(context).textTheme.bodySmall,
              textAlign: TextAlign.center,
            ),
          ),
        const SizedBox(height: 24),
        Text(
          'Your content stays on this device. Use Push / Pull on a setting '
          'to sync it with this server, or browse its published modules.',
          style: Theme.of(context).textTheme.bodyMedium,
          textAlign: TextAlign.center,
        ),
        const SizedBox(height: 24),
        FilledButton.icon(
          onPressed: _busy ? null : _logout,
          icon: const Icon(Icons.logout),
          label: const Text('Log out'),
        ),
      ],
    );
  }

  Widget _authForm() {
    final isRegister = _mode == _Mode.register;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Text(
          isRegister ? 'Register on a server' : 'Log in to a server',
          style: Theme.of(context).textTheme.headlineSmall,
          textAlign: TextAlign.center,
        ),
        const SizedBox(height: 8),
        Text(
          'Optional — everything you author lives on this device. '
          'A server connection adds module browsing, publishing, and sync.',
          style: Theme.of(context).textTheme.bodyMedium,
          textAlign: TextAlign.center,
        ),
        const SizedBox(height: 24),
        SegmentedButton<_Mode>(
          segments: const [
            ButtonSegment(value: _Mode.login, label: Text('Log in')),
            ButtonSegment(value: _Mode.register, label: Text('Register')),
          ],
          selected: {_mode},
          onSelectionChanged: (s) => setState(() {
            _mode = s.first;
            _error = null;
          }),
        ),
        const SizedBox(height: 24),
        TextField(
          controller: _urlCtl,
          decoration: const InputDecoration(
            labelText: 'Server URL',
            hintText: 'http://example.com:8080',
            border: OutlineInputBorder(),
          ),
          keyboardType: TextInputType.url,
          autocorrect: false,
        ),
        const SizedBox(height: 16),
        TextField(
          controller: _usernameCtl,
          decoration: const InputDecoration(
            labelText: 'Username',
            border: OutlineInputBorder(),
          ),
          autocorrect: false,
        ),
        if (isRegister) ...[
          const SizedBox(height: 16),
          TextField(
            controller: _emailCtl,
            decoration: const InputDecoration(
              labelText: 'Email',
              border: OutlineInputBorder(),
            ),
            keyboardType: TextInputType.emailAddress,
            autocorrect: false,
          ),
        ],
        const SizedBox(height: 16),
        TextField(
          controller: _passwordCtl,
          obscureText: !_passwordVisible,
          decoration: InputDecoration(
            labelText: 'Password',
            border: const OutlineInputBorder(),
            suffixIcon: IconButton(
              icon: Icon(_passwordVisible
                  ? Icons.visibility_off_outlined
                  : Icons.visibility_outlined),
              tooltip: _passwordVisible ? 'Hide password' : 'Show password',
              onPressed: () =>
                  setState(() => _passwordVisible = !_passwordVisible),
            ),
          ),
          autocorrect: false,
        ),
        if (isRegister) ...[
          const SizedBox(height: 16),
          TextField(
            controller: _confirmCtl,
            obscureText: !_confirmVisible,
            decoration: InputDecoration(
              labelText: 'Confirm password',
              border: const OutlineInputBorder(),
              suffixIcon: IconButton(
                icon: Icon(_confirmVisible
                    ? Icons.visibility_off_outlined
                    : Icons.visibility_outlined),
                tooltip: _confirmVisible ? 'Hide password' : 'Show password',
                onPressed: () =>
                    setState(() => _confirmVisible = !_confirmVisible),
              ),
            ),
            autocorrect: false,
          ),
          const SizedBox(height: 16),
          TextField(
            controller: _joinCodeCtl,
            decoration: const InputDecoration(
              labelText: 'Server join code',
              hintText: 'e.g. Abc123-DEf456-Ghi789',
              border: OutlineInputBorder(),
            ),
            autocorrect: false,
          ),
        ],
        const SizedBox(height: 24),
        if (_error != null)
          Padding(
            padding: const EdgeInsets.only(bottom: 12),
            child: Text(
              _error!,
              style: TextStyle(color: Theme.of(context).colorScheme.error),
              textAlign: TextAlign.center,
            ),
          ),
        FilledButton(
          onPressed: _busy ? null : _submit,
          child: _busy
              ? const SizedBox(
                  width: 18,
                  height: 18,
                  child: CircularProgressIndicator(strokeWidth: 2),
                )
              : Text(isRegister ? 'Register' : 'Log in'),
        ),
      ],
    );
  }
}
