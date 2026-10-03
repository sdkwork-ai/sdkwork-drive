/* @vitest-environment jsdom */

import { describe, expect, it } from 'vitest';
import { isPayloadTooLargeError } from '../src/utils/mutationError';
import {
  MAX_OBJECT_CONTENT_BYTES,
  objectContentRequestBytes,
} from 'sdkwork-drive-pc-commons';

/**
 * 413 的识别与体积换算。
 *
 * 网关用明文 413 拒绝超限请求体，框架把它归一化成 `Payload too large`（code 41301）：
 * 报文里既没有"对象超限"的业务细节，也没有"请求体超限"的说明。客户端的职责是认出这一类
 * 失败，并用"网线体积 ≈ 文件的 1.37 倍"解释清楚，而不是把厂商/框架的原文照抄给用户。
 */
describe('isPayloadTooLargeError', () => {
  it('recognizes the shapes a gateway 413 can arrive in', () => {
    // Problem+json 归一化后的样子（title/detail 都是 "Payload too large"）。
    expect(
      isPayloadTooLargeError({
        code: 41301,
        detail: 'Payload too large',
        status: 413,
        title: 'Payload too large',
      }),
    ).toBe(true);
    // 只有 HTTP 状态、没有业务字段时也要认出来。
    expect(isPayloadTooLargeError({ status: 413 })).toBe(true);
    expect(isPayloadTooLargeError({ response: { status: 413 } })).toBe(true);
    // SDK 客户端把报文塞进 Error 的情况。
    expect(isPayloadTooLargeError(new Error('Payload too large'))).toBe(true);
  });

  it('does not swallow unrelated failures', () => {
    expect(isPayloadTooLargeError(new Error('NoSuchBucket'))).toBe(false);
    expect(isPayloadTooLargeError({ code: 40401, status: 404 })).toBe(false);
    expect(isPayloadTooLargeError(undefined)).toBe(false);
  });
});

describe('objectContentRequestBytes', () => {
  it('accounts for base64 inflation plus the JSON envelope', () => {
    // 8 MiB → base64 约 11.18 MB，加信封后仍应落在 11.2 MB 量级。
    const wire = objectContentRequestBytes(MAX_OBJECT_CONTENT_BYTES);
    expect(wire).toBeGreaterThan(MAX_OBJECT_CONTENT_BYTES * 1.33);
    expect(wire).toBeLessThan(MAX_OBJECT_CONTENT_BYTES * 1.4);
    // 空对象（目录占位）也有信封开销，不能算出 0。
    expect(objectContentRequestBytes(0)).toBeGreaterThan(0);
  });
});
