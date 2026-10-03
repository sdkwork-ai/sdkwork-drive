/**
 * Minimal structural subset of the WeChat mini-program APIs the picker needs.
 * The adapter receives the host object explicitly (`wx` on device, a fake in
 * tests), so this package carries no ambient global declarations and stays
 * unit-testable under Node.
 */

export interface WxMediaFile {
  tempFilePath: string;
  size: number;
}

export interface WxFileSystemManagerLike {
  readFile(options: {
    filePath: string;
    /** Ranged read; both fields are required to read a byte window. */
    position?: number;
    length?: number;
    success: (result: { data: ArrayBuffer }) => void;
    fail: (error: { errMsg: string }) => void;
  }): void;
}

export interface WxMiniProgramLike {
  chooseMedia(options: {
    count: number;
    mediaType: ["image"];
    sourceType: Array<"album" | "camera">;
    sizeType?: Array<"original" | "compressed">;
    success: (result: { tempFiles: WxMediaFile[] }) => void;
    fail: (error: { errMsg: string }) => void;
  }): void;
  getFileSystemManager(): WxFileSystemManagerLike;
}

/** Structural subset of `globalThis` that may carry the mini-program host. */
export function resolveGlobalWx(host: unknown = globalThis): WxMiniProgramLike | null {
  if (
    host !== null &&
    typeof host === "object" &&
    "wx" in host &&
    typeof (host as { wx?: unknown }).wx === "object" &&
    (host as { wx?: unknown }).wx !== null
  ) {
    const wx = (host as { wx: unknown }).wx as Partial<WxMiniProgramLike>;
    if (typeof wx.chooseMedia === "function" && typeof wx.getFileSystemManager === "function") {
      return wx as WxMiniProgramLike;
    }
  }
  return null;
}
