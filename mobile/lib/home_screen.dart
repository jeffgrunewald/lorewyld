import 'package:flutter/material.dart';

import 'dice/dice_icon.dart';
import 'dice/dice_roller_screen.dart';
import 'dice/dice_type.dart';
import 'screens/modules_browse_screen.dart';
import 'screens/search_screen.dart';
import 'screens/setting_list_screen.dart';
import 'services/server_connection.dart';

class HomeScreen extends StatelessWidget {
  const HomeScreen({super.key, required this.connection});

  final ServerConnection connection;

  void _openDiceRoller(BuildContext context) {
    Navigator.of(context).push(
      MaterialPageRoute(builder: (_) => const DiceRollerScreen()),
    );
  }

  void _openSettings(BuildContext context) {
    Navigator.of(context).push(
      MaterialPageRoute(
        builder: (_) => SettingListScreen(connection: connection),
      ),
    );
  }

  void _openModules(BuildContext context) {
    Navigator.of(context).push(
      MaterialPageRoute(
        builder: (_) => ModulesBrowseScreen(connection: connection),
      ),
    );
  }

  void _openSearch(BuildContext context) {
    Navigator.of(context).push(
      MaterialPageRoute(
        builder: (_) => SearchScreen(connection: connection),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final user = connection.user;
    final greeting = user != null ? 'Welcome, ${user.displayName}' : 'Welcome';
    return Scaffold(
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Image.asset(
              'assets/branding/wordmark.png',
              height: 120,
              fit: BoxFit.contain,
              semanticLabel: 'Lorewyld',
            ),
            const SizedBox(height: 24),
            Text(greeting, style: Theme.of(context).textTheme.titleMedium),
            if (connection.serverUrl != null)
              Padding(
                padding: const EdgeInsets.only(top: 4),
                child: Text(
                  connection.serverUrl!,
                  style: Theme.of(context).textTheme.bodySmall,
                ),
              ),
            const SizedBox(height: 24),
            Wrap(
              spacing: 8,
              runSpacing: 8,
              alignment: WrapAlignment.center,
              children: [
                FilledButton.icon(
                  onPressed: () => _openSettings(context),
                  icon: const Icon(Icons.library_books_outlined),
                  label: const Text('Settings & lore'),
                ),
                FilledButton.tonalIcon(
                  onPressed: () => _openModules(context),
                  icon: const Icon(Icons.collections_bookmark_outlined),
                  label: const Text('Modules'),
                ),
                FilledButton.tonalIcon(
                  onPressed: () => _openSearch(context),
                  icon: const Icon(Icons.search),
                  label: const Text('Search'),
                ),
              ],
            ),
          ],
        ),
      ),
      floatingActionButton: _D20FloatingButton(
        onPressed: () => _openDiceRoller(context),
      ),
    );
  }
}

class _D20FloatingButton extends StatelessWidget {
  const _D20FloatingButton({required this.onPressed});

  final VoidCallback onPressed;

  @override
  Widget build(BuildContext context) {
    return Semantics(
      button: true,
      label: 'Open dice roller',
      child: GestureDetector(
        onTap: onPressed,
        behavior: HitTestBehavior.opaque,
        child: const SizedBox(
          width: 72,
          height: 72,
          child: DiceIcon(type: DiceType.d20, size: 72),
        ),
      ),
    );
  }
}
