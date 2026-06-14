// Mirror of `lorewyld_types::tag::Tag`.

class Tag {
  final String uuid;
  final String slug;
  final String displayName;
  final bool isSystem;
  final String? introducedByModuleUuid;
  final DateTime createdAt;

  const Tag({
    required this.uuid,
    required this.slug,
    required this.displayName,
    required this.isSystem,
    this.introducedByModuleUuid,
    required this.createdAt,
  });

  factory Tag.fromJson(Map<String, dynamic> json) => Tag(
    uuid: json['uuid'] as String,
    slug: json['slug'] as String,
    displayName: json['display_name'] as String,
    isSystem: json['is_system'] as bool? ?? false,
    introducedByModuleUuid: json['introduced_by_module_uuid'] as String?,
    createdAt: DateTime.parse(json['created_at'] as String),
  );
}
