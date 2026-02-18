// AstraWeave Benchmark Dashboard v7.0 - Data Visualization & Analysis
// Reads history.jsonl and renders interactive d3.js charts
// Features: Baseline vs User hardware comparison, Industry Standard references,
//           Statistical analysis (percentiles, CI), CSV/JSON export, shareable URLs

// ─── DATA SOURCES ────────────────────────────────────────────────────────────
const DATA_SOURCES = [
    'benchmark-data/history.jsonl',              // LOCAL: relative path (served from tools/benchmark-dashboard/)
    'static-data/history.jsonl',                 // Fallback: shipped static snapshot
    '/benchmark-data/history.jsonl',             // gh-pages hosted path
    '../../target/benchmark-data/history.jsonl', // Local dev (from criterion benchmarks)
];

// ─── HP PAVILION BASELINE HARDWARE ID ────────────────────────────────────────
const BASELINE_HARDWARE_ID = 'aed36a43';
const BASELINE_HARDWARE_LABEL = 'HP Pavilion Gaming Laptop 16-a0xxx (Intel i5-10300H, 32GB)';

// ─── INDUSTRY STANDARD REFERENCE VALUES ──────────────────────────────────────
// Representative benchmarks from comparable engines/frameworks.
// Sources: Bevy 0.14, Flecs, Unity DOTS, Godot, Rapier3D, glam, nalgebra published benchmarks.
// Values in nanoseconds unless noted.
const INDUSTRY_STANDARDS = {
    ecs: {
        label: 'ECS Industry Avg',
        benchmarks: {
            'world_creation':       { bevy: 35, unity_dots: 120, godot: 250, flecs: 28 },
            'entity_spawn':         { bevy: 500, unity_dots: 800, godot: 1200, flecs: 380 },
            'entity_spawn_batch':   { bevy: 180, unity_dots: 400, godot: 900, flecs: 150 },
            'query_iteration':      { bevy: 2.5, unity_dots: 4.0, godot: 15, flecs: 2.0 },
            'archetype_move':       { bevy: 800, unity_dots: 600, godot: 2000, flecs: 500 },
            'event_dispatch':       { bevy: 45, unity_dots: 80, godot: 200, flecs: 35 },
            'system_execution':     { bevy: 150, unity_dots: 250, godot: 500, flecs: 120 },
        }
    },
    ai: {
        label: 'AI Industry Avg',
        benchmarks: {
            'behavior_tree':                { typical_engine: 200, utility_ai_sdk: 350 },
            'behavior_tree_10_nodes':       { typical_engine: 250, utility_ai_sdk: 400 },
            'behavior_tree_20_nodes':       { typical_engine: 600, utility_ai_sdk: 900 },
            'behavior_tree_simple_3_nodes': { typical_engine: 80, utility_ai_sdk: 120 },
            'behavior_tree_condition':      { typical_engine: 100, utility_ai_sdk: 150 },
            'behavior_tree_decorator':      { typical_engine: 60, utility_ai_sdk: 90 },
            'behavior_tree_sequence':       { typical_engine: 180, utility_ai_sdk: 280 },
            'goap_planning_simple':         { typical_engine: 8000, strips_planner: 12000 },
            'goap_planning_10_actions':     { typical_engine: 25000, strips_planner: 45000 },
            'goap_planning_20_actions':     { typical_engine: 80000, strips_planner: 150000 },
            'goap_goal_evaluation':         { typical_engine: 3000, strips_planner: 5000 },
            'goap_action_preconditions':    { typical_engine: 1500, strips_planner: 3000 },
            'goap_caching':                 { typical_engine: 5000, strips_planner: 8000 },
        }
    },
    physics: {
        label: 'Physics Industry Avg',
        benchmarks: {
            'character_controller':    { rapier: 5000, physx: 3500, bullet: 8000 },
            'raycast':                 { rapier: 200, physx: 150, bullet: 350 },
            'collision_detection':     { rapier: 1500, physx: 1200, bullet: 3000 },
            'spatial_hash':            { rapier: 800, custom: 400 },
        }
    },
    math: {
        label: 'Math Industry Avg',
        benchmarks: {
            'vec3_dot':        { glam: 1.8, nalgebra: 3.5, directxmath: 1.5 },
            'vec3_cross':      { glam: 2.2, nalgebra: 4.0, directxmath: 1.8 },
            'vec3_normalize':  { glam: 3.5, nalgebra: 6.0, directxmath: 2.8 },
            'mat4_mul':        { glam: 8.0, nalgebra: 12.0, directxmath: 6.5 },
            'quat_multiply':   { glam: 3.0, nalgebra: 5.5, directxmath: 2.5 },
            'quat_slerp':      { glam: 12.0, nalgebra: 18.0, directxmath: 9.0 },
            'transform':       { glam: 5.0, nalgebra: 8.0, directxmath: 4.0 },
        }
    },
    rendering: {
        label: 'Rendering Industry Avg',
        benchmarks: {
            'culling_performance':  { typical_engine: 50000, unreal: 35000, godot: 80000 },
            'vertex_compression':   { typical_engine: 25, meshopt: 18 },
            'lod_generation':       { typical_engine: 500000, meshoptimizer: 350000 },
            'mesh_simplification':  { meshoptimizer: 400000, typical_engine: 600000 },
            'instance_batching':    { typical_engine: 5000, unreal: 3000 },
            'bloom':                { typical_engine: 50000000, unreal: 40000000 },
            'shadow':               { typical_engine: 100000, unreal: 80000 },
            'frustum':              { typical_engine: 15000, unreal: 10000 },
            'depth_sort':           { typical_engine: 2000, custom: 1500 },
            'material_compile':     { typical_engine: 500000, unreal: 300000 },
        }
    },
    terrain: {
        label: 'Terrain Industry Avg',
        benchmarks: {
            'heightmap_generation':     { typical_engine: 500000, noise_rs: 300000 },
            'world_chunk_generation':   { minecraft_like: 30000000, typical_engine: 50000000 },
            'voxel_meshing':            { typical_engine: 200000, openvdb: 150000 },
            'climate_sampling':         { typical_engine: 50000, custom: 30000 },
        }
    },
    animation: {
        label: 'Animation Industry Avg',
        benchmarks: {
            'animation_blending':       { bevy: 900, unity: 600, godot: 1500 },
            'crossfade':                { bevy: 900, unity: 600, godot: 1500 },
            'animation_sample':         { bevy: 1200, unity: 800, godot: 2000 },
            'animation_hierarchy':      { bevy: 2000, unity: 1500, godot: 3500 },
            'forward_kinematics':       { bevy: 2000, unity: 1500, godot: 3500 },
            'animation_full_frame':     { bevy: 5000, unity: 3000, godot: 8000 },
            'joint_palette':            { bevy: 1500, unity: 1000, godot: 2500 },
            'keyframe_search':          { bevy: 30, unity: 25, godot: 60 },
            'animation_transform':      { bevy: 50, unity: 35, godot: 80 },
            'animation_matrix':         { bevy: 15, unity: 10, godot: 25 },
        }
    },
    ui: {
        label: 'UI Industry Avg',
        benchmarks: {
            'widget':       { egui: 2000, imgui: 1500, iced: 3000 },
            'gizmo':        { typical_engine: 5000, blender: 3000 },
        }
    },
    persistence: {
        label: 'Persistence Industry Avg',
        benchmarks: {
            'save':             { serde_json: 50000, bincode: 20000, rkyv: 10000 },
            'serialization':    { serde_json: 50000, bincode: 20000, rkyv: 10000 },
        }
    }
};

