import { describe, expect, it } from 'vitest';
import { escapeBootstrapHtml, renderBootstrapFailureMarkup } from './renderBootstrapFailure';

describe('renderBootstrapFailure', () => {
  it('escapes HTML in bootstrap failure markup', () => {
    const markup = renderBootstrapFailureMarkup(
      '<script>alert(1)</script>',
      'Desc & "quoted"',
      "Detail <img src=x onerror=alert(1)>'",
      'Reload & go',
    );

    expect(markup).not.toContain('<script>');
    expect(markup).toContain('&lt;script&gt;');
    expect(markup).toContain('Desc &amp; &quot;quoted&quot;');
    expect(markup).toContain('&lt;img');
    expect(markup).toContain('Reload &amp; go');
  });

  it('escapeBootstrapHtml neutralizes common XSS vectors', () => {
    expect(escapeBootstrapHtml('<>&"\'')).toBe('&lt;&gt;&amp;&quot;&#39;');
  });
});
