/** Canonical transfer job speed/time tokens stored in job state (not for direct UI display). */
export const TRANSFER_SPEED_CONNECTING = 'Connecting...';
export const TRANSFER_SPEED_UPLOADING = 'Uploading...';
export const TRANSFER_SPEED_DOWNLOADING = 'Downloading...';
export const TRANSFER_SPEED_READY = 'Ready';

export const TRANSFER_TIME_CALCULATING = 'Calculating...';
export const TRANSFER_TIME_FINISHING = 'Finishing...';
export const TRANSFER_TIME_FINALIZING = 'Finalizing...';
export const TRANSFER_TIME_WAITING_BACKEND = 'Waiting for backend confirmation';
export const TRANSFER_TIME_AVAILABLE = 'Available';
export const TRANSFER_TIME_SAVE_CANCELLED = 'Save cancelled';

export const TRANSFER_INTERRUPTION_UPLOAD_NATIVE_RETRY =
  'transfer.uploadInterruptedNativeRetry';
export const TRANSFER_INTERRUPTION_UPLOAD_RESELECT =
  'transfer.uploadInterruptedReselect';
export const TRANSFER_INTERRUPTION_TRANSFER_RETRY =
  'transfer.transferInterruptedRetry';

type TranslateFn = (key: string) => string;

export function formatTransferJobSpeedLabel(speed: string, translate: TranslateFn): string {
  switch (speed) {
    case TRANSFER_SPEED_CONNECTING:
      return translate('transfer.connecting');
    case TRANSFER_SPEED_UPLOADING:
      return translate('transfer.uploading');
    case TRANSFER_SPEED_DOWNLOADING:
      return translate('transfer.downloading');
    case TRANSFER_SPEED_READY:
      return translate('downloadManager.ready');
    case '--':
    case '':
      return speed;
    default:
      return speed;
  }
}

export function formatTransferJobTimeRemainingLabel(
  timeRemaining: string,
  translate: TranslateFn,
): string {
  switch (timeRemaining) {
    case TRANSFER_TIME_CALCULATING:
      return translate('transfer.calculating');
    case TRANSFER_TIME_FINISHING:
      return translate('transfer.finishing');
    case TRANSFER_TIME_FINALIZING:
      return translate('transfer.finalizing');
    case TRANSFER_TIME_WAITING_BACKEND:
      return translate('transfer.waitingBackendConfirmation');
    case TRANSFER_TIME_AVAILABLE:
      return translate('transfer.available');
    case TRANSFER_TIME_SAVE_CANCELLED:
      return translate('transfer.saveCancelled');
    case '--':
    case '':
      return timeRemaining;
    default:
      return timeRemaining;
  }
}

export function formatTransferJobProgressDetail(
  job: { status: string; speed: string; timeRemaining: string },
  translate: TranslateFn,
): string {
  if (job.status === 'downloading' || job.status === 'uploading') {
    const speed = formatTransferJobSpeedLabel(job.speed, translate);
    const remaining = formatTransferJobTimeRemainingLabel(job.timeRemaining, translate);
    if (!remaining) {
      return speed;
    }
    if (!speed || speed === '--') {
      return remaining;
    }
    return `${speed} - ${remaining}`;
  }
  if (job.status === 'ready') {
    return translate('downloadManager.ready');
  }
  if (job.status === 'paused') {
    return translate('transfer.paused');
  }
  return '--';
}

export function formatTransferInterruptionMessage(
  message: string | undefined,
  translate: TranslateFn,
): string {
  switch (message) {
    case TRANSFER_INTERRUPTION_UPLOAD_NATIVE_RETRY:
      return translate('transfer.uploadInterruptedNativeRetry');
    case TRANSFER_INTERRUPTION_UPLOAD_RESELECT:
      return translate('transfer.uploadInterruptedReselect');
    case TRANSFER_INTERRUPTION_TRANSFER_RETRY:
      return translate('transfer.transferInterruptedRetry');
    default:
      return message ?? translate('transfer.transferInterruptedRetry');
  }
}
