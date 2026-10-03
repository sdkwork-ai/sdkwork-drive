import { describe, expect, it } from "vitest";
import { renderToStaticMarkup } from "react-dom/server";
import { createElement, type ReactElement } from "react";
import {
  DriveUploadImageController,
  type DriveUploadImageService,
  type DriveUploadImageValue,
} from "@sdkwork/drive-upload-image-core";
import { DriveUploadImage, DriveUploadImageList } from "../src";

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

function render(element: ReactElement): string {
  return renderToStaticMarkup(element);
}

describe("DriveUploadImage (PC shell)", () => {
  it("renders an accessible pick trigger with placeholder", () => {
    const markup = render(
      createElement(DriveUploadImage, {
        service,
        label: "Avatar",
        description: "PNG up to 5 MB",
      }),
    );
    expect(markup).toContain("Avatar");
    expect(markup).toContain("Upload image");
    expect(markup).toContain("PNG up to 5 MB");
    expect(markup).toContain('type="file"');
  });

  it("renders the stored value preview from an injected controller", () => {
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
    expect(markup).toContain('src="https://cdn.example.com/avatar.png"');
    expect(markup).toContain('alt="Team avatar"');
    expect(markup).toContain("Remove image");
    controller.destroy();
  });

  it("disables interaction when readOnly", () => {
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
    expect(markup).not.toContain("Remove image");
    expect(markup).toContain("aria-disabled=\"true\"");
    controller.destroy();
  });

  it("renders localized copy overrides", () => {
    const markup = render(
      createElement(DriveUploadImage, {
        service,
        copy: { pickImage: "上传图片" },
      }),
    );
    expect(markup).toContain("上传图片");
  });
});

describe("DriveUploadImageList (PC shell)", () => {
  it("renders stored values and keeps the add tile under the cap", () => {
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
        label: "Gallery",
      }),
    );
    expect(markup).toContain("Gallery");
    expect(markup).toContain('src="https://cdn.example.com/avatar.png"');
    expect(markup).toContain('aria-label="Upload image"');
    controller.destroy();
  });

  it("hides the add tile at the cap", () => {
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
    expect(markup).not.toContain('aria-label="Upload image"');
    expect(markup).toContain('src="https://cdn.example.com/second.png"');
    controller.destroy();
  });
});
