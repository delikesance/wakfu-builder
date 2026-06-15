// Wakfu Builder — Frontend Application
// Communique avec le backend Tauri via @tauri-apps/api (v2)

import { invoke } from '@tauri-apps/api/core';

// ===== State =====
let allItems = [];
let buildResult = null;
let currentLevel = 200;

// ===== DOM Refs =====
const statusBadge = document.getElementById('status-badge');
const configPanel = document.getElementById('config-panel');
const resultsArea = document.getElementById('results-area');

// ===== Status Helpers =====
function setStatus(text, type = 'loading') {
  if (!statusBadge) return;
  statusBadge.textContent = text;
  statusBadge.className = `badge badge-${type}`;
}

// ===== Config Panel Rendering =====
function renderConfigPanel() {
  if (!configPanel) return;

  configPanel.innerHTML = `
    <div class="card">
      <h3>Configuration</h3>

      <div class="form-group">
        <label>Niveau cible : <span id="level-value">${currentLevel}</span></label>
        <input type="range" id="level" min="1" max="245" value="${currentLevel}">
      </div>

      <div class="form-group">
        <label>Rôle</label>
        <select id="role">
          <option value="dps">DPS</option>
          <option value="tank">Tank</option>
          <option value="support">Support</option>
        </select>
      </div>

      <div class="form-group">
        <label>Mode</label>
        <select id="mode">
          <option value="solo">Solo</option>
          <option value="team">Team</option>
        </select>
      </div>

      <div class="form-group">
        <label>Portée</label>
        <select id="range">
          <option value="melee">Mêlée</option>
          <option value="distance">Distance</option>
          <option value="hybrid" selected>Hybride</option>
        </select>
      </div>

      <div class="form-group">
        <label>Élément</label>
        <select id="element">
          <option value="all" selected>Auto (Best)</option>
          <option value="fire">Feu</option>
          <option value="earth">Terre</option>
          <option value="water">Eau</option>
          <option value="air">Air</option>
        </select>
      </div>

      <details class="advanced">
        <summary>Contraintes avancées</summary>
        <div class="form-group">
          <label>PA minimum</label>
          <input type="number" id="min-ap" min="6" max="12" value="10">
        </div>
        <div class="form-group">
          <label>PM minimum</label>
          <input type="number" id="min-mp" min="3" max="6" value="4">
        </div>
        <div class="form-group">
          <label>Résistance minimum</label>
          <input type="number" id="min-res" min="0" max="200" value="0">
        </div>
      </details>

      <button id="optimize-btn" class="btn-primary" disabled>Optimiser</button>
      <p class="hint" id="config-hint">Chargement des items...</p>
    </div>
  `;

  // Wire up level slider live display
  const levelInput = document.getElementById('level');
  const levelValue = document.getElementById('level-value');

  const updateLevel = () => {
    currentLevel = parseInt(levelInput.value, 10);
    levelValue.textContent = currentLevel;
  };

  levelInput.addEventListener('input', updateLevel);
  levelInput.addEventListener('change', updateLevel);

  // Wire up optimize button
  document.getElementById('optimize-btn').addEventListener('click', runOptimization);
}

// ===== Load Items from Backend =====
async function loadItems() {
  setStatus('Chargement des items...', 'loading');

  // If not running in Tauri, set mock data for dev
  if (!invoke) {
    setStatus('⚠ Mode démo (hors Tauri)', 'ready');
    const hint = document.getElementById('config-hint');
    if (hint) hint.textContent = 'Mode démo — pas de données réelles';
    const btn = document.getElementById('optimize-btn');
    if (btn) btn.disabled = false;
    return;
  }

  try {
    allItems = await invoke('fetch_equipment');
    setStatus(`${allItems.length} items chargés`, 'ready');

    const hint = document.getElementById('config-hint');
    if (hint) hint.textContent = `${allItems.length} items disponibles`;

    const btn = document.getElementById('optimize-btn');
    if (btn) btn.disabled = false;
  } catch (e) {
    console.error('Failed to load items:', e);
    setStatus('Erreur chargement items', 'error');

    const hint = document.getElementById('config-hint');
    if (hint) hint.textContent = `Erreur : ${e}`;
  }
}

