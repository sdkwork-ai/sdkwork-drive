/**
 * Generates the SDKWork Drive Electron tray icon as a real PNG.
 *
 * The Electron tray controller loads `resources/icons/icon.png` through
 * `nativeImage.createFromPath`; when that file is absent the controller throws
 * `BridgeError('unavailable', ...)` and the tray never appears. This script
 * keeps the asset reproducible without adding an image-processing dependency to
 * the workspace: it rasterizes the same folder glyph used by the Tauri host
 * (`src-tauri/icons/icon.svg`) and encodes it with a minimal PNG writer.
 *
 * Run: node ./scripts/generate-tray-icon.mjs
 */

import { deflateSync } from "node:zlib";
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const SCRIPT_DIR = dirname(fileURLToPath(import.meta.url));
const OUTPUT_PATH = resolve(SCRIPT_DIR, "../resources/icons/icon.png");

const SIZE = 256;

/** Glyph palette mirrored from `src-tauri/icons/icon.svg`. */
const BACKDROP = [15, 23, 42, 255]; // #0F172A
const FOLDER_BACK = [37, 99, 235, 255]; // #2563EB
const FOLDER_FRONT = [56, 189, 248, 255]; // #38BDF8
const MARK = [255, 255, 255, 255];

function lerp(a, b, t) {
  return Math.round(a + (b - a) * t);
}

function mix(from, to, t) {
  return [
    lerp(from[0], to[0], t),
    lerp(from[1], to[1], t),
    lerp(from[2], to[2], t),
    255,
  ];
}

/**
 * Anti-aliased coverage estimate for a rounded rectangle, sampled on a 3x3
 * grid. Returns 0..1 so edges blend instead of looking stair-stepped at the
 * 16-24px sizes a tray actually renders.
 */
function roundedRectCoverage(x, y, left, top, right, bottom, radius) {
  const samples = 3;
  let hits = 0;
  for (let sy = 0; sy < samples; sy += 1) {
    for (let sx = 0; sx < samples; sx += 1) {
      const px = x + (sx + 0.5) / samples;
      const py = y + (sy + 0.5) / samples;
      if (px < left || px > right || py < top || py > bottom) {
        continue;
      }
      const cx = Math.min(Math.max(px, left + radius), right - radius);
      const cy = Math.min(Math.max(py, top + radius), bottom - radius);
      const dx = px - cx;
      const dy = py - cy;
      if (dx * dx + dy * dy <= radius * radius) {
        hits += 1;
      }
    }
  }
  return hits / (samples * samples);
}

/** Folder silhouette: a tab on the left, then the body. */
function folderBodyCoverage(x, y, top, bottom, inset) {
  const left = 38 + inset;
  const right = 218 - inset;
  const radius = 14;
  const bodyTop = top + 24;
  const coverage = roundedRectCoverage(x, y, left, bodyTop, right, bottom, radius);
  if (coverage > 0) {
    return coverage;
  }
  // The tab rises above the body on the upper-left.
  const tabRight = left + 84;
  return roundedRectCoverage(x, y, left, top, tabRight, bodyTop + radius, 12);
}

function horizontalMarkCoverage(x, y, yCenter, halfLength, thickness) {
  const cx = SIZE / 2;
  const dy = Math.abs(y - yCenter);
  if (dy > thickness / 2) {
    return 0;
  }
  const dx = Math.abs(x - cx);
  if (dx > halfLength) {
    return 0;
  }
  return 1;
}

function buildRgbaBuffer() {
  const rgba = Buffer.alloc(SIZE * SIZE * 4, 0);

  for (let y = 0; y < SIZE; y += 1) {
    for (let x = 0; x < SIZE; x += 1) {
      const offset = (y * SIZE + x) * 4;

      // Rounded-square backdrop holds the tray glyph together on any theme.
      const backdrop = roundedRectCoverage(x, y, 8, 8, SIZE - 8, SIZE - 8, 52);
      if (backdrop <= 0) {
        continue;
      }
      let color = mix([0, 0, 0, 0], BACKDROP, backdrop);

      // Rear folder plane.
      const rear = folderBodyCoverage(x, y, 68, 192, 0);
      if (rear > 0) {
        color = mix(color, FOLDER_BACK, rear);
      }

      // Front folder plane, offset down-right to read as depth.
      const front = folderBodyCoverage(x, y, 88, 198, 12);
      if (front > 0) {
        color = mix(color, FOLDER_FRONT, front);
      }

      // Two horizontal marks standing in for document lines.
      const upperMark = horizontalMarkCoverage(x, y, 150, 42, 15);
      const lowerMark = horizontalMarkCoverage(x, y, 175, 26, 15);
      const mark = Math.max(upperMark, lowerMark);
      if (mark > 0) {
        color = mix(color, MARK, mark);
      }

      rgba[offset] = color[0];
      rgba[offset + 1] = color[1];
      rgba[offset + 2] = color[2];
      rgba[offset + 3] = color[3];
    }
  }

  return rgba;
}

