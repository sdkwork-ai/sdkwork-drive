/**
 * Rewrites extensionless relative ESM specifiers in compiled output to carry an
 * explicit `.js` extension.
 *
 * Why this exists: the Electron host is authored as ESM TypeScript with
 * bundler-style extensionless imports (`from './clipboard'`), which is what the
 * workspace's `moduleResolution: "bundler"` allows. `tsc` does not rewrite
 * relative specifiers on emit, and Node's ESM loader — which `electron .` uses
 * to load `main` — requires fully-specified paths. Without this pass the app
 * crashes at startup with `ERR_MODULE_NOT_FOUND` before any window appears.
 *
 * Only relative specifiers are touched; bare package specifiers (`electron`,
 * `node:fs`) and already-extensioned paths are left alone.
 *
 * Run: node ./scripts/fix-esm-specifiers.mjs <output-dir>
 */

import { readdirSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';

const targetDir = process.argv[2];
if (!targetDir) {
  // eslint-disable-next-line no-console
  console.error('[sdkwork-drive-pc-electron] usage: fix-esm-specifiers.mjs <output-dir>');
  process.exit(1);
}

const root = resolve(targetDir);

/** Matches `from '<specifier>'` and `import('<specifier>')` for relative paths. */
const SPECIFIER_PATTERN = /(from\s+|import\()(['"])(\.{1,2}\/[^'"]+)\2/g;

function isAlreadyResolved(specifier) {
  // Leave anything with a file extension (or a query/hash) untouched.
  return /\.[a-z0-9]+($|[?#])/i.test(specifier);
}

function rewriteSource(source) {
  let changed = false;
  const rewritten = source.replace(
    SPECIFIER_PATTERN,
    (match, prefix, quote, specifier) => {
      if (isAlreadyResolved(specifier)) {
        return match;
      }
      changed = true;
      return `${prefix}${quote}${specifier}.js${quote}`;
    },
  );
  return { rewritten, changed };
}

function walk(dir) {
  const files = [];
  for (const entry of readdirSync(dir)) {
    const absolute = join(dir, entry);
    if (statSync(absolute).isDirectory()) {
      files.push(...walk(absolute));
    } else if (entry.endsWith('.js')) {
      files.push(absolute);
    }
  }
  return files;
}

let rewrittenCount = 0;
for (const file of walk(root)) {
  const source = readFileSync(file, 'utf8');
  const { rewritten, changed } = rewriteSource(source);
  if (changed) {
    writeFileSync(file, rewritten);
    rewrittenCount += 1;
  }
}

// eslint-disable-next-line no-console
console.log(
  `[sdkwork-drive-pc-electron] ESM specifiers normalized in ${rewrittenCount} file(s) under ${root}`,
);