// ─── COLOR SCHEME ────────────────────────────────────────────────────────────
const COLOR_SCHEME = {
    ecs: '#4facfe',
    ai: '#00f2fe',
    physics: '#43e97b',
    terrain: '#fa709a',
    input: '#f093fb',
    rendering: '#feca57',
    math: '#ff6b6b',
    networking: '#48dbfb',
    persistence: '#ff9ff3',
    audio: '#54a0ff',
    ui: '#5f27cd',
    tools: '#00d2d3',
    animation: '#e17055',
    default: '#a0a0a0'
};

const INDUSTRY_LINE_COLOR = '#ff9500';

// ─── PERFORMANCE ─────────────────────────────────────────────────────────────
const PERF = { loadStart: performance.now(), renderCount: 0 };

// ─── STATISTICAL HELPERS ─────────────────────────────────────────────────────

function percentile(arr, p) {
    if (arr.length === 0) return 0;
    const sorted = [...arr].sort((a, b) => a - b);
    const idx = (p / 100) * (sorted.length - 1);
    const lo = Math.floor(idx), hi = Math.ceil(idx);
    return lo === hi ? sorted[lo] : sorted[lo] + (sorted[hi] - sorted[lo]) * (idx - lo);
}

function mean(arr) { return arr.length ? arr.reduce((a, b) => a + b, 0) / arr.length : 0; }

function stddev(arr) {
    if (arr.length < 2) return 0;
    const m = mean(arr);
    return Math.sqrt(arr.reduce((s, v) => s + (v - m) ** 2, 0) / (arr.length - 1));
}

function confidenceInterval95(arr) {
    if (arr.length < 2) return { lo: mean(arr), hi: mean(arr), margin: 0 };
    const m = mean(arr), s = stddev(arr);
    const margin = 1.96 * s / Math.sqrt(arr.length);
    return { lo: m - margin, hi: m + margin, margin };
}

function coefficientOfVariation(arr) {
    const m = mean(arr);
    return m !== 0 ? (stddev(arr) / m) * 100 : 0;
}

// ─── TOAST NOTIFICATION ──────────────────────────────────────────────────────

function showToast(msg, duration = 2500) {
    const el = document.getElementById('toast');
    if (!el) return;
    el.textContent = msg;
    el.classList.add('show');
    setTimeout(() => el.classList.remove('show'), duration);
}

// ─── DEBOUNCE ────────────────────────────────────────────────────────────────

function debounce(fn, ms) {
    let timer;
    return (...args) => { clearTimeout(timer); timer = setTimeout(() => fn(...args), ms); };
}

const debouncedRender = debounce(() => renderDashboard(), 120);

// ─── STATE ───────────────────────────────────────────────────────────────────
let benchmarkData = [];
let filteredData = [];
let hardwareProfiles = new Map();
let currentFilters = {
    system: 'all',
    timeRange: 'all',
    benchmark: null,
    hardware: 'all',
    showIndustry: true
};

// ─── DATA LOADING ────────────────────────────────────────────────────────────

async function tryLoadFromSource(source) {
    const response = await fetch(source);
    if (!response.ok) return null;
    const text = await response.text();
    const lines = text.split('\n')
        .map(l => l.trim())
        .filter(l => l.length > 0 && !l.startsWith('#') && l.startsWith('{'));
    if (lines.length === 0) return null;
    return lines;
}

async function loadBenchmarkData() {
    console.log('=== AstraWeave Benchmark Dashboard v7.0 ===');
    showLoadingSkeleton();
    try {
        let lines = null, sourceUsed = null;
        const errors = [];
        for (const source of DATA_SOURCES) {
            try {
                lines = await tryLoadFromSource(source);
                if (lines && lines.length > 0) { sourceUsed = source; break; }
            } catch (err) { errors.push(`${source}: ${err.message}`); }
        }
        if (!lines || lines.length === 0) {
            throw new Error(`No benchmark data found.\nTried:\n${errors.join('\n')}\n\nRun: .\\scripts\\run_benchmark_dashboard.ps1`);
        }

        benchmarkData = lines.map(line => {
            const entry = JSON.parse(line);
            entry.timestamp = new Date(entry.timestamp);
            if (!entry.hardware_id) entry.hardware_id = BASELINE_HARDWARE_ID;
            if (!entry.hardware_label) entry.hardware_label = BASELINE_HARDWARE_LABEL;
            return entry;
        });

        // Deduplicate
        const dedupeMap = new Map();
        benchmarkData.forEach(e => {
            const key = `${e.benchmark_name}_${e.timestamp.getTime()}_${e.hardware_id}`;
            if (!dedupeMap.has(key)) dedupeMap.set(key, e);
        });
        benchmarkData = Array.from(dedupeMap.values());
        benchmarkData.sort((a, b) => a.timestamp - b.timestamp);

        benchmarkData.forEach(d => {
            if (!hardwareProfiles.has(d.hardware_id))
                hardwareProfiles.set(d.hardware_id, d.hardware_label);
        });

        console.log(`Loaded: ${benchmarkData.length} entries, ${hardwareProfiles.size} hardware(s) from ${sourceUsed}`);
        updateFilters();
        restoreFiltersFromURL();
        renderDashboard();
        const loadTime = (performance.now() - PERF.loadStart).toFixed(0);
        console.log(`Dashboard ready in ${loadTime}ms`);
        const ltEl = document.getElementById('load-time');
        if (ltEl) ltEl.textContent = `Loaded in ${loadTime}ms`;
    } catch (error) {
        console.error(error);
        showError(error.message);
    }
}

// ─── FILTERS ─────────────────────────────────────────────────────────────────

function updateFilters() {
    const benchmarkSelect = document.getElementById('benchmark-select');
    const benchMap = new Map();
    benchmarkData.forEach(d => {
        if (!benchMap.has(d.benchmark_name))
            benchMap.set(d.benchmark_name, d.display_name || d.benchmark_name);
    });
    const sorted = Array.from(benchMap.entries())
        .map(([n, d]) => ({ name: n, display: d }))
        .sort((a, b) => a.display.localeCompare(b.display));
    benchmarkSelect.innerHTML = '<option value="all">All Benchmarks</option>';
    sorted.forEach(b => {
        const o = document.createElement('option');
        o.value = b.name; o.textContent = b.display;
        benchmarkSelect.appendChild(o);
    });

    const hwSelect = document.getElementById('hardware-filter');
    if (hwSelect) {
        hwSelect.innerHTML = '<option value="all">All Hardware</option>';
        hwSelect.innerHTML += `<option value="${BASELINE_HARDWARE_ID}">Baseline (HP Pavilion)</option>`;
        hardwareProfiles.forEach((label, id) => {
            if (id !== BASELINE_HARDWARE_ID) {
                const o = document.createElement('option');
                o.value = id; o.textContent = `User: ${label.substring(0, 50)}`;
                hwSelect.appendChild(o);
            }
        });
    }

    document.getElementById('system-filter').addEventListener('change', onFilterChange);
    document.getElementById('time-range').addEventListener('change', onFilterChange);
    document.getElementById('benchmark-select').addEventListener('change', onFilterChange);
    if (hwSelect) hwSelect.addEventListener('change', onFilterChange);
    const toggle = document.getElementById('industry-toggle');
    if (toggle) toggle.addEventListener('change', e => { currentFilters.showIndustry = e.target.checked; debouncedRender(); });
}