const CRC_TABLE = (() => {
  const table = new Int32Array(256);
  for (let n = 0; n < 256; n += 1) {
    let c = n;
    for (let k = 0; k < 8; k += 1) {
      c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    }
    table[n] = c;
  }
  return table;
})();

function crc32(buffer) {
  let crc = -1;
  for (let i = 0; i < buffer.length; i += 1) {
    crc = CRC_TABLE[(crc ^ buffer[i]) & 0xff] ^ (crc >>> 8);
  }
  return (crc ^ -1) >>> 0;
}

function chunk(type, data) {
  const length = Buffer.alloc(4);
  length.writeUInt32BE(data.length, 0);
  const typeBuffer = Buffer.from(type, "ascii");
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(Buffer.concat([typeBuffer, data])), 0);
  return Buffer.concat([length, typeBuffer, data, crc]);
}

function encodePng(rgba) {
  const signature = Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);

  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(SIZE, 0);
  ihdr.writeUInt32BE(SIZE, 4);
  ihdr[8] = 8; // bit depth
  ihdr[9] = 6; // color type: RGBA
  ihdr[10] = 0; // compression
  ihdr[11] = 0; // filter
  ihdr[12] = 0; // interlace

  // Each scanline is prefixed with a filter byte (0 = None).
  const stride = SIZE * 4;
  const raw = Buffer.alloc((stride + 1) * SIZE);
  for (let y = 0; y < SIZE; y += 1) {
    raw[y * (stride + 1)] = 0;
    rgba.copy(raw, y * (stride + 1) + 1, y * stride, (y + 1) * stride);
  }

  return Buffer.concat([
    signature,
    chunk("IHDR", ihdr),
    chunk("IDAT", deflateSync(raw, { level: 9 })),
    chunk("IEND", Buffer.alloc(0)),
  ]);
}

const png = encodePng(buildRgbaBuffer());
mkdirSync(dirname(OUTPUT_PATH), { recursive: true });
writeFileSync(OUTPUT_PATH, png);

/**
 * Windows packaging (`electron-builder` `win.icon`) and the NSIS installer both
 * require a `.ico`. Wrapping the PNG in a single-image ICO keeps one source of
 * truth for the glyph; Vista+ decodes PNG-compressed ICO entries natively.
 */
function encodeIco(pngBuffer) {
  const header = Buffer.alloc(6);
  header.writeUInt16LE(0, 0); // reserved
  header.writeUInt16LE(1, 2); // type: icon
  header.writeUInt16LE(1, 4); // image count

  const entry = Buffer.alloc(16);
  entry[0] = SIZE >= 256 ? 0 : SIZE; // width (0 means 256)
  entry[1] = SIZE >= 256 ? 0 : SIZE; // height (0 means 256)
  entry[2] = 0; // palette colors
  entry[3] = 0; // reserved
  entry.writeUInt16LE(1, 4); // color planes
  entry.writeUInt16LE(32, 6); // bits per pixel
  entry.writeUInt32LE(pngBuffer.length, 8); // size of image data
  entry.writeUInt32LE(6 + 16, 12); // offset to image data

  return Buffer.concat([header, entry, pngBuffer]);
}

const icoPath = resolve(SCRIPT_DIR, "../resources/icons/icon.ico");
writeFileSync(icoPath, encodeIco(png));

/**
 * macOS packaging (`electron-builder` `mac.icon`) requires an `.icns`. CI runs
 * on `macos-latest`, where `iconutil` is available, but relying on it would
 * make the asset non-reproducible off-macOS. Writing the container directly
 * keeps one source of truth for the glyph on every host.
 *
 * The ICNS format is a simple chunk sequence: a 4-byte magic (`icns`), a
 * big-endian total length, then typed chunks. `ic09` carries a 512x512 PNG and
 * `ic10`/`ic14` carry 1024x1024; both accept PNG payloads on macOS 10.7+.
 * Sizes below 256 reuse the same PNG because electron-builder downsamples the
 * largest entry anyway; shipping one 512 entry is enough for a valid bundle.
 */
function encodeIcns(pngBuffer) {
  const chunkHeader = (type) => {
    const header = Buffer.alloc(8);
    header.write(type, 0, 4, "ascii");
    header.writeUInt32BE(pngBuffer.length + 8, 4);
    return header;
  };
  const body = Buffer.concat([chunkHeader("ic09"), pngBuffer]);
  const header = Buffer.alloc(8);
  header.write("icns", 0, 4, "ascii");
  header.writeUInt32BE(body.length + 8, 4);
  return Buffer.concat([header, body]);
}

const icnsPath = resolve(SCRIPT_DIR, "../resources/icons/icon.icns");
writeFileSync(icnsPath, encodeIcns(png));

// eslint-disable-next-line no-console
console.log(
  `[sdkwork-drive-pc-electron] tray icon written: ${OUTPUT_PATH} (${SIZE}x${SIZE}, ${png.length} bytes)`,
);
