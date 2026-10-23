import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';
import test from 'node:test';

const script = readFileSync(new URL('../../src/commands/desktop_zoom.js', import.meta.url), 'utf8');

function press(key, options = {}, mac = false) {
  const messages = [];
  let handler;
  runInNewContext(script, { window: {
    __ORX_MAC_TITLEBAR__: mac,
    addEventListener: (type, listener, capture) => {
      assert.equal(type, 'keydown');
      assert.equal(capture, true);
      handler = listener;
    },
    ipc: { postMessage: (message) => messages.push(message) },
  } });
  let prevented = false;
  let stopped = false;
  handler({ key, ctrlKey: true, ...options,
    preventDefault() { prevented = true; },
    stopImmediatePropagation() { stopped = true; },
  });
  return { messages, prevented, stopped };
}

test('Ctrl zoom shortcuts consume the key and forward exactly one native command', () => {
  for (const [key, action] of [['=', 'in'], ['+', 'in'], ['-', 'out'], ['0', 'reset']]) {
    assert.deepEqual(press(key), { messages: [`page-zoom:${action}`], prevented: true, stopped: true });
  }
  assert.deepEqual(press('+', { shiftKey: true }).messages, ['page-zoom:in']);
});

test('unrelated input and macOS menu accelerators are left alone', () => {
  const ignored = { messages: [], prevented: false, stopped: false };
  for (const options of [{ ctrlKey: false }, { altKey: true }, { metaKey: true }, { isComposing: true }]) {
    assert.deepEqual(press('=', options), ignored);
  }
  assert.deepEqual(press('0', { shiftKey: true }), ignored);
  assert.deepEqual(press('x'), ignored);
  assert.deepEqual(press('=', {}, true), ignored);
});
