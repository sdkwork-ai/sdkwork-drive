/* @vitest-environment node */

import { describe, expect, it } from 'vitest';
import {
  FALLBACK_FILE_KIND_VISUAL,
  FILE_KIND_VISUALS,
  fileKindVisual,
} from '../src/kinds/fileKindVisuals';
import { FILE_PREVIEW_KINDS } from '../src/kinds/filePreviewKind';

/**
 * 图标配色的对比度闸门。
 *
 * 列表里每种文件靠彩色图块区分，图块上的图标必须看得清：WCAG 1.4.11 要求图形对象
 * 与相邻颜色的对比度不低于 3:1。这条线不能靠肉眼守——本轮审计就用它发现文件夹图标
 * 在浅色底上只有 **2.07:1**（`amber-500` on `amber-50`），而文件夹正是列表里出现
 * 最多的元素。
 *
 * 颜色值直接取自 Tailwind v4.3.3 的 `theme.css`（OKLCh），在这里内联一份是为了让
 * 预览包保持零构建期依赖：测试只依赖"我们选了哪两个色阶"这一事实。
 */
const OKLCH: Readonly<Record<string, readonly [number, number, number]>> = {
  // [亮度(0-1), 彩度, 色相]
  'amber-50': [0.987, 0.022, 95.277],
  'amber-300': [0.879, 0.169, 91.605],
  'amber-500': [0.769, 0.188, 70.08],
  'amber-600': [0.666, 0.179, 58.318],
  'amber-700': [0.555, 0.163, 48.998],
  'amber-950': [0.279, 0.077, 45.635],
  'blue-50': [0.97, 0.014, 254.604],
  'blue-300': [0.809, 0.105, 251.813],
  'blue-700': [0.488, 0.217, 264.376],
  'blue-950': [0.282, 0.091, 267.935],
  'cyan-50': [0.984, 0.019, 200.873],
  'cyan-300': [0.865, 0.127, 207.078],
  'cyan-700': [0.52, 0.105, 223.128],
  'cyan-950': [0.302, 0.056, 229.695],
  'emerald-50': [0.979, 0.021, 166.113],
  'emerald-300': [0.845, 0.143, 164.978],
  'emerald-700': [0.508, 0.15, 162.48],
  'emerald-950': [0.262, 0.051, 172.552],
  'fuchsia-50': [0.977, 0.017, 320.058],
  'fuchsia-300': [0.833, 0.145, 321.434],
  'fuchsia-700': [0.518, 0.253, 323.949],
  'fuchsia-950': [0.293, 0.136, 325.661],
  'neutral-100': [0.97, 0, 0],
  'neutral-300': [0.87, 0, 0],
  'neutral-600': [0.439, 0, 0],
  'neutral-800': [0.269, 0, 0],
  'neutral-950': [0.145, 0, 0],
  'orange-50': [0.98, 0.016, 73.684],
  'orange-300': [0.837, 0.128, 66.29],
  'orange-700': [0.553, 0.195, 38.402],
  'orange-950': [0.266, 0.079, 36.259],
  'red-50': [0.971, 0.013, 17.38],
  'red-300': [0.808, 0.114, 19.571],
  'red-700': [0.505, 0.213, 27.518],
  'red-950': [0.258, 0.092, 26.042],
  'rose-50': [0.969, 0.015, 12.422],
  'rose-300': [0.81, 0.117, 11.638],
  'rose-600': [0.586, 0.253, 17.585],
  'rose-950': [0.271, 0.105, 12.094],
  'sky-50': [0.977, 0.013, 236.62],
  'sky-300': [0.827, 0.101, 230.318],
  'sky-700': [0.5, 0.134, 242.749],
  'sky-950': [0.293, 0.066, 242.749],
  'slate-100': [0.968, 0.007, 247.896],
  'slate-300': [0.869, 0.022, 252.894],
  'slate-600': [0.446, 0.043, 257.281],
  'slate-800': [0.279, 0.041, 260.031],
  'violet-50': [0.969, 0.016, 293.756],
  'violet-300': [0.811, 0.111, 293.571],
  'violet-600': [0.541, 0.281, 293.009],
  'violet-700': [0.491, 0.27, 292.581],
  'violet-950': [0.283, 0.141, 291.089],
  'yellow-50': [0.987, 0.026, 102.212],
  'yellow-300': [0.905, 0.182, 98.111],
  'yellow-700': [0.554, 0.135, 66.442],
  'yellow-950': [0.286, 0.066, 53.813],
};

const srgbToLinear = (value: number) =>
  value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;

/** OKLCh → 线性 sRGB（Ottosson 的矩阵）。 */
function oklchToLinearRgb([lightness, chroma, hue]: readonly [number, number, number]) {
  const hRad = (hue * Math.PI) / 180;
  const a = chroma * Math.cos(hRad);
  const b = chroma * Math.sin(hRad);
  const l = (lightness + 0.3963377774 * a + 0.2158037573 * b) ** 3;
  const m = (lightness - 0.1055613458 * a - 0.0638541728 * b) ** 3;
  const s = (lightness - 0.0894841775 * a - 1.291485548 * b) ** 3;
  return [
    4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
    -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
    -0.0041960863 * l - 0.7034186147 * m + 1.707614701 * s,
  ] as const;
}

