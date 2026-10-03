/* @vitest-environment jsdom */

import React from 'react';
import { cleanup, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import { LanguageProvider } from 'sdkwork-drive-pc-commons';
import { useTranslation } from '../src/hooks/useTranslation';
import {
  BUILTIN_PROVIDER_DISPLAY_NAMES,
  BUILTIN_PROVIDER_ID_PREFIX,
  providerDisplayName,
} from '../src/utils/providerKindConfig';

afterEach(() => cleanup());

/** A stub dictionary that answers for the two keys the rule can ask for. */
const stubDictionary = (key: string): string => `translated:${key}`;

/**
 * What a component gets back when no `LanguageProvider` is mounted: the bare
 * key, unprefixed. The rule has to treat that as a miss too, or a host that
 * forgot the provider would paint raw keys into its table.
 */
const providerlessDictionary = (key: string): string => key;

const builtInRow = (kind: string) => ({
  id: `${BUILTIN_PROVIDER_ID_PREFIX}${kind.replace(/_/g, '-')}`,
  providerKind: kind,
  displayName: BUILTIN_PROVIDER_DISPLAY_NAMES[kind],
});

describe('provider configuration name rule', () => {
  it('translates a built-in row that still carries its bootstrap name', () => {
    expect(providerDisplayName(stubDictionary, builtInRow('aliyun_oss')))
      .toBe('translated:builtInProviderName.aliyun_oss');
  });

  it('keeps a name an operator chose, even on a built-in row', () => {
    // 运维改过名的配置是他自己的数据，翻译它等于替他改名。
    expect(
      providerDisplayName(stubDictionary, {
        ...builtInRow('aliyun_oss'),
        displayName: '生产 OSS 主账号',
      }),
    ).toBe('生产 OSS 主账号');
  });

  it('keeps a hand-made row that happens to be named like a built-in one', () => {
    // id 前缀才是"这行是内置的"的证据，名字不是。
    expect(
      providerDisplayName(stubDictionary, {
        id: 'provider-hand-made',
        providerKind: 'aliyun_oss',
        displayName: BUILTIN_PROVIDER_DISPLAY_NAMES.aliyun_oss,
      }),
    ).toBe(BUILTIN_PROVIDER_DISPLAY_NAMES.aliyun_oss);
  });

  it('keeps the stored name for a kind outside the built-in catalog', () => {
    for (const kind of ['custom', 'custom:acme', 'vendor_added_later']) {
      expect(
        providerDisplayName(stubDictionary, {
          id: BUILTIN_PROVIDER_ID_PREFIX + 'whatever',
          providerKind: kind,
          displayName: 'Anything At All',
        }),
        `${kind} must not be translated`,
      ).toBe('Anything At All');
    }
  });

  it('falls back to the stored name when no dictionary is mounted', () => {
    expect(providerDisplayName(providerlessDictionary, builtInRow('tencent_cos')))
      .toBe('Built-in Tencent Cloud COS');
  });
});

function BuiltInNameProbe({ kinds }: { kinds: string[] }) {
  const { t } = useTranslation();
  return (
    <ul>
      {kinds.map((kind) => (
        <li key={kind}>{providerDisplayName(t, builtInRow(kind))}</li>
      ))}
    </ul>
  );
}

function readNames(locale: 'zh-CN' | 'en-US', kinds: string[]): string[] {
  render(
    <LanguageProvider defaultLanguage={locale} resolveHostLanguage={() => locale}>
      <BuiltInNameProbe kinds={kinds} />
    </LanguageProvider>,
  );
  const names = screen.getAllByRole('listitem').map((item) => item.textContent ?? '');
  cleanup();
  return names;
}

describe('built-in provider name catalog', () => {
  const kinds = Object.keys(BUILTIN_PROVIDER_DISPLAY_NAMES);

  it('resolves every built-in name in both languages', () => {
    // 这一条同时是字典覆盖率的守卫：少一条 builtInProviderName.<kind>，这里拿到
    // 的就是原始 key 或英文名，而不是本地化名称。
    const zh = readNames('zh-CN', kinds);
    const en = readNames('en-US', kinds);

    expect(en).toEqual(kinds.map((kind) => BUILTIN_PROVIDER_DISPLAY_NAMES[kind]));
    zh.forEach((name, index) => {
      expect(name, `${kinds[index]} has no Chinese built-in name`).not.toBe(
        BUILTIN_PROVIDER_DISPLAY_NAMES[kinds[index]],
      );
      expect(name, `${kinds[index]} rendered a raw dictionary key`).not.toContain('builtInProviderName.');
    });
  });
});
