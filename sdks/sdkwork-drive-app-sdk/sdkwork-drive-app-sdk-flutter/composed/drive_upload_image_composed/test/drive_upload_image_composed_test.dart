import 'dart:typed_data';

import 'package:drive_uploader_composed/drive_uploader_composed.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:drive_upload_image_composed/drive_upload_image_composed.dart';

DriveUploadImageSource pngSource({int sizeBytes = 8, String name = 'cat.png'}) {
  return DriveUploadImageSource(
    bytes: Uint8List(sizeBytes)..fillRange(0, sizeBytes, 7),
    fileName: name,
    contentType: 'image/png',
  );
}

DriveUploadImageValue driveValue(String nodeId) => DriveUploadImageValue(
      uri: 'drive://spaces/space_1/nodes/$nodeId',
      source: 'drive',
      metadata: <String, dynamic>{
        'drive': <String, dynamic>{
          'spaceId': 'space_1',
          'nodeId': nodeId,
          'originalFileName': '$nodeId.png',
        },
      },
    );

DriveUploaderImageService buildService() {
  return DriveUploaderImageService(
    uploader: _FakeUploader(),
    declaration: const DriveUploadImageDeclaration(
      appResourceType: 'profile.avatar',
      appResourceIdKind: 'entity',
      scene: 'avatar',
      source: 'sdkwork-test-app',
      uploadProfileCode: DriveUploadImageProfile.avatar,
      retention: 'long_term',
      purpose: 'Test avatar uploads for the reusable image component.',
    ),
  );
}

