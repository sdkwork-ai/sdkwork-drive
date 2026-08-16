export function confirmDriveSensitiveAction(
  message: string,
  confirmationRequired: boolean,
): boolean {
  if (!confirmationRequired) {
    return true;
  }
  if (typeof window === 'undefined') {
    return true;
  }
  return window.confirm(message);
}