// ===== Build Request from Form =====
function getBuildRequest() {
  return {
    level: parseInt(document.getElementById('level').value, 10),
    role: document.getElementById('role').value,
    mode: document.getElementById('mode').value,
    range: document.getElementById('range').value,
    element: document.getElementById('element').value,
    min_ap: parseInt(document.getElementById('min-ap').value, 10) || null,
    min_mp: parseInt(document.getElementById('min-mp').value, 10) || null,
    min_res: parseFloat(document.getElementById('min-res').value) || null,
  };
}

// ===== Get Stat Helper =====
function getStatValue(stats, id, fallback = 0) {
  return stats?.[id] ?? fallback;
}

// ===== Run Optimization =====
async function runOptimization() {
  const btn = document.getElementById('optimize-btn');
  if (!btn) return;

  setStatus('Optimisation en cours...', 'loading');
  btn.disabled = true;

  const request = getBuildRequest();

  try {
    // If not in Tauri, show a mock result
    if (!invoke) {
      buildResult = generateMockResult(request);
      renderResults(request);
      setStatus('Build optimal trouvé !', 'ready');
      btn.disabled = false;
      return;
    }

    buildResult = await invoke('optimize_build', { request, items: allItems });
    renderResults(request);
    setStatus('Build optimal trouvé !', 'ready');
  } catch (e) {
    console.error('Optimization failed:', e);
    setStatus('Erreur optimisation', 'error');

    if (resultsArea) {
      resultsArea.innerHTML = `
        <div class="empty-state error">
          <p>Erreur d'optimisation : ${e}</p>
        </div>
      `;
    }
  } finally {
    btn.disabled = false;
  }
}

// ===== Generate Mock Results (dev mode) =====
function generateMockResult(request) {
  const level = request.level || 200;
  return {
    items: [
      { slot_name: 'Amulette', name: 'Collier du Dévoreur', rarity: 7, level, enchant_name: 'Sagesse', enchant_doubled: false },
      { slot_name: 'Anneau', name: 'Anneau de l\'Ourobouros', rarity: 5, level, enchant_name: 'Puissance', enchant_doubled: true },
      { slot_name: 'Anneau', name: 'Anneau Draconique', rarity: 7, level: level - 10, enchant_name: 'Sagesse', enchant_doubled: false },
      { slot_name: 'Bottes', name: 'Bottes du Printemps', rarity: 7, level, enchant_name: 'Puissance', enchant_doubled: false },
      { slot_name: 'Cape', name: 'Cape du Néant', rarity: 7, level: level - 5, enchant_name: 'Sagesse', enchant_doubled: false },
      { slot_name: 'Ceinture', name: 'Ceinture des Éléments', rarity: 5, level, enchant_name: null, enchant_doubled: false },
      { slot_name: 'Coiffe', name: 'Chapeau du Sinistrofu', rarity: 7, level, enchant_name: 'Puissance', enchant_doubled: false },
      { slot_name: 'Arme 1H', name: 'Épée du Dragon Noir', rarity: 5, level, enchant_name: 'Dommages', enchant_doubled: false },
      { slot_name: 'Bouclier', name: 'Bouclier du Zobal', rarity: 7, level: level - 15, enchant_name: 'Résistance', enchant_doubled: false },
      { slot_name: 'Familier', name: 'Tofu Royal', rarity: 7, level, enchant_name: null, enchant_doubled: false },
    ],
    stats: {
      20: 4500 + level * 15,
      31: 4 + Math.floor(level / 50),
      41: 2 + Math.floor(level / 80),
      122: 800 + level * 4,
      123: 600 + level * 3,
      124: 400 + level * 2,
      125: 500 + level * 3,
      1052: 300 + level * 2,
      180: 150 + level,
      150: 8 + Math.floor(level / 30),
      80: 200 + level,
    },
  };
}