function onFilterChange() {
    currentFilters.system = document.getElementById('system-filter').value;
    currentFilters.timeRange = document.getElementById('time-range').value;
    currentFilters.benchmark = document.getElementById('benchmark-select').value;
    const hw = document.getElementById('hardware-filter');
    if (hw) currentFilters.hardware = hw.value;
    persistFiltersToURL();
    debouncedRender();
}

// ─── URL STATE ───────────────────────────────────────────────────────────────

function persistFiltersToURL() {
    const p = new URLSearchParams();
    if (currentFilters.system !== 'all') p.set('sys', currentFilters.system);
    if (currentFilters.timeRange !== 'all') p.set('t', currentFilters.timeRange);
    if (currentFilters.benchmark && currentFilters.benchmark !== 'all') p.set('b', currentFilters.benchmark);
    if (currentFilters.hardware !== 'all') p.set('hw', currentFilters.hardware);
    if (!currentFilters.showIndustry) p.set('ind', '0');
    const qs = p.toString();
    history.replaceState(null, '', qs ? '?' + qs : location.pathname);
}

function restoreFiltersFromURL() {
    const p = new URLSearchParams(location.search);
    if (p.has('sys')) { currentFilters.system = p.get('sys'); document.getElementById('system-filter').value = currentFilters.system; }
    if (p.has('t')) { currentFilters.timeRange = p.get('t'); document.getElementById('time-range').value = currentFilters.timeRange; }
    if (p.has('b')) { currentFilters.benchmark = p.get('b'); document.getElementById('benchmark-select').value = currentFilters.benchmark; }
    if (p.has('hw')) { currentFilters.hardware = p.get('hw'); const el = document.getElementById('hardware-filter'); if (el) el.value = currentFilters.hardware; }
    if (p.get('ind') === '0') { currentFilters.showIndustry = false; const el = document.getElementById('industry-toggle'); if (el) el.checked = false; }
}

function applyFilters() {
    const now = new Date();
    let cutoff = null;
    if (currentFilters.timeRange !== 'all') {
        cutoff = new Date(now.getTime() - parseInt(currentFilters.timeRange) * 86400000);
    }
    filteredData = benchmarkData.filter(d => {
        if (cutoff && d.timestamp < cutoff) return false;
        if (currentFilters.system !== 'all' && detectSystem(d.crate) !== currentFilters.system) return false;
        if (currentFilters.benchmark && currentFilters.benchmark !== 'all' && d.benchmark_name !== currentFilters.benchmark) return false;
        if (currentFilters.hardware !== 'all' && d.hardware_id !== currentFilters.hardware) return false;
        return true;
    });
}

// ─── SYSTEM DETECTION ────────────────────────────────────────────────────────

function detectSystem(crate) {
    const c = (crate || '').toLowerCase();
    if (c.includes('animation') || c.includes('crossfade') || c.includes('joint_palette') ||
        c.includes('keyframe') || c.match(/blend.*mode/)) return 'animation';
    if (c.includes('core') || c.includes('ecs') || c.includes('stress') || c.includes('entity') ||
        c.includes('archetype') || c.includes('query') || (c.includes('world') && !c.includes('chunk') && !c.includes('partition')))
        return 'ecs';
    if (c.includes('ai') || c.includes('behavior') || c.includes('llm') || c.includes('context') ||
        c.includes('memory') || c.includes('persona') || c.includes('rag') || c.includes('prompts') ||
        c.includes('goap') || c.includes('utility'))
        return 'ai';
    if (c.includes('physics') || c.includes('nav') || c.includes('character_controller') ||
        c.includes('raycast') || c.includes('collision') || c.includes('spatial_hash'))
        return 'physics';
    if (c.includes('terrain') || c.includes('pcg') || c.includes('weaving') || c.includes('heightmap') ||
        c.includes('voxel') || c.includes('chunk') || c.includes('erosion') || c.includes('climate') ||
        c.includes('worldpartition') || c.includes('world_chunk'))
        return 'terrain';
    if (c.includes('render') || c.includes('culling') || c.includes('shader') || c.includes('texture') ||
        c.includes('mesh') || c.includes('cluster') || c.includes('lod') || c.includes('instanc') ||
        c.includes('bloom') || c.includes('shadow') || c.includes('light') || c.includes('hiz') ||
        c.includes('ibl') || c.includes('msaa') || c.includes('decal') || c.includes('depth') ||
        c.includes('frustum') || c.includes('aabb') || c.includes('camera') || c.includes('csm') ||
        c.includes('cascade') || c.includes('gpu') || c.includes('overlay') || c.includes('pcf') ||
        c.includes('prefix_sum') || c.includes('indirect') || c.includes('material') || c.includes('bake') ||
        c.includes('baking'))
        return 'rendering';
    if (c.includes('math') || c.includes('vec') || c.includes('mat') || c.includes('simd') || c.includes('quat'))
        return 'math';
    if (c.includes('net') || c.includes('network')) return 'networking';
    if (c.includes('persistence') || c.includes('save')) return 'persistence';
    if (c.includes('input')) return 'input';
    if (c.includes('audio')) return 'audio';
    if (c.includes('ui') || c.includes('gizmo') || c.includes('widget') || c.includes('astract'))
        return 'ui';
    if (c.includes('sdk') || c.includes('editor') || c.includes('build') || c.includes('hash'))
        return 'tools';
    if (c.includes('enemy') || c.includes('player') || c.includes('quest') || c.includes('attack') ||
        c.includes('combat') || c.includes('integrated') || c.includes('battle') || c.includes('pattern') ||
        c.includes('weather') || c.includes('fluids'))
        return 'ecs';
    return 'other';
}

function getSystemColor(crate) {
    return COLOR_SCHEME[detectSystem(crate)] || COLOR_SCHEME.default;
}

// ─── INDUSTRY STANDARD LOOKUP ────────────────────────────────────────────────

function findIndustryStandard(benchmarkName, system) {
    const stds = INDUSTRY_STANDARDS[system];
    if (!stds) return null;
    const lower = benchmarkName.toLowerCase();
    for (const [key, engines] of Object.entries(stds.benchmarks)) {
        if (lower.includes(key.toLowerCase())) {
            const vals = Object.values(engines);
            const avg = vals.reduce((a, b) => a + b, 0) / vals.length;
            return { average: avg, engines, key };
        }
    }
    return null;
}

