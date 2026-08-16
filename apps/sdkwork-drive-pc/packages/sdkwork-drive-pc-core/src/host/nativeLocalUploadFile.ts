import type { DriveUploaderBlobLike } from '@sdkwork/drive-app-sdk';
import type { HostAdapter } from './hostAdapter';

export interface NativeLocalUploadDescriptor {
  path: string;
  name: string;
  size: number;
  modifiedAt: string;
  mimeType: string;
}

export class NativeLocalUploadFile implements DriveUploaderBlobLike {
  readonly path: string;
  readonly name: string;
  readonly size: number;
  readonly type: string;
  readonly lastModified: number;

  constructor(
    descriptor: NativeLocalUploadDescriptor,
    private readonly host: Pick<HostAdapter, 'readLocalUploadRange'>,
  ) {
    this.path = descriptor.path;
    this.name = descriptor.name;
    this.size = descriptor.size;
    this.type = descriptor.mimeType;
    this.lastModified = Number(descriptor.modifiedAt) || 0;
  }

  slice(): Blob {
    throw new Error('Native local uploads must use readRange instead of Blob.slice().');
  }

  async readRange(offsetBytes: number, lengthBytes: number): Promise<ArrayBuffer> {
    return this.host.readLocalUploadRange(this.path, offsetBytes, lengthBytes);
  }

  async arrayBuffer(): Promise<ArrayBuffer> {
    if (this.size === 0) {
      return new ArrayBuffer(0);
    }
    return this.readRange(0, this.size);
  }
}

export function isNativeLocalUploadFile(
  file: DriveUploaderBlobLike,
): file is NativeLocalUploadFile {
  return file instanceof NativeLocalUploadFile;
}
