/* @vitest-environment jsdom */

import { act, renderHook } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { useUnsavedChangesGuard } from '../src/hooks/useUnsavedChangesGuard';

describe('useUnsavedChangesGuard', () => {
  it('closes straight away when nothing is dirty', () => {
    const onClose = vi.fn();
    const { result } = renderHook(() => useUnsavedChangesGuard({ dirty: false, onClose }));

    act(() => result.current.requestClose());

    expect(onClose).toHaveBeenCalledTimes(1);
    expect(result.current.pending).toBe(false);
  });

  it('holds the close request back while the editor is dirty', () => {
    const onClose = vi.fn();
    const { result } = renderHook(() => useUnsavedChangesGuard({ dirty: true, onClose }));

    act(() => result.current.requestClose());

    // 未保存的草稿不能被一次误触抹掉：先挂起，等用户确认。
    expect(onClose).not.toHaveBeenCalled();
    expect(result.current.pending).toBe(true);
  });

  it('closes after the discard is confirmed', () => {
    const onClose = vi.fn();
    const { result } = renderHook(() => useUnsavedChangesGuard({ dirty: true, onClose }));

    act(() => result.current.requestClose());
    act(() => result.current.confirmDiscard());

    expect(onClose).toHaveBeenCalledTimes(1);
    expect(result.current.pending).toBe(false);
  });

  it('returns to editing when the discard is cancelled', () => {
    const onClose = vi.fn();
    const { result } = renderHook(() => useUnsavedChangesGuard({ dirty: true, onClose }));

    act(() => result.current.requestClose());
    act(() => result.current.cancelDiscard());

    expect(onClose).not.toHaveBeenCalled();
    expect(result.current.pending).toBe(false);

    // 取消之后脏状态若已消失（用户保存了），下一次关闭直接通过。
    act(() => result.current.requestClose());
    expect(onClose).not.toHaveBeenCalled();
  });
});
