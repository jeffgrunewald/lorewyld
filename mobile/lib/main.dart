import 'package:flutter/material.dart';

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
      home: LorewyldShell(connection: connection, store: store),
    );
  }
}