type Rgb = readonly [number, number, number];

const clamp01 = (value: number) => Math.min(1, Math.max(0, value));
const luminance = ([r, g, b]: Rgb) =>
  0.2126 * clamp01(r) + 0.7152 * clamp01(g) + 0.0722 * clamp01(b);

function contrastRatio(foreground: Rgb, background: Rgb): number {
  const a = luminance(foreground);
  const b = luminance(background);
  const [hi, lo] = a > b ? [a, b] : [b, a];
  return (hi + 0.05) / (lo + 0.05);
}

/** 解析 `bg-amber-50` / `dark:bg-amber-950/40` / `text-amber-700` 并与底色合成。 */
function resolveUtility(utility: string, surface: Rgb): Rgb {
  const token = utility.replace(/^dark:/, '').replace(/^(bg|text)-/, '');
  const [colorName, alphaText] = token.split('/');
  const base = OKLCH[colorName];
  if (!base) {
    throw new Error(`unknown palette token in a file kind visual: ${utility}`);
  }
  const rgb = oklchToLinearRgb(base);
  const alpha = alphaText ? Number(alphaText) / 100 : 1;
  return rgb.map((channel, index) => channel * alpha + surface[index] * (1 - alpha)) as unknown as Rgb;
}

const LIGHT_SURFACE: Rgb = srgbToLinear(1) === 1 ? [1, 1, 1] : [1, 1, 1];
const DARK_SURFACE: Rgb = oklchToLinearRgb(OKLCH['neutral-950'] as readonly [number, number, number]);

/** 每种 kind 在两种主题下的图标/图块对比度。 */
function ratiosFor(iconClass: string, tileClass: string) {
  const tokens = tileClass.split(/\s+/);
  const lightTile = resolveUtility(tokens.find((t) => !t.startsWith('dark:'))!, LIGHT_SURFACE);
  const darkTileToken = tokens.find((t) => t.startsWith('dark:'));
  const darkTile = darkTileToken ? resolveUtility(darkTileToken, DARK_SURFACE) : lightTile;
  const iconTokens = iconClass.split(/\s+/);
  const lightIconToken = iconTokens.find((t) => !t.startsWith('dark:'))!;
  const darkIconToken = iconTokens.find((t) => t.startsWith('dark:')) ?? lightIconToken;
  return {
    dark: contrastRatio(resolveUtility(darkIconToken, darkTile), darkTile),
    light: contrastRatio(resolveUtility(lightIconToken, lightTile), lightTile),
  };
}

describe('file kind visuals contrast', () => {
  it('covers every preview kind plus a fallback', () => {
    for (const kind of FILE_PREVIEW_KINDS) {
      expect(FILE_KIND_VISUALS[kind], `missing visuals for ${kind}`).toBeTruthy();
    }
    expect(fileKindVisual('unsupported')).toBe(FILE_KIND_VISUALS.unsupported);
    expect(FALLBACK_FILE_KIND_VISUAL.iconClass).toMatch(/^text-/);
  });

  it('keeps every icon at or above the 3:1 graphical contrast threshold', () => {
    const failures: string[] = [];
    const measured: Record<string, { dark: number; light: number }> = {};
    for (const kind of FILE_PREVIEW_KINDS) {
      const visual = FILE_KIND_VISUALS[kind];
      const ratios = ratiosFor(visual.iconClass, visual.tileClass);
      measured[kind] = ratios;
      // 四舍五入到两位再判定：避免浮点误差把刚好达标的色阶判死。
      if (Math.round(ratios.light * 100) / 100 < 3) {
        failures.push(`${kind} light ${ratios.light.toFixed(2)}:1`);
      }
      if (Math.round(ratios.dark * 100) / 100 < 3) {
        failures.push(`${kind} dark ${ratios.dark.toFixed(2)}:1`);
      }
    }
    expect(failures).toEqual([]);
    // 记录最差的一对，便于下次调色时知道还有多少余量。
    const worst = Math.min(...Object.values(measured).flatMap((r) => [r.light, r.dark]));
    expect(worst).toBeGreaterThan(3);
  });

  it('keeps the fallback visual as legible as the declared kinds', () => {
    const ratios = ratiosFor(FALLBACK_FILE_KIND_VISUAL.iconClass, FALLBACK_FILE_KIND_VISUAL.tileClass);
    expect(ratios.light).toBeGreaterThan(3);
    expect(ratios.dark).toBeGreaterThan(3);
  });

  it('gives every kind its own tile and icon colour', () => {
    // 颜色是列表里最快的分类信号；两类共用一套色阶时，用户只能靠读文件名区分。
    // 这条用例把"13 类两两不同色"变成可执行的契约（曾经文件夹与幻灯片都是琥珀）。
    const seen = new Map<string, string>();
    const collisions: string[] = [];
    for (const kind of FILE_PREVIEW_KINDS) {
      const visual = FILE_KIND_VISUALS[kind];
      const signature = `${visual.tileClass}|${visual.iconClass}`;
      const previous = seen.get(signature);
      if (previous) {
        collisions.push(`${kind} and ${previous} share "${signature}"`);
      } else {
        seen.set(signature, kind);
      }
    }
    expect(collisions).toEqual([]);
  });
});
