/* RePikaChat — chat client. Plain DOM, no framework, no CDN. */
(() => {
  'use strict';

  const TOKEN = (document.querySelector('meta[name="repikachat-token"]') || {}).content || '';
  const $ = (id) => document.getElementById(id);

  const KIND_LABEL = {
    name: 'Name',
    like: 'Loves',
    dislike: 'Avoids',
    job: 'Work',
    home: 'Home',
    goal: 'Goal',
    feeling: 'Feeling',
    pet: 'Pet',
    fact: 'Fact',
  };

  const SUGGESTIONS = [
    'my name is Sam',
    'I love rainy nights and old synths',
    'I work as a night-shift nurse',
    'I feel restless today',
    'what do you know about me',
  ];

  let state = null;
  let busy = false;
  let lastLearned = [];
  let engineWarn = null;
  let personaRequired = false;
  let catalog = null;
  let catalogCategory = 'all';
  let catalogQuery = '';
  let catalogPickId = null;

  /* ── transport ─────────────────────────────────────────── */

  async function api(path, options = {}) {
    const opts = { method: options.method || 'GET', headers: { ...(options.headers || {}) } };
    if (opts.method !== 'GET') opts.headers['x-repikachat-token'] = TOKEN;
    if (options.body !== undefined) {
      opts.headers['content-type'] = 'application/json';
      opts.body = JSON.stringify(options.body);
    }
    const res = await fetch(path, opts);
    const text = await res.text();
    let data = null;
    try { data = text ? JSON.parse(text) : null; } catch { data = null; }
    if (!res.ok) throw new Error((data && data.error) || `Request failed (${res.status})`);
    return data;
  }

  const esc = (s) => String(s == null ? '' : s)
    .replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;').replace(/'/g, '&#39;');

  const fmtTime = (ms) => {
    try { return new Date(ms || Date.now()).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }); }
    catch { return ''; }
  };

  let toastTimer = null;
  function toast(message, kind) {
    const el = $('toast');
    el.textContent = message;
    el.className = 'toast' + (kind ? ' ' + kind : '');
    el.hidden = false;
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => { el.hidden = true; }, 5200);
  }

  /* ── rendering ─────────────────────────────────────────── */

  function setPose(pose) {
    const orb = $('orb');
    const next = pose || 'idle';
    orb.dataset.pose = 'idle';
    // Force a reflow so the same pose can replay its animation.
    void orb.offsetWidth;
    orb.dataset.pose = next;
  }

  function render(view, opts = {}) {
    state = view;
    const c = view.companion;

    $('compName').textContent = c.name;
    $('compSub').textContent = `${c.pronouns} · ${c.mode}`;
    $('moodChip').textContent = view.mood;
    $('levelChip').textContent = `lvl ${c.level}`;
    $('moodNote').textContent = view.mood_note;
    $('memoryName').textContent = c.name;

    const pct = Math.min(100, Math.round((c.xp / Math.max(1, view.xp_next)) * 100));
    $('xpBar').style.width = pct + '%';
    $('xpLabel').textContent = `${c.xp} / ${view.xp_next} xp`;

    $('traits').innerHTML = [
      ['Warmth', c.traits.warmth],
      ['Playfulness', c.traits.playfulness],
      ['Curiosity', c.traits.curiosity],
    ].map(([label, value]) => `
      <div class="trait">
        <div class="trait-top"><span>${label}</span><b>${value}</b></div>
        <div class="track"><span style="width:${value}%"></span></div>
      </div>`).join('');

    $('gMessages').textContent = view.messages;
    $('gAffinity').textContent = `${c.affinity}/100`;
    $('gMemories').textContent = view.memories.length;
    $('gLevel').textContent = c.level;
    $('dataPath').textContent = 'Data: ' + view.data_dir;

    renderEngine(view);
    renderTranscript(opts.keepLearned ? lastLearned : []);
    if (!opts.keepLearned) lastLearned = [];
    renderMemories();
    renderTopics();

    const lastBot = [...view.history].reverse().find((m) => m.role !== 'user');
    setPose(lastBot ? lastBot.pose : 'idle');
  }

  function renderEngine(view) {
    const pill = $('enginePill');
    const brain = view.brain;
    pill.classList.remove('model', 'warn');
    if (engineWarn) {
      pill.textContent = 'model unreachable';
      pill.classList.add('warn');
      pill.title = engineWarn;
    } else if (brain.active) {
      pill.textContent = `model · ${brain.model}`;
      pill.classList.add('model');
      pill.title = `${brain.provider} at ${brain.endpoint}`;
    } else {
      pill.textContent = 'built-in engine';
      pill.title = 'Deterministic local replies; configure a local model in settings.';
    }
  }

  function renderTranscript(learned) {
    const box = $('transcript');
    box.innerHTML = '';

    if (!state.history.length) {
      const empty = document.createElement('div');
      empty.className = 'empty';
      empty.innerHTML = '<h3>Nothing said yet.</h3><p>Start with something small — what you love, ' +
        'what your day was, or what you are building. Every fact you share becomes a memory you can see.</p>';
      box.appendChild(empty);
    } else {
      state.history.forEach((m) => box.appendChild(bubble(m)));
    }

    if (learned && learned.length) {
      const bots = box.querySelectorAll('.msg.bot');
      const last = bots[bots.length - 1];
      if (last) {
        const wrap = document.createElement('div');
        wrap.className = 'learned';
        learned.forEach((m) => {
          const chip = document.createElement('span');
          chip.textContent = `remembered · ${(KIND_LABEL[m.kind] || 'Fact').toLowerCase()}: ${m.value}`;
          wrap.appendChild(chip);
        });
        last.querySelector('.bubble-wrap').appendChild(wrap);
      }
    }

    scrollToEnd();
  }

  function bubble(m) {
    const isBot = m.role !== 'user';
    const el = document.createElement('div');
    el.className = 'msg ' + (isBot ? 'bot' : 'user');

    const avatar = document.createElement('div');
    avatar.className = 'avatar';

    const wrap = document.createElement('div');
    wrap.className = 'bubble-wrap';

    const who = document.createElement('div');
    who.className = 'who';
    who.innerHTML = isBot
      ? `<b>${esc(state.companion.name)}</b><span>${fmtTime(m.at_ms)}</span>`
      : `<b>You</b><span>${fmtTime(m.at_ms)}</span>`;

    const text = document.createElement('div');
    text.className = 'bubble';
    text.textContent = m.text;

    wrap.append(who, text);
    el.append(avatar, wrap);
    return el;
  }

  function scrollToEnd() {
    const box = $('transcript');
    box.scrollTop = box.scrollHeight;
  }

  function renderMemories() {
    const box = $('memoryList');
    const q = ($('memoryFilter').value || '').trim().toLowerCase();
    box.innerHTML = '';

    const all = state.memories.slice().reverse();
    const shown = all.filter((m) => !q
      || m.value.toLowerCase().includes(q)
      || m.kind.includes(q)
      || (KIND_LABEL[m.kind] || '').toLowerCase().includes(q)
      || (m.snippet || '').toLowerCase().includes(q));

    $('memoryCount').textContent = String(state.memories.length);

    if (!state.memories.length) {
      box.innerHTML = '<p class="hint">Nothing learned yet. Tell it your name, something you love, or what you do — it will keep the fact and show it here.</p>';
      return;
    }
    if (!shown.length) {
      box.innerHTML = '<p class="hint">No memory matches that filter.</p>';
      return;
    }

    shown.forEach((m) => {
      const row = document.createElement('div');
      row.className = 'memory';
      row.innerHTML = `
        <div class="memory-top">
          <span class="memory-kind">${esc(KIND_LABEL[m.kind] || 'Fact')}</span>
          <button class="del" type="button" title="Forget this fact" aria-label="Forget ${esc(m.value)}">×</button>
        </div>
        <div class="memory-value">${esc(m.value)}</div>
        <div class="memory-foot">
          <div class="conf"><span style="width:${Math.min(100, m.confidence)}%"></span></div>
          <span class="conf-num">${m.confidence}%</span>
        </div>
        ${m.snippet ? `<p class="src">“${esc(m.snippet)}”</p>` : ''}`;
      row.querySelector('.del').addEventListener('click', () => forgetMemory(m.id));
      box.appendChild(row);
    });
  }

  function renderTopics() {
    const box = $('topicList');
    if (!state.topics.length) {
      box.innerHTML = '<p class="hint">Words that come up more than once appear here. They are counted locally, never sent anywhere.</p>';
      return;
    }
    box.innerHTML = state.topics.map((t) =>
      `<span class="topic">${esc(t.word)}<b>${t.count}</b></span>`).join('');
  }

  /* ── actions ───────────────────────────────────────────── */

  async function forgetMemory(id) {
    try {
      const view = await api(`/api/memory/${encodeURIComponent(id)}`, { method: 'DELETE' });
      lastLearned = [];
      render(view);
      toast('Forgotten. It is out of the save file.');
    } catch (e) {
      toast(e.message, 'bad');
    }
  }

  function setTyping(on) {
    $('typing').hidden = !on;
    if (on) {
      $('typingNote').textContent = state && state.brain.active ? 'asking the local model…' : 'thinking…';
    }
  }

  function setBusy(on) {
    busy = on;
    $('sendBtn').disabled = on;
    $('input').disabled = on;
  }

  async function send(text) {
    const value = String(text || '').trim();
    if (!value || busy) return;

    setBusy(true);
    setTyping(true);
    $('input').value = '';
    autoGrow();

    const holder = document.createElement('div');
    holder.className = 'msg user pending';
    holder.innerHTML = '<div class="avatar"></div>';
    const wrap = document.createElement('div');
    wrap.className = 'bubble-wrap';
    const who = document.createElement('div');
    who.className = 'who';
    who.innerHTML = `<b>You</b><span>${fmtTime(Date.now())}</span>`;
    const bub = document.createElement('div');
    bub.className = 'bubble';
    bub.textContent = value;
    wrap.append(who, bub);
    holder.appendChild(wrap);
    const box = $('transcript');
    const empty = box.querySelector('.empty');
    if (empty) empty.remove();
    box.appendChild(holder);
    scrollToEnd();

    try {
      const out = await api('/api/chat', { method: 'POST', body: { text: value } });
      lastLearned = out.learned || [];
      engineWarn = out.model_error || null;
      render(out.state, { keepLearned: true });
      if (out.model_error) {
        toast('Local model unavailable — the built-in engine answered. ' + out.model_error, 'warn');
      }
    } catch (e) {
      holder.remove();
      $('input').value = value;
      autoGrow();
      toast(e.message, 'bad');
    } finally {
      setTyping(false);
      setBusy(false);
      $('input').focus();
    }
  }

  function autoGrow() {
    const input = $('input');
    input.style.height = 'auto';
    input.style.height = Math.min(160, input.scrollHeight) + 'px';
  }

  /* ── open generative AI catalog ────────────────────────── */

  const CATEGORY_HINT = {
    image: 'Image generation — not a chat engine',
    video: 'Video generation — not a chat engine',
    audio: 'Audio generation — not a chat engine',
    embedding: 'Embeddings — not a chat engine',
  };

  async function openCatalog() {
    $('catalogModal').hidden = false;
    if (!catalog) {
      $('catalogList').innerHTML = '<p class="hint">Loading the open model catalog…</p>';
      try {
        catalog = await api('/api/catalog');
        $('catalogCredit').textContent =
          `${catalog.attribution.name} · ${catalog.attribution.note}`;
        renderHardware();
      } catch (e) {
        $('catalogList').innerHTML = `<p class="hint">Could not load the catalog: ${esc(e.message)}</p>`;
        return;
      }
    }
    renderCatalog();
  }

  function closeCatalog() {
    $('catalogModal').hidden = true;
  }

  function renderHardware() {
    $('catalogHardware').textContent = catalog.hardware.summary;
    const pick = catalog.recommendation;
    const alreadyInUse = pick && state && state.settings && state.settings.model === pick.model_tag;
    catalogPickId = pick && !alreadyInUse ? pick.id : null;
    $('catalogPick').hidden = !pick || alreadyInUse;
    if (pick) {
      $('catalogPick').innerHTML = `<span>${esc(pick.note)}</span>
        <button class="btn small primary" type="button" data-use="${esc(pick.id)}">Use it</button>`;
    }
  }

  function renderCatalogFilters() {
    const fits = catalog.entries.filter((e) => e.fit === 'good' || e.fit === 'tight').length;
    const cats = [
      { id: 'all', label: 'All', count: catalog.entries.length },
      ...catalog.categories,
      { id: 'fits', label: 'Fits here', count: fits },
    ];
    // Only offered once the local runtime confirmed what it has; otherwise the
    // count would just be zero everywhere and mean nothing.
    if (catalog.disk && catalog.disk.checked) {
      cats.push({
        id: 'disk',
        label: 'On disk',
        count: catalog.entries.filter((e) => e.installed === true).length,
      });
    }
    $('catalogFilters').innerHTML = cats.map((c) =>
      `<button type="button" class="filter${c.id === catalogCategory ? ' active' : ''}" data-cat="${esc(c.id)}">${esc(c.label)}<b>${c.count}</b></button>`).join('');
  }

  function renderCatalog() {
    if (!catalog) return;
    if (catalogCategory === 'disk' && !(catalog.disk && catalog.disk.checked)) {
      catalogCategory = 'all';
    }
    renderCatalogFilters();

    const q = catalogQuery.trim().toLowerCase();
    const shown = catalog.entries.filter((e) => {
      const inCategory = catalogCategory === 'all'
        || (catalogCategory === 'fits'
          ? e.fit === 'good' || e.fit === 'tight'
          : catalogCategory === 'disk'
            ? e.installed === true
            : e.category === catalogCategory);
      return inCategory
        && (!q || `${e.name} ${e.org} ${e.license} ${e.summary} ${e.parameters}`.toLowerCase().includes(q));
    });

    if (!shown.length) {
      $('catalogList').innerHTML = '<p class="hint">No model matches that filter.</p>';
      return;
    }

    const activeModel = state && state.settings ? state.settings.model : '';
    $('catalogList').innerHTML = shown.map((e) => {
      const inUse = e.chat_capable && e.model_tag === activeModel;
      const action = e.chat_capable
        ? `<button class="btn small${inUse ? '' : ' primary'}" type="button" data-use="${esc(e.id)}"${inUse ? ' disabled' : ''}>${inUse ? 'In use' : (e.installed === true ? 'Use for chat' : 'Pull & use')}</button>`
        : '<span class="model-locked">reference only</span>';
      const size = e.estimate_mb ? ` · ~${(e.estimate_mb / 1024).toFixed(1)} GB` : '';
      const disk = e.installed === true
        ? '<span class="disk" title="Already pulled in Ollama">on disk</span>'
        : e.installed === false
          ? '<span class="disk pull" title="Not pulled yet">needs pull</span>'
          : '';
      const recommended = catalogPickId === e.id;
      return `<article class="model${recommended ? ' recommended' : ''}">
        <div class="model-head">
          <div><h4>${esc(e.name)}${recommended ? '<span class="model-star">recommended</span>' : ''}</h4><p class="model-org">${esc(e.org)}</p></div>
          <div class="model-tags">
            <span class="fit ${esc(e.fit)}">${esc(e.fit_label)}${size}</span>
            ${disk}
            <span class="model-cat ${esc(e.category)}">${esc(e.category)}</span>
          </div>
        </div>
        <p class="model-summary">${esc(e.summary)}</p>
        <div class="model-meta">
          <span>${esc(e.parameters)}</span>
          <span>${esc(e.license)}</span>
          <span>${esc(e.runtime)}</span>
        </div>
        <div class="model-foot">
          <a href="${esc(e.source)}" target="_blank" rel="noopener">Model card ↗</a>
          ${action}
        </div>
        ${e.chat_capable ? '' : `<p class="model-note">${esc(CATEGORY_HINT[e.category] || 'Not a chat engine')}</p>`}
      </article>`;
    }).join('');
  }

  async function useCatalogModel(id) {
    try {
      const out = await api('/api/catalog/use', { method: 'POST', body: { id } });
      engineWarn = null;
      lastLearned = [];
      catalog = null; // re-measure on the next open, now that the pick changed
      catalogPickId = null;
      render(out.state);
      closeCatalog();
      toast(out.note, 'warn');
    } catch (e) {
      toast(e.message, 'bad');
    }
  }

  /* ── settings ──────────────────────────────────────────── */

  function openSettings() {
    const s = state.settings;
    $('modelProvider').value = s.provider;
    $('modelEndpoint').value = s.endpoint;
    $('modelName').value = s.model;
    $('modelTimeout').value = s.timeout_ms;
    $('modelTemp').value = s.temperature;
    $('probeResult').hidden = true;
    $('dataPath').textContent = 'Data: ' + state.data_dir;
    $('settingsModal').hidden = false;
    loadInstalledModels();
    updateModelFit();
  }

  function closeSettings() {
    $('settingsModal').hidden = true;
  }

  function settingsPayload() {
    return {
      provider: $('modelProvider').value,
      endpoint: $('modelEndpoint').value.trim(),
      model: $('modelName').value.trim(),
      timeout_ms: Number($('modelTimeout').value) || 20000,
      temperature: Number($('modelTemp').value),
    };
  }

  async function saveSettings() {
    const btn = $('settingsSave');
    btn.disabled = true;
    try {
      const view = await api('/api/settings', { method: 'POST', body: settingsPayload() });
      engineWarn = null;
      lastLearned = [];
      render(view);
      closeSettings();
      toast(view.brain.active ? `Saved. Replies can come from ${view.brain.model}.` : 'Saved. Using the built-in engine.');
    } catch (e) {
      toast(e.message, 'bad');
    } finally {
      btn.disabled = false;
    }
  }

  let sizeTimer = null;

  /// Warns when the chosen model needs more memory than this machine can give
  /// it, using the same sizing as the open models catalog.
  function updateModelFit() {
    const box = $('modelFit');
    const model = $('modelName').value.trim();
    box.className = 'hint model-fit';
    box.textContent = '';
    if ($('modelProvider').value === 'off' || !model) return;
    clearTimeout(sizeTimer);
    sizeTimer = setTimeout(async () => {
      try {
        const out = await api('/api/model/size', { method: 'POST', body: { model } });
        if ($('modelName').value.trim() !== model) return;
        const gb = (mb) => (mb / 1024).toFixed(1);
        if (out.fit === 'heavy') {
          box.className = 'hint model-fit bad';
          box.textContent = `About ${gb(out.estimate_mb)} GB, but this machine can give a model roughly ${gb(out.budget_mb)} GB. Expect heavy offload, very slow replies or a failed load.`;
        } else if (out.fit === 'tight') {
          box.className = 'hint model-fit warn';
          box.textContent = `About ${gb(out.estimate_mb)} GB — close to the ${gb(out.budget_mb)} GB this machine can give a model, so the context may have to stay short.`;
        } else if (out.fit === 'good') {
          box.textContent = `About ${gb(out.estimate_mb)} GB — comfortably inside the ${gb(out.budget_mb)} GB this machine can give a model.`;
        } else {
          box.textContent = 'No size estimate for this name; check its model card before pulling it.';
        }
      } catch (e) {
        box.textContent = '';
      }
    }, 350);
  }

  /// Fills the model field's suggestions with the tags the local runtime
  /// already has, so the name never has to be typed from memory.
  function renderModelPicks(list, note) {
    $('modelList').innerHTML = list.map((m) => `<option value="${esc(m)}"></option>`).join('');
    const current = $('modelName').value.trim();
    $('modelPicks').innerHTML = list.map((m) =>
      `<button class="pick${m === current ? ' active' : ''}" type="button" data-model="${esc(m)}">${esc(m)}</button>`).join('');
    $('modelHint').textContent = note || '';
  }

  function pickModel(name) {
    $('modelName').value = name;
    const list = [...document.querySelectorAll('#modelPicks .pick')].map((b) => b.dataset.model);
    renderModelPicks(list, `${name} selected. Save to start using it.`);
    updateModelFit();
  }

  async function loadInstalledModels() {
    if ($('modelProvider').value === 'off') {
      renderModelPicks([], 'The built-in engine needs no model file. Switch to Ollama to pick one from disk.');
      return;
    }
    $('modelHint').textContent = 'Looking for models on disk…';
    try {
      const out = await api('/api/model/probe', { method: 'POST', body: settingsPayload() });
      const list = out.models || [];
      if (!out.ok) {
        renderModelPicks([], out.error || 'The endpoint did not answer.');
        return;
      }
      const current = $('modelName').value.trim();
      const note = list.length
        ? `${list.length} already on disk — click one, or type to narrow the list.`
          + (current && !list.includes(current) ? ` “${current}” is not on disk yet.` : '')
        : 'Nothing on disk yet — pull a tag with `ollama pull <tag>`, or use the open models catalog.';
      renderModelPicks(list, note);
    } catch (e) {
      renderModelPicks([], e.message);
    }
  }

  async function probeModel() {
    const btn = $('probeBtn');
    const box = $('probeResult');
    btn.disabled = true;
    box.hidden = false;
    box.className = 'probe-result';
    box.textContent = 'Testing the local endpoint…';
    try {
      const out = await api('/api/model/probe', { method: 'POST', body: settingsPayload() });
      if (out.ok) {
        const list = out.models || [];
        box.className = 'probe-result ok';
        box.textContent = list.length
          ? `Connected. ${list.length} models on disk.`
          : 'Connected, but the server listed no models.';
        const current = $('modelName').value.trim();
        const pick = current && list.includes(current) ? current : list[0];
        if (pick) $('modelName').value = pick;
        renderModelPicks(list, list.length
          ? (pick ? `${pick} selected. Save to start using it.` : '')
          : 'Nothing on disk yet — pull a tag with `ollama pull <tag>`.');
        updateModelFit();
      } else {
        box.className = 'probe-result bad';
        box.textContent = out.error || 'The endpoint did not answer.';
      }
    } catch (e) {
      box.className = 'probe-result bad';
      box.textContent = e.message;
    } finally {
      btn.disabled = false;
    }
  }

  async function clearHistory() {
    const name = state.companion.name;
    const sure = window.confirm(
      `Clear the conversation with ${name}?\n\nThe transcript is erased. What ${name} has learned about you, its personality, level and your model settings stay. This cannot be undone.`,
    );
    if (!sure) return;
    try {
      const view = await api('/api/history/clear', { method: 'POST' });
      engineWarn = null;
      lastLearned = [];
      render(view);
      toast(`Transcript cleared. ${name} still remembers what you shared.`);
    } catch (e) {
      toast(e.message, 'bad');
    }
  }

  async function resetEverything() {
    const sure = window.confirm('Erase every memory, the transcript and the persona? Model settings are kept. This cannot be undone.');
    if (!sure) return;
    try {
      const view = await api('/api/reset', { method: 'POST' });
      engineWarn = null;
      lastLearned = [];
      render(view);
      closeSettings();
      openPersona(true);
      toast('Erased. A fresh save file is on disk.');
    } catch (e) {
      toast(e.message, 'bad');
    }
  }

  async function quitApp() {
    if (!window.confirm('Stop RePikaChat? The window will keep showing this page until you close it.')) return;
    try {
      await api('/api/quit', { method: 'POST' });
      $('stopped').hidden = false;
    } catch (e) {
      toast(e.message, 'bad');
    }
  }

  /* ── persona ───────────────────────────────────────────── */

  function openPersona(required) {
    personaRequired = !!required;
    const c = state.companion;
    $('personaTitle').textContent = required ? 'Create your companion' : 'Companion persona';
    $('personaName').value = c.name;
    $('personaPronouns').value = c.pronouns;
    $('personaMode').value = c.mode;
    $('personaWarmth').value = c.traits.warmth;
    $('personaPlay').value = c.traits.playfulness;
    $('personaCuriosity').value = c.traits.curiosity;
    syncSliders();
    $('personaClose').hidden = !!required;
    $('personaModal').hidden = false;
    setTimeout(() => $('personaName').focus(), 30);
  }

  function closePersona() {
    if (personaRequired) return;
    $('personaModal').hidden = true;
  }

  function syncSliders() {
    $('warmVal').textContent = $('personaWarmth').value;
    $('playVal').textContent = $('personaPlay').value;
    $('curVal').textContent = $('personaCuriosity').value;
  }

  async function savePersona() {
    const btn = $('personaSave');
    btn.disabled = true;
    try {
      const view = await api('/api/companion', {
        method: 'POST',
        body: {
          name: $('personaName').value.trim(),
          pronouns: $('personaPronouns').value,
          mode: $('personaMode').value,
          warmth: Number($('personaWarmth').value),
          playfulness: Number($('personaPlay').value),
          curiosity: Number($('personaCuriosity').value),
        },
      });
      personaRequired = false;
      lastLearned = [];
      render(view);
      $('personaModal').hidden = true;
      toast(`Saved. ${view.companion.name} is listening.`);
      $('input').focus();
    } catch (e) {
      toast(e.message, 'bad');
    } finally {
      btn.disabled = false;
    }
  }

  /* ── wiring ────────────────────────────────────────────── */

  function bind() {
    $('composer').addEventListener('submit', (e) => {
      e.preventDefault();
      send($('input').value);
    });

    $('input').addEventListener('input', autoGrow);
    $('input').addEventListener('keydown', (e) => {
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        send($('input').value);
      }
    });

    $('catalogBtn').addEventListener('click', openCatalog);
    $('catalogClose').addEventListener('click', closeCatalog);
    $('catalogModal').addEventListener('click', (e) => {
      if (e.target === $('catalogModal')) closeCatalog();
    });
    $('catalogSearch').addEventListener('input', () => {
      catalogQuery = $('catalogSearch').value;
      renderCatalog();
    });
    $('catalogFilters').addEventListener('click', (e) => {
      const btn = e.target.closest('button[data-cat]');
      if (!btn) return;
      catalogCategory = btn.dataset.cat;
      renderCatalog();
    });
    $('catalogList').addEventListener('click', (e) => {
      const btn = e.target.closest('button[data-use]');
      if (btn) useCatalogModel(btn.dataset.use);
    });
    $('catalogPick').addEventListener('click', (e) => {
      const btn = e.target.closest('button[data-use]');
      if (btn) useCatalogModel(btn.dataset.use);
    });

    $('settingsBtn').addEventListener('click', openSettings);
    $('settingsClose').addEventListener('click', closeSettings);
    $('settingsModal').addEventListener('click', (e) => {
      if (e.target === $('settingsModal')) closeSettings();
    });
    $('settingsSave').addEventListener('click', saveSettings);
    $('probeBtn').addEventListener('click', probeModel);
    $('modelPicks').addEventListener('click', (e) => {
      const btn = e.target.closest('button[data-model]');
      if (btn) pickModel(btn.dataset.model);
    });
    $('modelName').addEventListener('input', updateModelFit);
    $('clearHistoryBtn').addEventListener('click', clearHistory);
    $('resetBtn').addEventListener('click', resetEverything);
    $('quitBtn').addEventListener('click', quitApp);
    $('exportBtn').addEventListener('click', () => { window.location.href = '/api/export'; });
    $('modelProvider').addEventListener('change', () => {
      const provider = $('modelProvider').value;
      if (provider === 'ollama' && !$('modelEndpoint').value.trim()) {
        $('modelEndpoint').value = 'http://127.0.0.1:11434';
      }
      if (provider === 'openai' && (!$('modelEndpoint').value.trim() || $('modelEndpoint').value.includes(':11434'))) {
        $('modelEndpoint').value = 'http://127.0.0.1:1234';
      }
      loadInstalledModels();
      updateModelFit();
    });
    $('modelEndpoint').addEventListener('change', () => {
      loadInstalledModels();
      updateModelFit();
    });

    $('editBtn').addEventListener('click', () => openPersona(false));
    $('personaClose').addEventListener('click', closePersona);
    $('personaModal').addEventListener('click', (e) => {
      if (e.target === $('personaModal')) closePersona();
    });
    $('personaSave').addEventListener('click', savePersona);
    ['personaWarmth', 'personaPlay', 'personaCuriosity'].forEach((id) =>
      $(id).addEventListener('input', syncSliders));

    $('memoryFilter').addEventListener('input', renderMemories);
    $('memoryToggle').addEventListener('click', () => $('side').classList.toggle('open'));

    document.addEventListener('keydown', (e) => {
      if (e.key !== 'Escape') return;
      if (!$('settingsModal').hidden) closeSettings();
      if (!$('catalogModal').hidden) closeCatalog();
      if (!$('personaModal').hidden) closePersona();
    });

    const suggestions = $('suggestions');
    SUGGESTIONS.forEach((text) => {
      const b = document.createElement('button');
      b.type = 'button';
      b.textContent = text;
      b.addEventListener('click', () => {
        $('input').value = text;
        autoGrow();
        $('input').focus();
      });
      suggestions.appendChild(b);
    });
  }

  async function init() {
    bind();
    try {
      const view = await api('/api/state');
      render(view);
      if (!view.onboarded) openPersona(true);
    } catch (e) {
      toast('Cannot reach RePikaChat: ' + e.message, 'bad');
      return;
    }
    $('input').focus();
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }
})();
