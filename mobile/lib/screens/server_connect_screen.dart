// First-run screen — captures server URL, join code, and display name.
// Successful registration persists the session and hands control to
// the main app via `onConnected`.

import 'package:flutter/material.dart';

import '../services/api_client.dart';
import '../services/server_connection.dart';

class ServerConnectScreen extends StatefulWidget {
  const ServerConnectScreen({
    super.key,
    required this.connection,
    required this.onConnected,
  });

  final ServerConnection connection;
  final VoidCallback onConnected;

  @override
  State<ServerConnectScreen> createState() => _ServerConnectScreenState();
}

class _ServerConnectScreenState extends State<ServerConnectScreen> {
  final _urlCtl = TextEditingController(text: 'http://10.0.2.2:8080');
  final _codeCtl = TextEditingController();
  final _nameCtl = TextEditingController();
  bool _busy = false;
  String? _error;

  @override
  void dispose() {
    _urlCtl.dispose();
    _codeCtl.dispose();
    _nameCtl.dispose();
    super.dispose();
  }

  Future<void> _submit() async {
    final url = _urlCtl.text.trim();
    final code = _codeCtl.text.trim();
    final name = _nameCtl.text.trim();
    if (url.isEmpty || code.isEmpty || name.isEmpty) {
      setState(() => _error = 'All three fields are required.');
      return;
    }
    setState(() {
      _busy = true;
      _error = null;
    });
    try {
      await widget.connection.connect(
        serverUrl: url,
        joinCode: code,
        displayName: name,
      );
      if (!mounted) return;
      widget.onConnected();
    } on ApiException catch (e) {
      setState(() => _error = '${e.code}: ${e.message}');
    } catch (e) {
      setState(() => _error = 'Connection failed: $e');
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Center(
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 480),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                Text(
                  'Connect to a server',
                  style: Theme.of(context).textTheme.headlineSmall,
                  textAlign: TextAlign.center,
                ),
                const SizedBox(height: 8),
                Text(
                  'Enter the URL and join code your DM shared, plus a display name to identify yourself on the server.',
                  style: Theme.of(context).textTheme.bodyMedium,
                  textAlign: TextAlign.center,
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
                  controller: _codeCtl,
                  decoration: const InputDecoration(
                    labelText: 'Join code',
                    hintText: 'e.g. abc123-def456-ghi789',
                    border: OutlineInputBorder(),
                  ),
                  autocorrect: false,
                ),
                const SizedBox(height: 16),
                TextField(
                  controller: _nameCtl,
                  decoration: const InputDecoration(
                    labelText: 'Display name',
                    hintText: 'How others see you on this server',
                    border: OutlineInputBorder(),
                  ),
                  autocorrect: false,
                ),
                const SizedBox(height: 24),
                if (_error != null)
                  Padding(
                    padding: const EdgeInsets.only(bottom: 12),
                    child: Text(
                      _error!,
                      style: TextStyle(
                          color: Theme.of(context).colorScheme.error),
                      textAlign: TextAlign.center,
                    ),
                  ),
                FilledButton(
                  onPressed: _busy ? null : _submit,
                  child: _busy
                      ? const SizedBox(
                          width: 18,
                          height: 18,
                          child:
                              CircularProgressIndicator(strokeWidth: 2),
                        )
                      : const Text('Connect'),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
