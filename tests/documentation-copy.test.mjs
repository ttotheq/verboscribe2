import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(repoRoot, relativePath), 'utf8');
}

test('desktop copy and docs treat shipped recovery controls as implemented', () => {
  const mainTs = read('apps/desktop/src/main.ts');
  const manualQa = read('docs/MANUAL_QA.md');
  const featureList = read('docs/FEATURE_LIST.md');

  assert.match(
    mainTs,
    /Paste-last and retry-failed recovery controls are now shipped\./,
    'prototype-gap copy should acknowledge the shipped recovery controls',
  );
  assert.doesNotMatch(
    manualQa,
    /still calls out intentionally missing controls such as retry-last/i,
    'manual QA should no longer describe retry-last as missing',
  );
  assert.match(
    featureList,
    /- dedicated cancel hotkey works/,
    'feature list should reflect the shipped cancel hotkey',
  );
  assert.match(
    featureList,
    /- dedicated retry-failed hotkey works/,
    'feature list should reflect the shipped retry-failed hotkey',
  );
  assert.doesNotMatch(
    featureList,
    /### Missing compared with the prototype[\s\S]*?- cancel hotkey/,
    'feature list should not list the cancel hotkey as missing once shipped',
  );
});
