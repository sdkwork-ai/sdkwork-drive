export function escapeBootstrapHtml(value: string): string {
  return value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

export function renderBootstrapFailureMarkup(
  title: string,
  description: string,
  detail: string,
  reloadLabel: string,
): string {
  const safeTitle = escapeBootstrapHtml(title);
  const safeDescription = escapeBootstrapHtml(description);
  const safeDetail = escapeBootstrapHtml(detail);
  const safeReload = escapeBootstrapHtml(reloadLabel);
  return `
      <div style="font-family: system-ui, sans-serif; max-width: 32rem; margin: 4rem auto; padding: 1.5rem; color: #111;">
        <h1 style="font-size: 1.25rem; margin: 0 0 0.75rem;">${safeTitle}</h1>
        <p style="margin: 0 0 1rem; line-height: 1.5; color: #444;">${safeDescription}</p>
        <pre style="white-space: pre-wrap; background: #f6f6f6; padding: 0.75rem; border-radius: 0.5rem; font-size: 0.85rem;">${safeDetail}</pre>
        <button type="button" onclick="window.location.reload()" style="margin-top: 1rem; padding: 0.5rem 1rem; cursor: pointer;">${safeReload}</button>
      </div>`;
}