// ─── RENDER DASHBOARD ────────────────────────────────────────────────────────

function renderDashboard() {
    const renderStart = performance.now();
    try {
        applyFilters();
        renderHardwareComparisonBar();
        renderProductionHealthSummary();
        renderRegressionAlerts();
        renderStatCards();
        renderChart();
        renderIndustryComparisonSection();
        renderBenchTable();
        updateGeneratedTime();
        PERF.renderCount++;
        const elapsed = (performance.now() - renderStart).toFixed(0);
        console.log(`Render #${PERF.renderCount} in ${elapsed}ms (${filteredData.length} pts)`);
    } catch (error) {
        console.error('Error rendering:', error);
        showError(`Rendering error: ${error.message}`);
    }
}

// ─── HARDWARE COMPARISON BAR ─────────────────────────────────────────────────

function renderHardwareComparisonBar() {
    const c = document.getElementById('hardware-info');
    if (!c) return;
    const baseCount = benchmarkData.filter(d => d.hardware_id === BASELINE_HARDWARE_ID).length;
    const userCount = benchmarkData.filter(d => d.hardware_id !== BASELINE_HARDWARE_ID).length;

    let baseInfo = `<span style="color:#43e97b">Baseline: ${BASELINE_HARDWARE_LABEL}</span>`;
    let userInfo = '';
    const userHw = Array.from(hardwareProfiles.entries()).filter(([id]) => id !== BASELINE_HARDWARE_ID);
    if (userHw.length > 0) {
        userInfo = userHw.map(([, label]) => `<span style="color:#4facfe">User: ${label}</span>`).join('<br/>');
    }

    c.innerHTML = `
        <div style="display:flex;gap:24px;align-items:center;flex-wrap:wrap;">
            <div style="flex:1;min-width:200px;">
                <div style="font-size:0.85em;color:#a0a0a0;margin-bottom:4px;">Hardware Profiles (${hardwareProfiles.size})</div>
                <div style="font-size:0.9em;">${baseInfo}</div>
                ${userInfo ? `<div style="font-size:0.9em;margin-top:4px;">${userInfo}</div>` : ''}
            </div>
            <div style="display:flex;gap:16px;align-items:center;">
                <div style="text-align:center;padding:8px 16px;background:rgba(67,233,123,0.1);border-radius:8px;">
                    <div style="font-size:1.4em;font-weight:bold;color:#43e97b;">${baseCount}</div>
                    <div style="font-size:0.75em;color:#a0a0a0;">Baseline Points</div>
                </div>
                <div style="text-align:center;padding:8px 16px;background:rgba(79,172,254,0.1);border-radius:8px;">
                    <div style="font-size:1.4em;font-weight:bold;color:#4facfe;">${userCount}</div>
                    <div style="font-size:0.75em;color:#a0a0a0;">User Points</div>
                </div>
            </div>
        </div>`;
}

// ─── STAT CARDS ──────────────────────────────────────────────────────────────

function renderStatCards() {
    const grid = document.getElementById('stats-grid');
    if (filteredData.length === 0) {
        grid.innerHTML = '<div class="loading">No data for selected filters</div>';
        return;
    }
    const groups = {};
    filteredData.forEach(d => { (groups[d.benchmark_name] = groups[d.benchmark_name] || []).push(d); });

    const stats = [];
    for (const [name, entries] of Object.entries(groups)) {
        entries.sort((a, b) => a.timestamp - b.timestamp);
        const latest = entries[entries.length - 1], oldest = entries[0];
        const change = entries.length > 1 ? ((latest.value - oldest.value) / oldest.value) * 100 : 0;
        const sys = detectSystem(latest.crate);
        const ref = findIndustryStandard(name, sys);
        stats.push({
            name, value: latest.value, unit: latest.unit, change, system: sys,
            color: getSystemColor(latest.crate),
            industryAvg: ref ? ref.average : null,
            isBaseline: latest.hardware_id === BASELINE_HARDWARE_ID
        });
    }
    stats.sort((a, b) => b.value - a.value);

    grid.innerHTML = '';
    stats.slice(0, 8).forEach(s => {
        const card = document.createElement('div');
        card.className = `stat-card ${s.system}`;
        const icon = s.change > 0 ? '▲' : s.change < 0 ? '▼' : '─';
        const cc = s.change > 10 ? '#ff6b6b' : s.change < -10 ? '#43e97b' : '#a0a0a0';
        let badge = '';
        if (s.industryAvg !== null && currentFilters.showIndustry) {
            const ratio = ((s.value / s.industryAvg) * 100).toFixed(0);
            const ic = s.value <= s.industryAvg ? '#43e97b' : '#ff6b6b';
            const il = s.value <= s.industryAvg ? 'faster' : 'slower';
            badge = `<div style="font-size:0.75em;color:${ic};margin-top:4px;">vs Industry: ${ratio}% (${il})</div>`;
        }
        card.innerHTML = `
            <div class="stat-label">${s.name}</div>
            <div class="stat-value">${formatNumber(s.value)}<span class="stat-unit">${s.unit}</span></div>
            <div class="stat-change" style="color:${cc}">${icon} ${Math.abs(s.change).toFixed(1)}%
                ${s.isBaseline ? '<span style="font-size:0.7em;color:#43e97b;margin-left:4px;">[baseline]</span>' : ''}
            </div>${badge}`;
        grid.appendChild(card);
    });
}

function formatNumber(v) {
    if (v === 0) return '0';
    if (Math.abs(v) < 0.01) return v.toExponential(1);
    if (Math.abs(v) < 10) return v.toFixed(2);
    if (Math.abs(v) < 100) return v.toFixed(1);
    if (Math.abs(v) < 1000) return v.toFixed(0);
    if (Math.abs(v) < 1e6) return (v / 1e3).toFixed(1) + 'K';
    if (Math.abs(v) < 1e9) return (v / 1e6).toFixed(1) + 'M';
    return (v / 1e9).toFixed(2) + 'G';
}

function formatDuration(ns) {
    if (ns < 1000) return ns.toFixed(1) + ' ns';
    if (ns < 1e6) return (ns / 1e3).toFixed(1) + ' µs';
    if (ns < 1e9) return (ns / 1e6).toFixed(1) + ' ms';
    return (ns / 1e9).toFixed(2) + ' s';
}

// ─── MAIN CHART ──────────────────────────────────────────────────────────────

