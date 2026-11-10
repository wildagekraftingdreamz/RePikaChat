#!/usr/bin/env node
/**
 * End-to-end checks against a running RePikaChat instance.
 *
 *   node scripts/smoke-test.mjs
 *   REPIKACHAT_BASE=http://127.0.0.1:7719 node scripts/smoke-test.mjs
 *
 * The test resets the instance first, so point it at a spare save/port
 * (REPIKACHAT_PORT=7720 REPIKACHAT_DATA=... ) rather than your real memories.
 */

const BASE = process.env.REPIKACHAT_BASE || 'http://127.0.0.1:7719';

let passed = 0;
let failed = 0;

function check(name, condition, detail = '') {
  if (condition) {
    passed += 1;
    console.log(`  ok   ${name}`);
  } else {
    failed += 1;
    console.log(`  FAIL ${name}${detail ? ` — ${detail}` : ''}`);
  }
}

let token = '';

async function request(path, { method = 'GET', body, withToken = true } = {}) {
  const headers = {};
  if (withToken && token) headers['x-repikachat-token'] = token;
  if (body !== undefined) headers['content-type'] = 'application/json';
  const res = await fetch(BASE + path, {
    method,
    headers,
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  const text = await res.text();
  let json = null;
  try { json = text ? JSON.parse(text) : null; } catch { json = null; }
  return { status: res.status, text, json, headers: res.headers };
}

async function main() {
  console.log(`RePikaChat smoke test → ${BASE}\n`);

  const index = await request('/', { withToken: false });
  const m = index.text.match(/name="repikachat-token" content="([^"]+)"/);
  token = m ? m[1] : '';
  check('serves the chat page', index.status === 200, `status ${index.status}`);
  check('page carries a session token', token.length > 8);
  check('page has no unsubstituted token', !index.text.includes('__SESSION_TOKEN__'));

  check('app.css served', (await request('/app.css', { withToken: false })).status === 200);
  check('app.js served', (await request('/app.js', { withToken: false })).status === 200);

  const unauth = await request('/api/chat', { method: 'POST', body: { text: 'hi' }, withToken: false });
  check('chat without a token is refused', unauth.status === 403, `status ${unauth.status}`);

  const empty = await request('/api/chat', { method: 'POST', body: { text: '   ' } });
  check('empty message is a 400', empty.status === 400, `status ${empty.status}`);

  const long = await request('/api/chat', { method: 'POST', body: { text: 'x'.repeat(1201) } });
  check('over-long message is a 400', long.status === 400, `status ${long.status}`);

  const reset = await request('/api/reset', { method: 'POST' });
  check('reset returns a fresh view', reset.status === 200 && reset.json && reset.json.onboarded === false);

  const onboard = await request('/api/companion', {
    method: 'POST',
    body: { name: 'Nova', pronouns: 'they/them', mode: 'friend', warmth: 80, playfulness: 40, curiosity: 70 },
  });
  check('companion is created', onboard.status === 200 && onboard.json.companion.name === 'Nova');
  check('first run seeds a greeting', onboard.json.history.length === 1 && onboard.json.history[0].role === 'bot');

  const learn = await request('/api/chat', {
    method: 'POST',
    body: { text: 'my name is Sam, and I love rainy nights with old synths' },
  });
  check('chat answers', learn.status === 200 && learn.json.reply.length > 0);
  check('chat reports the engine used', ['local', 'model'].includes(learn.json.source));
  const learnedKinds = (learn.json.learned || []).map((x) => x.kind);
  check('learns the name mid-sentence', learnedKinds.includes('name'), JSON.stringify(learnedKinds));
  check('learns the like', learnedKinds.includes('like'), JSON.stringify(learnedKinds));
  check('memory is exposed to the UI', learn.json.state.memories.some((x) => x.value === 'Sam'));

  const recall = await request('/api/chat', { method: 'POST', body: { text: 'what do you know about me' } });
  check('recall lists the stored name', recall.json.reply.includes('Name: Sam'), recall.json.reply.slice(0, 80));
  check('recall is answered locally', recall.json.source === 'local');

  const before = recall.json.state.memories.length;
  const forget = await request('/api/chat', { method: 'POST', body: { text: 'forget rain' } });
  check('forget removes one memory', forget.json.state.memories.length === before - 1);

  const target = forget.json.state.memories[0];
  if (target) {
    const del = await request(`/api/memory/${encodeURIComponent(target.id)}`, { method: 'DELETE' });
    check('delete removes exactly that memory', del.json.memories.every((m2) => m2.id !== target.id));
  } else {
    check('delete removes exactly that memory', true, 'no memory left to delete');
  }

  await request('/api/chat', { method: 'POST', body: { text: 'my name is Ada' } });
  const preClear = await request('/api/state');
  check(
    'a fact is learned before clearing',
    preClear.json.memories.some((m) => m.value === 'Ada'),
    JSON.stringify(preClear.json.memories.map((m) => m.value)),
  );
  const cleared = await request('/api/history/clear', { method: 'POST' });
  check(
    'clear history empties the transcript',
    cleared.status === 200 && cleared.json.history.length === 0,
    cleared.status === 200 ? `${cleared.json.history.length} messages left` : `status ${cleared.status}`,
  );
  check(
    'clear history keeps the learned memories',
    cleared.json.memories.length === preClear.json.memories.length &&
      cleared.json.memories.some((m) => m.value === 'Ada'),
    `${preClear.json.memories.length} → ${cleared.json.memories.length}`,
  );
  check(
    'clear history keeps the companion and settings',
    cleared.json.companion.name === preClear.json.companion.name &&
      cleared.json.companion.level === preClear.json.companion.level &&
      cleared.json.settings.model === preClear.json.settings.model,
    `${cleared.json.companion.name} level ${cleared.json.companion.level}`,
  );

  const remote = await request('/api/settings', {
    method: 'POST',
    body: { provider: 'ollama', endpoint: 'http://192.168.0.10:11434', model: 'llama3.2' },
  });
  check('remote model endpoint is refused', remote.status === 400, `status ${remote.status}`);

  const tls = await request('/api/settings', {
    method: 'POST',
    body: { provider: 'openai', endpoint: 'https://127.0.0.1:1234', model: 'x' },
  });
  check('tls endpoint is refused', tls.status === 400, `status ${tls.status}`);

  const catalogRes = await request('/api/catalog', { withToken: false });
  check('open model catalog is served', catalogRes.status === 200 && catalogRes.json.count >= 20);
  check('catalog credits its inspiration', Boolean(catalogRes.json.attribution && catalogRes.json.attribution.url));
  const chatEntries = catalogRes.json.entries.filter((e) => e.chat_capable);
  const otherEntries = catalogRes.json.entries.filter((e) => !e.chat_capable);
  check('catalog has chat models', chatEntries.length >= 5);
  check('catalog has non-chat models', otherEntries.length >= 5);
  check('non-chat models carry no engine tag', otherEntries.every((e) => !e.model_tag && e.engine === 'none'));

  const hardware = catalogRes.json.hardware;
  check('catalog reports local hardware', typeof hardware.total_ram_mb === 'number' && typeof hardware.budget_mb === 'number');
  check('hardware summary is readable', /RAM/.test(hardware.summary || ''), hardware.summary);
  check(
    'every model is sized and graded',
    catalogRes.json.entries.every((e) => typeof e.estimate_mb === 'number' && ['good', 'tight', 'heavy', 'unknown'].includes(e.fit)),
  );
  const pick = catalogRes.json.recommendation;
  check('catalog recommends a chat model', Boolean(pick && pick.id));
  const pickedEntry = catalogRes.json.entries.find((e) => e.id === pick.id);
  check('the recommendation is a chat model that fits', Boolean(pickedEntry && pickedEntry.chat_capable && pickedEntry.fit !== 'heavy'));

  const disk = catalogRes.json.disk;
  check('catalog reports on-disk models', Boolean(disk && typeof disk.checked === 'boolean' && typeof disk.models === 'number'), JSON.stringify(disk));
  check(
    'chat models carry an on-disk flag',
    chatEntries.every((e) => e.installed === true || e.installed === false || e.installed === null),
  );
  check('non-chat models have no on-disk flag', otherEntries.every((e) => e.installed === null));
  if (disk.checked) {
    check(
      'every chat model carries a verdict once the runtime answered',
      chatEntries.every((e) => typeof e.installed === 'boolean'),
    );
  }

  const useWrong = await request('/api/catalog/use', { method: 'POST', body: { id: otherEntries[0].id } });
  check('catalog refuses a non-chat model', useWrong.status === 400, `status ${useWrong.status}`);

  const useChat = await request('/api/catalog/use', { method: 'POST', body: { id: chatEntries[0].id } });
  check(
    'catalog switches the chat engine',
    useChat.status === 200 && useChat.json.state.brain.model === chatEntries[0].model_tag,
    useChat.status === 200 ? useChat.json.state.brain.model : `status ${useChat.status}`,
  );
  check(
    'catalog switch explains the pull step',
    useChat.json.note.includes('ollama pull') || useChat.json.note.includes('already on disk'),
    useChat.json.note,
  );

  const sizedKnown = await request('/api/model/size', { method: 'POST', body: { model: 'phi4:14b' } });
  check(
    'model sizing uses the catalog entry',
    sizedKnown.status === 200 && sizedKnown.json.estimate_mb === 9600 && sizedKnown.json.matched_id === 'phi4-14b',
    JSON.stringify(sizedKnown.json),
  );
  check('model sizing quotes this machine budget', typeof sizedKnown.json.budget_mb === 'number' && sizedKnown.json.budget_mb > 0);
  const sizedTag = await request('/api/model/size', { method: 'POST', body: { model: 'some-model:16b' } });
  check(
    'model sizing reads parameters from the tag',
    sizedTag.status === 200 && sizedTag.json.estimate_mb === 10800 && sizedTag.json.matched_id === null,
    JSON.stringify(sizedTag.json),
  );
  const sizedUnknown = await request('/api/model/size', { method: 'POST', body: { model: 'mystery-model' } });
  check(
    'unknown names are not given a size',
    sizedUnknown.status === 200 && sizedUnknown.json.estimate_mb === null && sizedUnknown.json.fit === 'unknown',
    JSON.stringify(sizedUnknown.json),
  );

  const bareProbe = await request('/api/model/probe', {
    method: 'POST',
    body: { provider: 'ollama', endpoint: 'http://127.0.0.1:11434', model: '' },
  });
  check(
    'listing on-disk models needs no model name',
    bareProbe.status === 200 && !/model name/i.test(bareProbe.json.error || ''),
    bareProbe.json.error || `ok=${bareProbe.json.ok}`,
  );

  const off = await request('/api/settings', {
    method: 'POST',
    body: { provider: 'off', endpoint: '', model: '', timeout_ms: 20000, temperature: 0.8 },
  });
  check('built-in engine settings save', off.status === 200 && off.json.brain.active === false);

  const profile = await request('/api/public', { withToken: false });
  check('public profile is JSON', profile.status === 200 && profile.json.name === 'RePikaChat');
  check('profile lists endpoints', Array.isArray(profile.json.endpoints) && profile.json.endpoints.length > 5);
  check('profile never invents a support email', profile.json.support_email === null);
  check('profile labels missing values', Array.isArray(profile.json.gaps) && profile.json.gaps.length > 0);

  const docs = (profile.json.docs || []).filter((d) => d.available);
  check('published docs are listed', docs.length >= 5, `${docs.length} docs`);
  for (const doc of docs) {
    const res = await request(doc.path, { withToken: false });
    check(`doc ${doc.path} responds 200`, res.status === 200, `status ${res.status}`);
  }

  const page = await request('/profile', { withToken: false });
  check('profile page renders', page.status === 200 && page.text.includes('Public docs'));
  check('profile page handles missing fields', page.text.includes('Not published'));
  check('profile page is factual about Replika', page.text.includes('Luka, Inc.'));
  check('profile.css served', (await request('/profile.css', { withToken: false })).status === 200);

  const openapi = await request('/openapi.json', { withToken: false });
  check('openapi.json parses', openapi.status === 200 && openapi.json.openapi === '3.1.0');
  check('openapi covers chat', Boolean(openapi.json.paths['/api/chat']));

  const llms = await request('/llms.txt', { withToken: false });
  check('llms.txt is text', llms.status === 200 && llms.text.startsWith('# RePikaChat'));
  check('robots.txt served', (await request('/robots.txt', { withToken: false })).status === 200);

  const sec = await request('/security.txt', { withToken: false });
  check('security.txt has an expiry', sec.status === 200 && /Expires: 20\d\d-/.test(sec.text));
  check('well-known security.txt served', (await request('/.well-known/security.txt', { withToken: false })).status === 200);

  const health = await request('/healthz', { withToken: false });
  check('healthz reports status', health.status === 200 && health.json.status === 'ok');

  const readme = await request('/docs/README.md', { withToken: false });
  check('README is served as markdown', readme.status === 200 && readme.text.includes('# RePikaChat'));

  const exportRes = await request('/api/export', { withToken: false });
  check('export downloads the save', exportRes.status === 200 && exportRes.json && exportRes.json.companion);

  const finalReset = await request('/api/reset', { method: 'POST' });
  check('cleanup reset works', finalReset.status === 200);

  console.log(`\n${passed} passed, ${failed} failed`);
  process.exit(failed === 0 ? 0 : 1);
}

main().catch((e) => {
  console.error('\nSmoke test could not run:', e.message);
  process.exit(1);
});
