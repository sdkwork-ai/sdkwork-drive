import { useCallback, useState } from 'react';

export interface UnsavedChangesGuardOptions {
  /** 编辑器报告的脏状态。 */
  dirty: boolean;
  /** 真正关闭的动作。 */
  onClose: () => void;
}

export interface UnsavedChangesGuard {
  /** 取消关闭，回到编辑。 */
  cancelDiscard: () => void;
  /** 确认放弃修改并关闭。 */
  confirmDiscard: () => void;
  /** 是否存在待确认的关闭请求。 */
  pending: boolean;
  /** 请求关闭：干净时直接关闭，有未保存修改时挂起并等待确认。 */
  requestClose: () => void;
}

/**
 * 「带着未保存修改关闭」的守卫。
 *
 * 编辑器里的草稿只存在于内存中：关闭对话框等于丢弃它。所以关闭入口（对话框自带的 X、
 * Esc、工具条里的关闭按钮）都必须先问一句，而不是让一次误触抹掉用户刚写的内容。
 * 判定逻辑单独成 hook，是为了在 Monaco 之外也能被测试覆盖——真要在 jsdom 里敲字，
 * 就得把整个编辑器内核拉进测试环境。
 */
export function useUnsavedChangesGuard({
  dirty,
  onClose,
}: UnsavedChangesGuardOptions): UnsavedChangesGuard {
  const [pending, setPending] = useState(false);

  const requestClose = useCallback(() => {
    if (dirty) {
      setPending(true);
      return;
    }
    onClose();
  }, [dirty, onClose]);

  const confirmDiscard = useCallback(() => {
    setPending(false);
    onClose();
  }, [onClose]);

  const cancelDiscard = useCallback(() => setPending(false), []);

  return { cancelDiscard, confirmDiscard, pending, requestClose };
}
