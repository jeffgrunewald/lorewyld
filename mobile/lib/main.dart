import 'package:flutter/material.dart';

import 'services/content_store.dart';
import 'services/local_store.dart';
import 'services/server_connection.dart';
import 'widgets/lorewyld_shell.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  final connection = ServerConnection();
  final store = await LocalStore.open();
  await connection.load();
  runApp(LorewyldApp(connection: connection, store: store));
}

class LorewyldApp extends StatelessWidget {
  const LorewyldApp({
    super.key,
    required this.connection,
    required this.store,
  });

  final ServerConnection connection;
  final LocalStore store;

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Lorewyld',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.deepPurple),
        useMaterial3: true,
      ),
      home: ContentSeedGate(
        contentStore: ContentStore(store),
        child: LorewyldShell(connection: connection, store: store),
      ),
    );
  }
}

/// Blocks the first launch on the SRD content import, showing progress;
/// every later launch passes straight through to [child].
class ContentSeedGate extends StatefulWidget {
  const ContentSeedGate({
    super.key,
    required this.contentStore,
    required this.child,
  });

  final ContentStore contentStore;
  final Widget child;

  @override
  State<ContentSeedGate> createState() => _ContentSeedGateState();
}

class _ContentSeedGateState extends State<ContentSeedGate> {
  late final Future<void> _seedFuture;
  double _progress = 0;

  @override
  void initState() {
    super.initState();
    _seedFuture = widget.contentStore.importBundle(
      onProgress: (p) {
        if (mounted) setState(() => _progress = p);
      },
    );
  }

  @override
  Widget build(BuildContext context) {
    return FutureBuilder<void>(
      future: _seedFuture,
      builder: (context, snapshot) {
        if (snapshot.connectionState == ConnectionState.done) {
          if (snapshot.hasError) {
            return Scaffold(
              body: Center(
                child: Padding(
                  padding: const EdgeInsets.all(32),
                  child: Text(
                    'Failed to install SRD content:\n${snapshot.error}',
                    textAlign: TextAlign.center,
                  ),
                ),
              ),
            );
          }
          return widget.child;
        }
        return Scaffold(
          body: Center(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                const Text('Installing SRD content…'),
                const SizedBox(height: 16),
                SizedBox(
                  width: 220,
                  child: LinearProgressIndicator(value: _progress),
                ),
              ],
            ),
          ),
        );
      },
    );
  }
}