// ===== Render Results =====
function renderResults(request) {
  if (!buildResult || !resultsArea) return;

  const { items, stats } = buildResult;

  // Compute derived stats
  const basePA = 6; // base PA
  const majorPA = request?.level >= 25 ? 1 : 0; // major PA at level 25+
  const totalPA = getStatValue(stats, 31) + basePA + majorPA;
  const baseMP = 3;
  const totalMP = getStatValue(stats, 41) + baseMP;

  const elemMasteries = [
    getStatValue(stats, 122),
    getStatValue(stats, 123),
    getStatValue(stats, 124),
    getStatValue(stats, 125),
  ];
  const maxMastery = Math.max(...elemMasteries);
  const melee = getStatValue(stats, 1052);
  const contactPower = maxMastery + melee;
  const backstab = getStatValue(stats, 180);
  const baseCrit = 3;
  const totalCrit = getStatValue(stats, 150) + baseCrit;
  const totalRes = getStatValue(stats, 80);

  // Slot icon mapping — French names matching backend
  const slotIcons = {
    'Coiffe': '🎩',
    'Amulette': '📿',
    'Épaulières': '🦺',
    'Cape': '🧥',
    'Plastron': '🛡️',
    'Ceinture': '🔗',
    'Anneau': '💍',
    'Bottes': '👢',
    'Arme 2H': '⚔️',
    'Arme 1H': '🗡️',
    'Dague': '🔪',
    'Bouclier': '🛡️',
    'Familier': '🐾',
    'Emblème': '🥚',
    'Autre': '📦',
  };

  const getSlotIcon = (slot) => slotIcons[slot] || '📦';

  const getRarityClass = (rarity) => {
    if (rarity === 7) return 'rarity-epic';
    if (rarity === 5) return 'rarity-relic';
    return '';
  };

  const getRarityLabel = (rarity) => {
    if (rarity === 7) return '[ÉPIQUE]';
    if (rarity === 5) return '[RELIQUE]';
    return '';
  };

  const formatStat = (value) => {
    if (typeof value === 'number') {
      return Number.isInteger(value) ? value.toString() : value.toFixed(1);
    }
    return value ?? '0';
  };

  resultsArea.innerHTML = `
    <div class="card stats-card">
      <h3>📊 Stats Totales</h3>
      <div class="stats-grid">
        <div class="stat-item">
          <span class="stat-label">❤️ Points de Vie</span>
          <span class="stat-value">${formatStat(getStatValue(stats, 20))}</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">⚡ PA</span>
          <span class="stat-value">${totalPA}</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">👟 PM</span>
          <span class="stat-value">${totalMP}</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">🔥 Maîtrise Élém. Max</span>
          <span class="stat-value">${formatStat(maxMastery)}</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">⚔️ Maîtrise Mêlée</span>
          <span class="stat-value">${formatStat(melee)}</span>
        </div>
        <div class="stat-item highlight">
          <span class="stat-label">💥 Puissance Contact</span>
          <span class="stat-value">${formatStat(contactPower)}</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">🔪 Maîtrise Dos</span>
          <span class="stat-value">${formatStat(backstab)}</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">🎯 Coup Critique</span>
          <span class="stat-value">${formatStat(totalCrit)}%</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">🛡️ Résistance</span>
          <span class="stat-value">${formatStat(totalRes)}</span>
        </div>
      </div>
    </div>

    <div class="card items-card">
      <h3>🎒 Équipement Optimal</h3>
      <div class="items-list">
        ${items.map(item => {
          const rarityClass = getRarityClass(item.rarity);
          const rarityLabel = getRarityLabel(item.rarity);
          const slotIcon = getSlotIcon(item.slot_name);
          const enchantHtml = item.enchant_name
            ? `<span class="item-enchant">4x ${item.enchant_name}${item.enchant_doubled ? ' DOUBLÉ' : ''}</span>`
            : '';

          return `
            <div class="item-row ${rarityClass}">
              <span class="item-slot">${slotIcon} ${item.slot_name}</span>
              <span class="item-name">${rarityLabel ? rarityLabel + ' ' : ''}${item.name}</span>
              <span class="item-level">Lvl ${item.level}</span>
              ${enchantHtml}
            </div>
          `;
        }).join('')}
      </div>
    </div>
  `;
}

// ===== Init =====
function init() {
  try {
    renderConfigPanel();
    loadItems();
  } catch (e) {
    console.error('Init failed:', e);
    setStatus('Erreur initialisation', 'error');
    if (configPanel) {
      configPanel.innerHTML = `
        <div class="card">
          <h3>Erreur</h3>
          <p style="color: #e74c3c;">Impossible de charger l'interface : ${e.message || e}</p>
          <pre style="font-size: 0.8em; margin-top: 8px; background: #1a1a2e; padding: 8px; border-radius: 4px; overflow-x: auto;">${e.stack || ''}</pre>
        </div>
      `;
    }
  }
}

if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', init);
} else {
  init();
}
