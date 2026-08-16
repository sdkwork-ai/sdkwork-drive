import { ConfirmDialog } from '@sdkwork/ui-pc-react';

interface OperationsConfirmDialogProps {
  cancelLabel: string;
  confirmLabel: string;
  message: string;
  onCancel: () => void;
  onConfirm: () => void;
  open: boolean;
  title: string;
  variant?: 'danger' | 'default';
}

export function OperationsConfirmDialog({
  cancelLabel,
  confirmLabel,
  message,
  onCancel,
  onConfirm,
  open,
  title,
  variant = 'default',
}: OperationsConfirmDialogProps) {
  return (
    <ConfirmDialog
      cancelLabel={cancelLabel}
      closeOnConfirm={false}
      confirmLabel={confirmLabel}
      description={message}
      onConfirm={onConfirm}
      onOpenChange={(nextOpen) => { if (!nextOpen) onCancel(); }}
      open={open}
      title={title}
      tone={variant === 'danger' ? 'danger' : 'warning'}
    />
  );
}