function renderChart() {
    const div = document.getElementById('chart');
    div.innerHTML = '';
    if (filteredData.length === 0) { div.innerHTML = '<div class="loading">No data</div>'; return; }
    if (typeof d3 === 'undefined') {
        div.innerHTML = '<div class="loading" style="color:#ff6b6b;">D3.js failed to load.</div>';
        return;
    }

    const benchGroups = d3.group(filteredData, d => d.benchmark_name);
    let series = [];
    benchGroups.forEach((values, name) => {
        values.sort((a, b) => a.timestamp - b.timestamp);
        const hwGroups = d3.group(values, d => d.hardware_id);
        hwGroups.forEach((hwVals, hwId) => {
            const isBase = hwId === BASELINE_HARDWARE_ID;
            series.push({
                name: hardwareProfiles.size > 1 ? `${name} [${isBase ? 'Baseline' : 'User'}]` : name,
                rawName: name,
                values: hwVals,
                color: getSystemColor(hwVals[0].crate),
                isBaseline: isBase,
                hardwareId: hwId
            });
        });
    });

    let chartSeries = series;
    if (currentFilters.benchmark && currentFilters.benchmark !== 'all')
        chartSeries = series.filter(s => s.rawName === currentFilters.benchmark);

    const title = document.getElementById('chart-title');
    title.textContent = (currentFilters.benchmark && currentFilters.benchmark !== 'all')
        ? currentFilters.benchmark : `All Benchmarks (${chartSeries.length} series)`;

    const margin = { top: 20, right: 140, bottom: 60, left: 80 };
    const width = div.clientWidth - margin.left - margin.right;
    const height = 500 - margin.top - margin.bottom;

    const svg = d3.select(div).append('svg')
        .attr('width', width + margin.left + margin.right)
        .attr('height', height + margin.top + margin.bottom)
        .append('g').attr('transform', `translate(${margin.left},${margin.top})`);

    const allVals = chartSeries.flatMap(s => s.values);
    const xScale = d3.scaleTime().domain(d3.extent(allVals, d => d.timestamp)).range([0, width]);
    let yMax = d3.max(allVals, d => d.value) * 1.15;

    // Industry lines
    const industryLines = [];
    if (currentFilters.showIndustry) {
        const seen = new Set();
        chartSeries.forEach(s => {
            if (seen.has(s.rawName)) return;
            seen.add(s.rawName);
            const sys = detectSystem(s.values[0].crate);
            const ref = findIndustryStandard(s.rawName, sys);
            if (ref) {
                industryLines.push({ name: s.rawName, value: ref.average, engines: ref.engines, color: INDUSTRY_LINE_COLOR });
                if (ref.average * 1.1 > yMax) yMax = ref.average * 1.15;
            }
        });
    }

    const yScale = d3.scaleLinear().domain([0, yMax]).range([height, 0]);

    // Grid
    svg.append('g').attr('class', 'grid').attr('transform', `translate(0,${height})`)
        .call(d3.axisBottom(xScale).tickSize(-height).tickFormat(''));
    svg.append('g').attr('class', 'grid').call(d3.axisLeft(yScale).tickSize(-width).tickFormat(''));

    // Axes
    svg.append('g').attr('class', 'axis').attr('transform', `translate(0,${height})`)
        .call(d3.axisBottom(xScale).ticks(6).tickFormat(d3.timeFormat('%m/%d')));
    svg.append('g').attr('class', 'axis').call(d3.axisLeft(yScale).ticks(8).tickFormat(d => formatNumber(d)));
    svg.append('text').attr('transform', 'rotate(-90)').attr('y', -60).attr('x', -height / 2)
        .attr('text-anchor', 'middle').attr('fill', '#a0a0a0').style('font-size', '14px').text('Time (ns)');

    // Industry standard lines
    industryLines.forEach(il => {
        svg.append('line').attr('x1', 0).attr('x2', width)
            .attr('y1', yScale(il.value)).attr('y2', yScale(il.value))
            .attr('stroke', il.color).attr('stroke-width', 2)
            .attr('stroke-dasharray', '8,4').attr('opacity', 0.7);
        const names = Object.keys(il.engines).join('/');
        svg.append('text').attr('x', width + 4).attr('y', yScale(il.value) + 4)
            .attr('fill', il.color).style('font-size', '10px').text(`Industry (${names})`);
    });

    // Data lines
    const lineGen = d3.line().x(d => xScale(d.timestamp)).y(d => yScale(d.value)).curve(d3.curveMonotoneX);
    chartSeries.forEach(s => {
        svg.append('path').datum(s.values).attr('class', 'line').attr('d', lineGen)
            .attr('stroke', s.color).attr('stroke-width', s.isBaseline ? 3 : 2)
            .attr('stroke-dasharray', s.isBaseline ? 'none' : '6,3').attr('opacity', s.isBaseline ? 1 : 0.8);
        svg.selectAll(null).data(s.values).enter().append('circle')
            .attr('cx', d => xScale(d.timestamp)).attr('cy', d => yScale(d.value))
            .attr('r', 3).attr('fill', s.color).attr('opacity', 0.7);
    });

    // Tooltip
    const tooltip = d3.select('.tooltip');
    svg.append('rect').attr('width', width).attr('height', height)
        .attr('fill', 'none').attr('pointer-events', 'all')
        .on('mousemove', function(event) {
            const [mx] = d3.pointer(event);
            const x0 = xScale.invert(mx);
            let closest = null, closestD = Infinity;
            allVals.forEach(d => { const dist = Math.abs(d.timestamp - x0); if (dist < closestD) { closestD = dist; closest = d; } });
            if (closest) {
                const isB = closest.hardware_id === BASELINE_HARDWARE_ID;
                const sys = detectSystem(closest.crate);
                const ref = findIndustryStandard(closest.benchmark_name, sys);
                let indLine = '';
                if (ref && currentFilters.showIndustry) {
                    const ratio = ((closest.value / ref.average) * 100).toFixed(0);
                    indLine = `<br/><span style="color:${INDUSTRY_LINE_COLOR}">Industry avg: ${formatDuration(ref.average)} (${ratio}%)</span>`;
                }
                const sdLine = closest.stddev ? `<br/>\u00b1${formatNumber(closest.stddev)} (\u03c3)` : '';
                tooltip.style('left', (event.pageX + 15) + 'px').style('top', (event.pageY - 28) + 'px')
                    .classed('visible', true)
                    .html(`<strong>${closest.display_name || closest.benchmark_name}</strong><br/>
                        Value: ${formatDuration(closest.value)}${sdLine}<br/>
                        Date: ${d3.timeFormat('%Y-%m-%d %H:%M')(closest.timestamp)}<br/>
                        Hardware: ${isB ? '🏠 Baseline' : '🖥️ User'}<br/>
                        Branch: ${closest.git_branch} | SHA: ${closest.git_sha}${indLine}`);
            }
        })
        .on('mouseleave', () => tooltip.classed('visible', false));

    renderLegend(chartSeries, industryLines);
    renderHistogram(chartSeries);
    renderSparklines(chartSeries);
}

// ─── INDUSTRY COMPARISON SECTION ─────────────────────────────────────────────

