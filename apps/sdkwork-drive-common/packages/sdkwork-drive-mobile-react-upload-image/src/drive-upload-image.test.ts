import assert from "node:assert/strict";
import { test } from "node:test";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import {
  DriveUploadImageController,
  type DriveUploadImageService,
  type DriveUploadImageValue,
} from "@sdkwork/drive-upload-image-core";
import { DriveUploadImage, DriveUploadImageList } from "./index";

const service: DriveUploadImageService = {
  upload: () =>
    Promise.resolve({
      uri: "drive://spaces/space_1/nodes/node_1",
      source: "drive",
    }),
  resolvePreview: () => Promise.resolve(null),
};

const externalImage: DriveUploadImageValue = {
  uri: "https://cdn.example.com/avatar.png",
  source: "external",
};

const nullPreviews = {
  createObjectUrl: () => null,
  revokeObjectUrl: () => undefined,
};

function render(element: ReturnType<typeof createElement>): string {
  return renderToStaticMarkup(element);
}

test("single field renders an accessible pick target and localized copy", () => {
  const markup = render(
    createElement(DriveUploadImage, {
      service,
      label: "头像",
      description: "不超过 5 MB",
      copy: { pickImage: "上传图片", cancel: "取消" },
    }),
  );
  assert.match(markup, /头像/);
  assert.match(markup, /上传图片/);
  assert.match(markup, /不超过 5 MB/);
  assert.match(markup, /type="file"/);
});

test("single field renders the stored external value preview", () => {
  const controller = new DriveUploadImageController({
    service,
    previewUrls: nullPreviews,
  });
  controller.setValues([externalImage]);
  const markup = render(
    createElement(DriveUploadImage, {
      service,
      controller,
      value: externalImage,
      alt: "Team avatar",
    }),
  );
  assert.match(markup, /src="https:\/\/cdn\.example\.com\/avatar\.png"/);
  assert.match(markup, /alt="Team avatar"/);
  assert.match(markup, /Remove image/);
  controller.destroy();
});

test("readOnly single field hides mutation affordances", () => {
  const controller = new DriveUploadImageController({
    service,
    previewUrls: nullPreviews,
  });
  controller.setValues([externalImage]);
  const markup = render(
    createElement(DriveUploadImage, {
      service,
      controller,
      value: externalImage,
      readOnly: true,
    }),
  );
  assert.equal(markup.includes("Remove image"), false);
  assert.match(markup, /aria-disabled="true"/);
  controller.destroy();
});

test("list renders stored values and keeps the add tile under the cap", () => {
  const controller = new DriveUploadImageController({
    service,
    previewUrls: nullPreviews,
    maxFiles: 2,
    replaceOnMax: false,
  });
  controller.setValues([externalImage]);
  const markup = render(
    createElement(DriveUploadImageList, {
      service,
      controller,
      value: [externalImage],
      maxFiles: 2,
    }),
  );
  assert.match(markup, /src="https:\/\/cdn\.example\.com\/avatar\.png"/);
  assert.match(markup, /aria-label="Upload image"/);
  controller.destroy();
});

test("list hides the add tile at the cap", () => {
  const controller = new DriveUploadImageController({
    service,
    previewUrls: nullPreviews,
    maxFiles: 2,
    replaceOnMax: false,
  });
  const second: DriveUploadImageValue = {
    uri: "https://cdn.example.com/second.png",
    source: "external",
  };
  controller.setValues([externalImage, second]);
  const markup = render(
    createElement(DriveUploadImageList, {
      service,
      controller,
      value: [externalImage, second],
      maxFiles: 2,
    }),
  );
  assert.equal(markup.includes('aria-label="Upload image"'), false);
  assert.match(markup, /src="https:\/\/cdn\.example\.com\/second\.png"/);
  controller.destroy();
});
