/* @vitest-environment jsdom */

import React from 'react';
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { LanguageProvider } from 'sdkwork-drive-pc-commons';
import { StorageProviderNavList } from '../src/components/StorageProviderNavList';
import type { StorageProviderView } from '../src/types/storageProviderAdminTypes';

afterEach(() => cleanup());

/**
 * A row the bootstrap wrote: id prefixed by the server's built-in marker and
 * still carrying the name it seeded.
 */
const BUILT_IN_OSS: StorageProviderView = {
  id: 'builtin-storage-provider-aliyun-oss',
  providerKind: 'aliyun_oss',
  displayName: 'Built-in Alibaba Cloud OSS',
  endpointUrl: 'https://oss-cn-hangzhou.aliyuncs.com',
  region: 'cn-hangzhou',
  bucket: 'sdkwork-drive-aliyun-oss',
  pathStyle: false,
  credentialConfigured: true,
  status: 'active',
  version: 1,
  strictTls: true,
};

/** The same bootstrap row after an operator renamed it — and still without keys. */
const RENAMED_BUILT_IN: StorageProviderView = {
  ...BUILT_IN_OSS,
  id: 'builtin-storage-provider-tencent-cos',
  providerKind: 'tencent_cos',
  displayName: '生产 COS 主账号',
  bucket: 'drive-production',
  credentialConfigured: false,
};

/** A hand-made row named like a built-in one: the id, not the text, decides. */
const HAND_MADE_LOOKALIKE: StorageProviderView = {
  ...BUILT_IN_OSS,
  id: 'provider-hand-made-minio',
  providerKind: 'minio',
  displayName: 'Built-in MinIO',
  bucket: 'drive-minio',
  credentialConfigured: false,
};

function renderRail(
  providers: StorageProviderView[],
  locale: 'zh-CN' | 'en-US' = 'zh-CN',
) {
  const onSelect = vi.fn();
  render(
    <LanguageProvider defaultLanguage={locale} resolveHostLanguage={() => locale}>
      <StorageProviderNavList
        providers={providers}
        selectedProviderId={providers[0]?.id ?? ''}
        onSelect={onSelect}
      />
    </LanguageProvider>,
  );
  return { onSelect };
}

const tab = (label: RegExp) => screen.getByRole('button', { name: label });

describe('StorageProviderNavList credential tabs', () => {
  it('opens on 已配置 and narrows the rail to the rows a tab can act on', () => {
    renderRail([BUILT_IN_OSS, RENAMED_BUILT_IN, HAND_MADE_LOOKALIKE]);

    // 默认停在"已配置"：左栏先回答"现在能用的有哪些"。
    expect(tab(/^已配置/).getAttribute('aria-pressed')).toBe('true');
    expect(tab(/^全部/).getAttribute('aria-pressed')).toBe('false');
    // 内置名按语言显示，运维改的名字原样显示，手建的配置名不做任何处理。
    expect(screen.getByText('内置 阿里云 OSS')).toBeTruthy();
    expect(screen.queryByText('生产 COS 主账号')).toBeNull();
    expect(screen.queryByText('Built-in MinIO')).toBeNull();

    fireEvent.click(tab(/^全部/));
    expect(tab(/^全部/).getAttribute('aria-pressed')).toBe('true');
    expect(screen.getByText('内置 阿里云 OSS')).toBeTruthy();
    expect(screen.getByText('生产 COS 主账号')).toBeTruthy();
    expect(screen.getByText('Built-in MinIO')).toBeTruthy();

    fireEvent.click(tab(/^未配置/));
    expect(tab(/^未配置/).getAttribute('aria-pressed')).toBe('true');
    expect(screen.queryByText('内置 阿里云 OSS')).toBeNull();
    expect(screen.getByText('生产 COS 主账号')).toBeTruthy();
    expect(screen.getByText('Built-in MinIO')).toBeTruthy();
  });

  it('keeps the selection inside the tab the operator is looking at', () => {
    // 页签默认变成"已配置"之后，右栏不能停在左栏根本不显示的那一行上。
    const { onSelect } = renderRail([RENAMED_BUILT_IN, BUILT_IN_OSS]);
    expect(onSelect).toHaveBeenCalledWith(BUILT_IN_OSS.id);
  });

  it('leaves the selection alone when the opening tab has no rows', () => {
    // 空页签只是空：把右栏也清掉比留着一个可用选择更糟。
    const { onSelect } = renderRail([{ ...BUILT_IN_OSS, credentialConfigured: false }]);
    expect(onSelect).not.toHaveBeenCalled();
  });

  it('counts each tab over the loaded rows', () => {
    renderRail([BUILT_IN_OSS, RENAMED_BUILT_IN, HAND_MADE_LOOKALIKE]);

    // 计数与行尾圆点同一个判据：已配置 1 / 未配置 2 / 全部 3。
    expect(tab(/^已配置/).textContent).toContain('1');
    expect(tab(/^未配置/).textContent).toContain('2');
    expect(tab(/^全部/).textContent).toContain('3');
  });

  it('says which side is empty instead of showing a blank rail', () => {
    renderRail([{ ...BUILT_IN_OSS, credentialConfigured: false }]);
    expect(screen.getByText('还没有配置过凭证的存储配置')).toBeTruthy();

    cleanup();

    renderRail([BUILT_IN_OSS]);
    fireEvent.click(tab(/^未配置/));
    expect(screen.getByText('所有存储配置都已配置凭证')).toBeTruthy();
  });

  it('composes the tab with the search box and reports a search miss as one', () => {
    renderRail([BUILT_IN_OSS, RENAMED_BUILT_IN, HAND_MADE_LOOKALIKE]);

    fireEvent.change(screen.getByRole('searchbox'), { target: { value: '阿里云' } });
    // 搜索命中本地化名称：读到的名字就是能搜到的名字。
    expect(screen.getByText('内置 阿里云 OSS')).toBeTruthy();
    expect(screen.queryByText('生产 COS 主账号')).toBeNull();

    fireEvent.click(tab(/^未配置/));
    // 页签与搜索是"且"：两个条件都不满足时说的是"没有匹配"，不是"这一类是空的"。
    expect(screen.getByText('没有匹配的存储配置')).toBeTruthy();
  });

  it('keeps the built-in name visible in English as well', () => {
    renderRail([BUILT_IN_OSS, HAND_MADE_LOOKALIKE], 'en-US');

    expect(screen.getByText('Built-in Alibaba Cloud OSS')).toBeTruthy();
    expect(tab(/^Configured/)).toBeTruthy();
    expect(tab(/^Missing/)).toBeTruthy();
    expect(tab(/^All/)).toBeTruthy();

    fireEvent.click(tab(/^All/));
    expect(screen.getByText('Built-in MinIO')).toBeTruthy();
  });
});