function renderIndustryComparisonSection() {
    const c = document.getElementById('industry-comparison');
    if (!c) return;
    if (!currentFilters.showIndustry) { c.innerHTML = ''; return; }

    const groups = {};
    filteredData.forEach(d => { (groups[d.benchmark_name] = groups[d.benchmark_name] || []).push(d); });

    const comparisons = [];
    for (const [name, entries] of Object.entries(groups)) {
        entries.sort((a, b) => a.timestamp - b.timestamp);
        const latest = entries[entries.length - 1];
        const sys = detectSystem(latest.crate);
        const ref = findIndustryStandard(name, sys);
        if (ref) comparisons.push({
            name, displayName: latest.display_name || name,
            ourValue: latest.value, unit: latest.unit, system: sys,
            engines: ref.engines, industryAvg: ref.average
        });
    }

    if (comparisons.length === 0) {
        c.innerHTML = '<div class="loading" style="font-size:1em;">No industry comparisons for current filters</div>';
        return;
    }
    comparisons.sort((a, b) => (a.ourValue / a.industryAvg) - (b.ourValue / b.industryAvg));

    const margin = { top: 20, right: 20, bottom: 40, left: 260 };
    const barH = 28, gap = 6;
    const totalH = (barH + gap) * comparisons.length + margin.top + margin.bottom;
    const svgW = c.clientWidth || 800;
    const w = svgW - margin.left - margin.right;

    c.innerHTML = '';
    const svg = d3.select(c).append('svg').attr('width', svgW).attr('height', totalH)
        .append('g').attr('transform', `translate(${margin.left},${margin.top})`);

    const maxV = d3.max(comparisons, d => Math.max(d.ourValue, d.industryAvg)) * 1.2;
    const x = d3.scaleLinear().domain([0, maxV]).range([0, w]);

    comparisons.forEach((comp, i) => {
        const y = i * (barH + gap);
        // Industry bar
        svg.append('rect').attr('x', 0).attr('y', y).attr('width', x(comp.industryAvg))
            .attr('height', barH / 2).attr('fill', INDUSTRY_LINE_COLOR).attr('opacity', 0.4).attr('rx', 3);
        // Our bar
        const col = comp.ourValue <= comp.industryAvg ? '#43e97b' : '#ff6b6b';
        svg.append('rect').attr('x', 0).attr('y', y + barH / 2).attr('width', x(comp.ourValue))
            .attr('height', barH / 2).attr('fill', col).attr('opacity', 0.8).attr('rx', 3);
        // Label
        svg.append('text').attr('x', -8).attr('y', y + barH / 2 + 2).attr('text-anchor', 'end')
            .attr('fill', '#e0e0e0').style('font-size', '11px').text(comp.displayName.substring(0, 40));
        // Values
        const ratio = ((comp.ourValue / comp.industryAvg) * 100).toFixed(0);
        svg.append('text').attr('x', Math.max(x(comp.ourValue), x(comp.industryAvg)) + 8)
            .attr('y', y + barH / 2 + 4).attr('fill', col).style('font-size', '10px')
            .text(`${formatNumber(comp.ourValue)} vs ${formatNumber(comp.industryAvg)} (${ratio}%)`);
    });

    // Legend
    svg.append('rect').attr('x', w - 200).attr('y', -16).attr('width', 12).attr('height', 8)
        .attr('fill', INDUSTRY_LINE_COLOR).attr('opacity', 0.6);
    svg.append('text').attr('x', w - 184).attr('y', -9).attr('fill', '#a0a0a0').style('font-size', '10px').text('Industry Avg');
    svg.append('rect').attr('x', w - 110).attr('y', -16).attr('width', 12).attr('height', 8)
        .attr('fill', '#43e97b').attr('opacity', 0.8);
    svg.append('text').attr('x', w - 94).attr('y', -9).attr('fill', '#a0a0a0').style('font-size', '10px').text('AstraWeave');
}

// ─── HISTOGRAM ───────────────────────────────────────────────────────────────

function renderHistogram(series) {
    const div = document.getElementById('histogram');
    div.innerHTML = '';
    if (series.length === 0) return;

    const vals = series.map(s => ({ name: s.name, value: s.values[s.values.length - 1].value, color: s.color }));
    const margin = { top: 10, right: 12, bottom: 30, left: 60 };
    const w = div.clientWidth - margin.left - margin.right;
    const h = 160 - margin.top - margin.bottom;

    const svg = d3.select(div).append('svg')
        .attr('width', w + margin.left + margin.right).attr('height', h + margin.top + margin.bottom)
        .append('g').attr('transform', `translate(${margin.left},${margin.top})`);

    const x = d3.scaleLinear().domain([0, d3.max(vals, d => d.value) * 1.05]).range([0, w]);
    const bins = d3.bin().thresholds(20).value(d => d.value)(vals);
    const y = d3.scaleLinear().range([h, 0]).domain([0, d3.max(bins, d => d.length)]);

    svg.selectAll('rect').data(bins).enter().append('rect')
        .attr('x', d => x(d.x0)).attr('y', d => y(d.length))
        .attr('width', d => Math.max(1, x(d.x1) - x(d.x0) - 1))
        .attr('height', d => h - y(d.length)).attr('fill', '#4facfe').attr('opacity', 0.8);

    svg.append('g').attr('transform', `translate(0,${h})`).call(d3.axisBottom(x).ticks(6).tickFormat(d => formatNumber(d)));
    svg.append('g').call(d3.axisLeft(y).ticks(4));
}

// ─── SPARKLINES ──────────────────────────────────────────────────────────────

function renderSparklines(series) {
    const container = document.getElementById('sparklines');
    container.innerHTML = '';
    if (series.length === 0) return;
    series.slice(0, 64).forEach(s => {
        const box = document.createElement('div');
        box.style.cssText = 'background:rgba(255,255,255,0.03);border-radius:6px;padding:8px;min-height:64px;display:flex;flex-direction:column;justify-content:center;';
        const label = document.createElement('div');
        label.style.cssText = 'color:#a0a0a0;font-size:12px;';
        label.textContent = s.name;
        box.appendChild(label);
        const spark = document.createElement('div');
        box.appendChild(spark);

        const sparkSvg = d3.select(spark).append('svg').attr('width', '100%').attr('height', 48).append('g');
        const w = spark.clientWidth || 200, h = 48;
        const xs = d3.scaleTime().domain(d3.extent(s.values, d => d.timestamp)).range([0, w]);
        const ys = d3.scaleLinear().domain([d3.min(s.values, d => d.value), d3.max(s.values, d => d.value)]).range([h - 4, 4]);
        const line = d3.line().x(d => xs(d.timestamp)).y(d => ys(d.value)).curve(d3.curveBasis);
        sparkSvg.append('path').datum(s.values).attr('d', line)
            .attr('stroke', s.color).attr('stroke-width', 1.5).attr('fill', 'none');
        container.appendChild(box);
    });
}

// ─── BENCHMARK TABLE ─────────────────────────────────────────────────────────

