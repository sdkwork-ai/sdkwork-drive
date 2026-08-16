import { ConfirmDialog as UiConfirmDialog } from '@sdkwork/ui-pc-react';
import { useTranslation } from '../hooks/useTranslation';

interface ConfirmDialogProps {
  busy?: boolean;
  confirmLabel?: string;
  message: string;
  onCancel: () => void;
  onConfirm: () => void;
  open: boolean;
  title: string;
  variant?: 'danger' | 'default';
}

export function ConfirmDialog({ busy = false, confirmLabel, message, onCancel, onConfirm, open, title, variant = 'default' }: ConfirmDialogProps) {
  const { t } = useTranslation();
  return (
    <UiConfirmDialog
      cancelLabel={t('cancel')}
      closeOnConfirm={false}
      confirmLabel={confirmLabel}
      confirmLoading={busy}
      description={message}
      onConfirm={onConfirm}
      onOpenChange={(nextOpen) => { if (!nextOpen && !busy) onCancel(); }}
      open={open}
      title={title}
      tone={variant === 'danger' ? 'danger' : 'warning'}
    />
  );
}
