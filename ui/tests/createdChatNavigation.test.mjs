import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import { runInNewContext } from "node:vm";

const source = readFileSync(new URL("../src/components/ChatPanel.tsx", import.meta.url), "utf8");
const callback = source.match(/const openSpawnedSession = useCallback\(\s*(async \(sessionId: string\) => \{[\s\S]*?\n    \}),/)[1]
  .replace("sessionId: string", "sessionId");

function navigation(getChatSession) {
  const calls = [];
  const open = runInNewContext(`(${callback})`, {
    getChatSession,
    setSessionFilter: (filter) => calls.push(["filter", filter]),
    onActiveSessionChange: (id, options) => calls.push(["navigate", id, options.projectId]),
    showAlert: (message, kind) => calls.push(["alert", message, kind]),
  });
  return { calls, open };
}

test("a created-chat link navigates to its destination project", async () => {
  const { calls, open } = navigation(async (id) => ({ id, projectId: "other-project" }));
  await open("created-chat");
  assert.deepEqual(calls, [["filter", "all"], ["navigate", "created-chat", "other-project"]]);
});

test("a missing created chat reports the error without changing navigation", async () => {
  const { calls, open } = navigation(async () => { throw "Chat unavailable"; });
  await open("deleted-chat");
  assert.deepEqual(calls, [["alert", "Chat unavailable", "error"]]);
});