class _FakeUploader implements DriveUploaderClient {
  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _ScriptedService implements DriveUploadImageService {
  _ScriptedService(this._onUpload);

  final DriveUploadImageValue Function(DriveUploadImageSource source,
      String appResourceId) _onUpload;
  final List<String> anchors = <String>[];

  @override
  Future<DriveUploadImageValue> upload({
    required DriveUploadImageSource file,
    required String appResourceId,
    void Function(DriveUploadImageProgress progress)? onProgress,
    bool Function()? shouldAbort,
  }) async {
    anchors.add(appResourceId);
    return _onUpload(file, appResourceId);
  }

  @override
  Future<String?> resolvePreview({required String uri, int? maxBytes}) async =>
      uri.contains('loaded') ? 'data:image/png;base64,AAA' : null;
}

void main() {
  group('DriveUploaderImageService', () {
    test('rejects a non-conforming declaration up front', () {
      expect(
        () => DriveUploaderImageService(
          uploader: _FakeUploader(),
          declaration: const DriveUploadImageDeclaration(
            appResourceType: 'profile.avatar',
            appResourceIdKind: 'entity',
            scene: 'im',
            source: 'sdkwork-test-app',
            uploadProfileCode: DriveUploadImageProfile.avatar,
            retention: 'long_term',
            purpose: 'Reserved scene must be rejected.',
          ),
        ),
        throwsA(isA<DriveUploadImageException>()),
      );
    });
  });

  group('DriveUploadImageController', () {
    test('validates sources and keeps the rejection on the item',
        () async {
      final controller = DriveUploadImageController(
        service: _ScriptedService((source, anchor) => driveValue('unused')),
        maxSizeBytes: 16,
      );
      await controller.addSources(<DriveUploadImageSource>[pngSource(sizeBytes: 32)]);
      expect(controller.snapshot.items.single.status,
          DriveUploadImageItemStatus.rejected);
      expect(controller.snapshot.items.single.rejectionCode,
          DriveUploadImageRejectionCode.fileTooLarge);
      expect(controller.values, isEmpty);
      controller.dispose();
    });

    test('defers the upload without an anchor and uploads with one',
        () async {
      final service = _ScriptedService(
          (source, anchor) => driveValue('node_$anchor'));
      final controller = DriveUploadImageController(service: service);
      await controller.addSources(<DriveUploadImageSource>[pngSource()]);
      expect(controller.snapshot.hasPending, isTrue);
      expect(service.anchors, isEmpty);

      final values = await controller.uploadPending(appResourceId: 'user_1');
      expect(service.anchors, <String>['user_1']);
      expect(values.single.uri, 'drive://spaces/space_1/nodes/node_user_1');
      expect(controller.snapshot.items.single.status,
          DriveUploadImageItemStatus.uploaded);
      controller.dispose();
    });

    test('single-field mode replaces the stored image on a new pick',
        () async {
      var counter = 0;
      final controller = DriveUploadImageController(
        service: _ScriptedService(
            (source, anchor) => driveValue('node_${++counter}')),
        resolveAppResourceId: () => 'user_01',
      );
      await controller.addSources(<DriveUploadImageSource>[
        pngSource(name: 'first.png'),
      ]);
      await controller.addSources(<DriveUploadImageSource>[
        pngSource(name: 'second.png'),
      ]);
      expect(controller.values.single.uri,
          'drive://spaces/space_1/nodes/node_2');
      expect(controller.snapshot.items, hasLength(1));
      controller.dispose();
    });

    test('setValues round-trips without losing transient state', () async {
      final controller = DriveUploadImageController(
        service: _ScriptedService(
            (source, anchor) => driveValue('node_roundtrip')),
        resolveAppResourceId: () => 'user_01',
      );
      await controller.addSources(<DriveUploadImageSource>[pngSource()]);
      final value = controller.values.single;
      controller.setValues(<DriveUploadImageValue>[value]);
      expect(controller.snapshot.items, hasLength(1));
      expect(controller.snapshot.items.single.status,
          DriveUploadImageItemStatus.uploaded);
      controller.setValues(<DriveUploadImageValue>[driveValue('external')]);
      expect(controller.values.single.uri, driveValue('external').uri);
      controller.dispose();
    });

    test('disposal rejects further use', () async {
      final controller = DriveUploadImageController(
        service: _ScriptedService((source, anchor) => driveValue('x')),
      );
      controller.dispose();
      expect(
        () => controller.addSources(const <DriveUploadImageSource>[]),
        throwsA(isA<DriveUploadImageException>()),
      );
    });
  });

  group('DriveUploadImageField', () {
    testWidgets('renders the pick affordance and localized copy',
        (WidgetTester tester) async {
      await tester.pumpWidget(MaterialApp(
        home: Scaffold(
          body: DriveUploadImageField(
            service: _ScriptedService((source, anchor) => driveValue('x')),
            onPick: () async => <DriveUploadImageSource>[],
            label: 'Avatar',
            copy: const DriveUploadImageCopy(pickImage: '上传图片'),
          ),
        ),
      ));
      expect(find.text('Avatar'), findsOneWidget);
      expect(find.text('上传图片'), findsOneWidget);
    });

    testWidgets('renders the stored external value and remove chip',
        (WidgetTester tester) async {
      const value = DriveUploadImageValue(
        uri: 'https://cdn.example.com/avatar.png',
        source: 'external',
      );
      final controller = DriveUploadImageController(
        service: _ScriptedService((source, anchor) => driveValue('x')),
      );
      controller.setValues(<DriveUploadImageValue>[value]);
      await tester.pumpWidget(MaterialApp(
        home: Scaffold(
          body: DriveUploadImageField(
            service: _ScriptedService((source, anchor) => driveValue('x')),
            onPick: () async => <DriveUploadImageSource>[],
            controller: controller,
            value: value,
          ),
        ),
      ));
      await tester.pump();
      expect(find.byType(Image), findsOneWidget);
      expect(find.text('Remove image'), findsOneWidget);
      controller.dispose();
    });

    testWidgets('picking through the host adapter uploads and reports',
        (WidgetTester tester) async {
      DriveUploadImageValue? reported;
      final controller = DriveUploadImageController(
        service: _ScriptedService(
            (source, anchor) => driveValue('node_$anchor')),
        resolveAppResourceId: () => 'user_42',
      );
      await tester.pumpWidget(MaterialApp(
        home: Scaffold(
          body: DriveUploadImageField(
            service: _ScriptedService((source, anchor) => driveValue('x')),
            onPick: () async => <DriveUploadImageSource>[pngSource()],
            controller: controller,
            onChanged: (value) => reported = value,
          ),
        ),
      ));
      await tester.tap(find.byType(GestureDetector).first);
      await tester.pumpAndSettle();
      expect(reported?.uri, 'drive://spaces/space_1/nodes/node_user_42');
      controller.dispose();
    });
  });
}
