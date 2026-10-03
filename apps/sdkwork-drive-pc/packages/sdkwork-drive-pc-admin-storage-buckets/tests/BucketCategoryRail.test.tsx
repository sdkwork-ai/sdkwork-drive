/* @vitest-environment jsdom */

import React from 'react';
import { cleanup, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { StorageProviderView } from 'sdkwork-drive-pc-admin-storage-providers';
import { BucketCategoryRail } from '../src/components/BucketCategoryRail';

/**
 * 左栏的横向溢出守卫。
 *
 * 截图里的横向滚动条来自隐式网格轨道：`<dl>` 的默认轨道按 max-content 定宽，而
 * `truncate` 的 max-content 就是整串文本宽度，于是那条长端点 URL 把整个信息块撑得比
 * 232px 的左栏还宽；再加上 `overflow-y-auto` 会让另一轴的计算值也变成 `auto`，浏览器
 * 就画出一条横向滚动条。
 *
 * jsdom 不做布局，量不出 scrollWidth，所以这里锁住的是"修复的结构前提"：轨道被显式约束、
 * 每一行都能收缩、栏本身裁掉横向溢出。三者缺一，滚动条就会回来。
 */
const PROVIDER = {
  id: 'provider-cos',
  providerKind: 'tencent_cos',
  displayName: 'Built-in Tencent Cloud COS',
  endpointUrl: 'https://cos.ap-guangzhou.myqcloud.com',
  region: 'ap-guangzhou',
  bucket: 'image2-1253947560',
  pathStyle: false,
  credentialConfigured: true,
  status: 'active',
  version: 1,
  strictTls: true,
} as unknown as StorageProviderView;

afterEach(() => cleanup());

function renderRail() {
  return render(
    <BucketCategoryRail
      bucket="image2-1253947560"
      category="all"
      counts={{ all: 100, archive: 0, audio: 0, document: 0, folder: 1, image: 99, other: 0, video: 0 }}
      loadedBytes={18_400_000}
      onSelectCategory={vi.fn()}
      provider={PROVIDER}
    />,
  );
}

describe('BucketCategoryRail', () => {
  it('cannot produce a horizontal scrollbar', () => {
    const { container } = renderRail();

    const aside = container.querySelector('aside');
    expect(aside?.className).toContain('overflow-x-hidden');
    // 纵向滚动与固定栏宽保持不变：修的是横向，不是布局。
    expect(aside?.className).toContain('lg:overflow-y-auto');
    expect(aside?.className).toContain('lg:w-[232px]');
  });

  it('constrains the info grid track so truncation actually truncates', () => {
    const { container } = renderRail();

    const list = container.querySelector('dl');
    expect(list?.className).toContain('grid-cols-[minmax(0,1fr)]');
  });

  it('lets every info row shrink below its content width', () => {
    const { container } = renderRail();

    const rows = Array.from(container.querySelectorAll('dl > div'));
    expect(rows.length).toBeGreaterThan(4);
    for (const row of rows) {
      expect(row.className).toContain('min-w-0');
    }
    // 值单元格必须可收缩且带省略号，否则长 URL 只能溢出。
    const values = Array.from(container.querySelectorAll('dl dd'));
    expect(values[0]?.className).toContain('truncate');
    expect(values.every((value) => !value.className.includes('truncate') || value.className.includes('min-w-0'))).toBe(true);
  });

  it('still renders the category counts and storage facts', () => {
    renderRail();

    expect(screen.getByText('image2-1253947560')).toBeTruthy();
    expect(screen.getByText('Built-in Tencent Cloud COS')).toBeTruthy();
    expect(screen.getByText('https://cos.ap-guangzhou.myqcloud.com')).toBeTruthy();
    // 已加载大小按宿主格式化函数渲染，断言"有值"而不是钉死某一种单位写法。
    expect(screen.getByText(/\d+(\.\d+)?\s?(B|KB|MB|GB)/)).toBeTruthy();
  });
});
