// Mirror of `lorewyld_types::setting::Setting`.

class Setting {
  final String uuid;
  final String name;
  final String? descriptionNoteUuid;
  final String ownerUserUuid;
  final String? publishedAsModuleUuid;
  final DateTime createdAt;
  final DateTime updatedAt;

  const Setting({
    required this.uuid,
    required this.name,
    this.descriptionNoteUuid,
    required this.ownerUserUuid,
    this.publishedAsModuleUuid,
    required this.createdAt,
    required this.updatedAt,
  });

  factory Setting.fromJson(Map<String, dynamic> json) => Setting(
        uuid: json['uuid'] as String,
        name: json['name'] as String,
        descriptionNoteUuid: json['description_note_uuid'] as String?,
        ownerUserUuid: json['owner_user_uuid'] as String,
        publishedAsModuleUuid: json['published_as_module_uuid'] as String?,
        createdAt: DateTime.parse(json['created_at'] as String),
        updatedAt: DateTime.parse(json['updated_at'] as String),
      );
}