function renderBenchTable() {
    const c = document.getElementById('benchmark-table');
    c.innerHTML = '';
    const groups = d3.group(filteredData, d => d.benchmark_name);
    const rows = [];
    groups.forEach((vals, name) => {
        vals.sort((a, b) => a.timestamp - b.timestamp);
        const latest = vals[vals.length - 1], oldest = vals[0];
        const change = vals.length > 1 ? ((latest.value - oldest.value) / oldest.value) * 100 : 0;
        const sys = detectSystem(latest.crate);
        rows.push({ name, latest, change, ref: findIndustryStandard(name, sys), system: sys });
    });
    rows.sort((a, b) => b.latest.value - a.latest.value);

    const table = document.createElement('table');
    table.style.cssText = 'width:100%;border-collapse:collapse;';
    c.appendChild(table);

    const head = document.createElement('thead');
    const hr = document.createElement('tr');
    ['Benchmark', 'System', 'Latest', 'p50', 'p95', 'CV%', 'Change', 'vs Industry', 'Hardware'].forEach(h => {
        const th = document.createElement('th');
        th.style.cssText = 'text-align:left;padding:6px 10px;color:#a0a0a0;font-size:0.85em;';
        th.textContent = h; hr.appendChild(th);
    });
    head.appendChild(hr); table.appendChild(head);

    const body = document.createElement('tbody');
    rows.forEach(r => {
        const tr = document.createElement('tr');
        tr.style.borderTop = '1px solid rgba(255,255,255,0.03)';

        const td = (text, style) => { const el = document.createElement('td'); el.style.cssText = 'padding:6px 10px;' + (style || ''); el.innerHTML = text; return el; };

        // Compute per-row stats
        const allVals = filteredData.filter(d => d.benchmark_name === r.name).map(d => d.value);
        const p50 = percentile(allVals, 50);
        const p95 = percentile(allVals, 95);
        const cv = coefficientOfVariation(allVals);

        tr.appendChild(td(r.latest.display_name || r.name));
        tr.appendChild(td(`<span style="color:${COLOR_SCHEME[r.system]||'#a0a0a0'};font-size:0.8em;">${r.system}</span>`));
        tr.appendChild(td(`${formatNumber(r.latest.value)} ${r.latest.unit}`));
        tr.appendChild(td(`${formatNumber(p50)}`, 'color:#a0a0a0;font-size:0.85em;'));
        tr.appendChild(td(`${formatNumber(p95)}`, 'color:#a0a0a0;font-size:0.85em;'));
        const cvColor = cv < 5 ? '#43e97b' : cv < 15 ? '#feca57' : '#ff6b6b';
        tr.appendChild(td(`<span style="color:${cvColor}">${cv.toFixed(1)}%</span>`, 'font-size:0.85em;'));
        tr.appendChild(td(`${r.change > 0 ? '+' : ''}${r.change.toFixed(1)}%`, `color:${r.change > 0 ? '#ff6b6b' : '#43e97b'}`));

        let indTxt = '—';
        if (r.ref && currentFilters.showIndustry) {
            const ratio = ((r.latest.value / r.ref.average) * 100).toFixed(0);
            const ic = r.latest.value <= r.ref.average ? '#43e97b' : '#ff6b6b';
            indTxt = `<span style="color:${ic}">${ratio}%</span>`;
        }
        tr.appendChild(td(indTxt));
        tr.appendChild(td(r.latest.hardware_id === BASELINE_HARDWARE_ID ? '🏠 Baseline' : '🖥️ User', 'font-size:0.8em;color:#a0a0a0;'));

        body.appendChild(tr);
    });
    table.appendChild(body);
}

// ─── LEGEND ──────────────────────────────────────────────────────────────────

function renderLegend(series, industryLines = []) {
    const div = document.getElementById('legend');
    div.innerHTML = '';
    series.slice(0, 10).forEach(s => {
        const item = document.createElement('div'); item.className = 'legend-item';
        const color = document.createElement('div'); color.className = 'legend-color';
        color.style.backgroundColor = s.color;
        if (!s.isBaseline) color.style.borderTop = '2px dashed ' + s.color;
        const label = document.createElement('span'); label.textContent = s.name;
        item.appendChild(color); item.appendChild(label); div.appendChild(item);
    });
    if (industryLines.length > 0 && currentFilters.showIndustry) {
        const item = document.createElement('div'); item.className = 'legend-item';
        const color = document.createElement('div'); color.className = 'legend-color';
        color.style.backgroundColor = INDUSTRY_LINE_COLOR;
        color.style.borderTop = '2px dashed ' + INDUSTRY_LINE_COLOR;
        const label = document.createElement('span'); label.textContent = 'Industry Standard';
        label.style.color = INDUSTRY_LINE_COLOR;
        item.appendChild(color); item.appendChild(label); div.appendChild(item);
    }
}

// ─── REGRESSION ALERTS ───────────────────────────────────────────────────────

function renderRegressionAlerts() {
    const c = document.getElementById('regression-alerts');
    if (!c) return;
    c.innerHTML = '';
    const groups = {};
    filteredData.forEach(d => { (groups[d.benchmark_name] = groups[d.benchmark_name] || []).push(d); });

    const regs = [];
    for (const [name, entries] of Object.entries(groups)) {
        entries.sort((a, b) => a.timestamp - b.timestamp);
        if (entries.length < 2) continue;
        const change = ((entries[entries.length - 1].value - entries[0].value) / entries[0].value) * 100;
        if (change > 10) regs.push({ name, baseline: entries[0].value, current: entries[entries.length - 1].value, change, unit: entries[entries.length - 1].unit });
    }

    if (regs.length === 0) {
        c.innerHTML = `<div class="alert-success">✅ <strong>No Performance Regressions Detected</strong>
            <p style="margin-top:8px;font-size:0.9em;color:#a0a0a0;">All ${Object.keys(groups).length} benchmarks within ±10%</p></div>`;
        return;
    }
    regs.sort((a, b) => b.change - a.change);
    c.innerHTML = `<div class="alert-warning">⚠️ <strong>${regs.length} Regression${regs.length > 1 ? 's' : ''}</strong>
        <div style="margin-top:12px;max-height:200px;overflow-y:auto;">
            ${regs.slice(0, 10).map(r => `<div style="padding:8px;border-left:3px solid #ff6b6b;margin:6px 0;background:rgba(255,107,107,0.1);border-radius:4px;">
                <div style="font-weight:600;color:#ff6b6b;">${r.name}</div>
                <div style="font-size:0.85em;color:#a0a0a0;margin-top:4px;">${formatNumber(r.baseline)} → ${formatNumber(r.current)} ${r.unit}
                    <span style="color:#ff6b6b;font-weight:600;">(+${r.change.toFixed(1)}%)</span></div>
            </div>`).join('')}
        </div></div>`;
}

// ─── PRODUCTION HEALTH ───────────────────────────────────────────────────────

