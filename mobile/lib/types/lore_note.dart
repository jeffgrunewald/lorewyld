// Mirrors of `lorewyld_types::lore_note::*`.

import 'tag.dart';

enum NoteScopeKind {
  module('module'),
  setting('setting'),
  campaign('campaign'),
  character('character');

  final String wire;
  const NoteScopeKind(this.wire);

  static NoteScopeKind fromWire(String s) =>
      NoteScopeKind.values.firstWhere((v) => v.wire == s,
          orElse: () => throw FormatException('unknown scope kind: $s'));
}

enum NoteVisibility {
  visible('visible'),
  authorOnly('author_only'),
  gamemasterOnly('gamemaster_only');

  final String wire;
  const NoteVisibility(this.wire);

  static NoteVisibility fromWire(String s) =>
      NoteVisibility.values.firstWhere((v) => v.wire == s,
          orElse: () => throw FormatException('unknown visibility: $s'));
}

class NoteScope {
  final NoteScopeKind kind;
  final String targetUuid;

  const NoteScope({required this.kind, required this.targetUuid});

  factory NoteScope.fromJson(Map<String, dynamic> json) => NoteScope(
        kind: NoteScopeKind.fromWire(json['kind'] as String),
        targetUuid: json['target_uuid'] as String,
      );

  Map<String, dynamic> toJson() => {
        'kind': kind.wire,
        'target_uuid': targetUuid,
      };
}

class LoreNote {
  final String uuid;
  final String title;
  final String bodyMarkdown;
  final NoteScope scope;
  final NoteVisibility visibility;
  final String? derivedFromSettingNoteUuid;
  final String createdByUserUuid;
  final DateTime createdAt;
  final DateTime updatedAt;

  const LoreNote({
    required this.uuid,
    required this.title,
    required this.bodyMarkdown,
    required this.scope,
    required this.visibility,
    this.derivedFromSettingNoteUuid,
    required this.createdByUserUuid,
    required this.createdAt,
    required this.updatedAt,
  });

  factory LoreNote.fromJson(Map<String, dynamic> json) => LoreNote(
        uuid: json['uuid'] as String,
        title: json['title'] as String,
        bodyMarkdown: json['body_markdown'] as String? ?? '',
        scope: NoteScope.fromJson(json['scope'] as Map<String, dynamic>),
        visibility: NoteVisibility.fromWire(
            json['visibility'] as String? ?? 'visible'),
        derivedFromSettingNoteUuid:
            json['derived_from_setting_note_uuid'] as String?,
        createdByUserUuid: json['created_by_user_uuid'] as String,
        createdAt: DateTime.parse(json['created_at'] as String),
        updatedAt: DateTime.parse(json['updated_at'] as String),
      );
}

class LoreNoteWithTags {
  final LoreNote note;
  final List<Tag> tags;

  const LoreNoteWithTags({required this.note, required this.tags});

  factory LoreNoteWithTags.fromJson(Map<String, dynamic> json) =>
      LoreNoteWithTags(
        note: LoreNote.fromJson(json['note'] as Map<String, dynamic>),
        tags: (json['tags'] as List<dynamic>? ?? const [])
            .map((e) => Tag.fromJson(e as Map<String, dynamic>))
            .toList(),
      );
}