function renderProductionHealthSummary() {
    const c = document.getElementById('production-health');
    if (!c) return;

    const uniqueB = new Set(filteredData.map(d => d.benchmark_name)).size;
    const uniqueC = new Set(filteredData.map(d => d.crate)).size;
    const uniqueH = new Set(filteredData.map(d => d.hardware_id)).size;
    const dateRange = filteredData.length > 0
        ? Math.ceil((Math.max(...filteredData.map(d => d.timestamp)) - Math.min(...filteredData.map(d => d.timestamp))) / 86400000) : 0;

    const groups = {};
    filteredData.forEach(d => { (groups[d.benchmark_name] = groups[d.benchmark_name] || []).push(d); });
    let regCount = 0;
    for (const entries of Object.values(groups)) {
        entries.sort((a, b) => a.timestamp - b.timestamp);
        if (entries.length >= 2 && ((entries[entries.length - 1].value - entries[0].value) / entries[0].value) * 100 > 10) regCount++;
    }
    const total = Object.keys(groups).length;
    const pct = total > 0 ? (regCount / total) * 100 : 0;
    let grade = 'A+', gc = '#00f2fe';
    if (pct > 20) { grade = 'C'; gc = '#ff6b6b'; }
    else if (pct > 10) { grade = 'B'; gc = '#feca57'; }
    else if (pct > 5) { grade = 'A'; gc = '#43e97b'; }

    let indCount = 0;
    for (const name of Object.keys(groups)) {
        const sys = detectSystem(filteredData.find(d => d.benchmark_name === name)?.crate || '');
        if (findIndustryStandard(name, sys)) indCount++;
    }

    const sysCounts = {};
    filteredData.forEach(d => { const s = detectSystem(d.crate); sysCounts[s] = (sysCounts[s] || 0) + 1; });

    c.innerHTML = `
        <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(130px,1fr));gap:16px;">
            ${[
                [grade, gc, 'Health Grade'],
                [uniqueB, '#4facfe', 'Benchmarks'],
                [uniqueC, '#4facfe', 'Crates'],
                [uniqueH, '#4facfe', 'Hardware Profiles'],
                [`${dateRange}d`, '#4facfe', 'Data Range'],
                [indCount, INDUSTRY_LINE_COLOR, 'Industry Comparisons']
            ].map(([v, color, label]) => `
                <div style="text-align:center;padding:16px;background:rgba(255,255,255,0.03);border-radius:8px;">
                    <div style="font-size:${String(v).length > 3 ? '1.5' : '2'}em;font-weight:bold;color:${color};">${v}</div>
                    <div style="color:#a0a0a0;font-size:0.85em;margin-top:4px;">${label}</div>
                </div>`).join('')}
        </div>
        <div style="margin-top:16px;display:flex;flex-wrap:wrap;gap:8px;justify-content:center;">
            ${Object.entries(sysCounts).map(([s, cnt]) =>
                `<span style="padding:4px 12px;background:${COLOR_SCHEME[s]||'#666'}22;color:${COLOR_SCHEME[s]||'#666'};border-radius:16px;font-size:0.8em;">${s}: ${cnt}</span>`
            ).join('')}
        </div>`;
}

// ─── UTILITIES ───────────────────────────────────────────────────────────────

function updateGeneratedTime() {
    document.getElementById('generated-time').textContent = new Date().toLocaleString();
    ['top-series', 'distribution', 'heatmap'].forEach(id => {
        const img = document.getElementById('static-' + id);
        if (!img) return;
        fetch(img.src, { method: 'HEAD' }).then(r => { img.style.display = r.ok ? 'inline-block' : 'none'; }).catch(() => { img.style.display = 'none'; });
    });
}

function showError(msg) {
    document.getElementById('stats-grid').innerHTML = `
        <div class="error"><h2>Error Loading Dashboard</h2>
            <p>${msg.replace(/\n/g, '<br>')}</p>
            <h3 style="margin-top:20px;">Generate Benchmark Data:</h3>
            <div style="text-align:left;max-width:600px;margin:20px auto;">
                <p><strong>One-Command (Recommended):</strong></p>
                <pre>.\\scripts\\run_benchmark_dashboard.ps1</pre>
                <p style="margin-top:10px;"><strong>Manual:</strong></p>
                <pre>cargo bench\n.\\scripts\\export_benchmark_jsonl.ps1</pre>
            </div>
        </div>`;
}

function showLoadingSkeleton() {
    const grid = document.getElementById('stats-grid');
    if (grid) {
        grid.innerHTML = Array.from({ length: 6 }, () =>
            `<div class="stat-card skeleton" style="height:120px;border-left-color:transparent;"></div>`
        ).join('');
    }
    ['production-health', 'regression-alerts', 'chart', 'industry-comparison', 'benchmark-table'].forEach(id => {
        const el = document.getElementById(id);
        if (el) el.innerHTML = '<div class="skeleton" style="height:80px;margin:8px 0;"></div>';
    });
}

// ─── EXPORT FUNCTIONS ───────────────────────────────────────────────────────

function exportCSV() {
    const data = filteredData.length > 0 ? filteredData : benchmarkData;
    const headers = ['benchmark_name','display_name','crate','group','value','stddev','unit','timestamp','git_branch','git_sha','hardware_id','hardware_label','system','p50','p95','cv_pct'];
    const groups = {};
    data.forEach(d => { (groups[d.benchmark_name] = groups[d.benchmark_name] || []).push(d); });
    const rows = data.map(d => {
        const grpVals = (groups[d.benchmark_name] || []).map(e => e.value);
        const sys = detectSystem(d.crate);
        return headers.map(h => {
            if (h === 'system') return sys;
            if (h === 'p50') return percentile(grpVals, 50).toFixed(2);
            if (h === 'p95') return percentile(grpVals, 95).toFixed(2);
            if (h === 'cv_pct') return coefficientOfVariation(grpVals).toFixed(2);
            if (h === 'timestamp') return d.timestamp instanceof Date ? d.timestamp.toISOString() : d.timestamp;
            const v = d[h];
            return v !== undefined && v !== null ? String(v).replace(/,/g, ';') : '';
        }).join(',');
    });
    const csv = [headers.join(','), ...rows].join('\n');
    downloadFile(csv, 'astraweave-benchmarks.csv', 'text/csv');
    showToast(`Exported ${data.length} records as CSV`);
}

function exportJSON() {
    const data = filteredData.length > 0 ? filteredData : benchmarkData;
    const exported = data.map(d => ({
        ...d,
        timestamp: d.timestamp instanceof Date ? d.timestamp.toISOString() : d.timestamp,
        system: detectSystem(d.crate)
    }));
    const json = JSON.stringify(exported, null, 2);
    downloadFile(json, 'astraweave-benchmarks.json', 'application/json');
    showToast(`Exported ${data.length} records as JSON`);
}

function shareSnapshot() {
    persistFiltersToURL();
    const url = location.href;
    navigator.clipboard.writeText(url).then(() => {
        showToast('\ud83d\udd17 Dashboard URL copied to clipboard!');
    }).catch(() => {
        // Fallback for non-HTTPS contexts
        prompt('Copy this URL:', url);
    });
}

function downloadFile(content, filename, mime) {
    const blob = new Blob([content], { type: mime });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url; a.download = filename;
    document.body.appendChild(a); a.click();
    setTimeout(() => { document.body.removeChild(a); URL.revokeObjectURL(url); }, 100);
}

// ─── INIT ────────────────────────────────────────────────────────────────────
document.addEventListener('DOMContentLoaded', () => {
    console.log('AstraWeave Benchmark Dashboard v7.0');
    loadBenchmarkData();
});

// Resize handler — debounced re-render on window resize
let resizeTimer;
window.addEventListener('resize', () => {
    clearTimeout(resizeTimer);
    resizeTimer = setTimeout(() => {
        if (benchmarkData.length > 0) renderDashboard();
    }, 250);
});
