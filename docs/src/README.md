<!-- markdownlint-disable MD013 MD033 MD041 -->
<div class="astra-landing">
  <section class="astra-hero">
    <div class="astra-hero__copy">
      <span class="astra-eyebrow">MIT licensed and AI-native</span>
      <h1>Build worlds where intelligent agents are a core system, not an afterthought.</h1>
      <p class="astra-lead">
        AstraWeave is a free, MIT-licensed Rust game engine for developers who need
        deterministic simulation, tool-validated AI behavior, and evidence-backed performance.
        Its core loop is built around perception, reasoning, planning, validation, and action,
        so large-scale agent logic can live inside the engine instead of fighting against it.
      </p>
      <div class="astra-actions">
        <a class="astra-button astra-button--primary" href="./architecture/overview.md">Explore the architecture</a>
        <a class="astra-button astra-button--secondary" href="./getting-started/installation.md">Build the workspace</a>
      </div>
      <div class="astra-meta" aria-label="Adoption highlights">
        <span class="astra-meta-badge">MIT license</span>
        <span class="astra-meta-badge">Free to use, fork, and modify</span>
        <span class="astra-meta-badge">Built for engine developers</span>
      </div>
      <p class="astra-note">
        Open documentation, desktop-first workflows, extensive examples, and a verification
        posture built around tests, Miri, Kani, and deterministic replay validation.
      </p>
    </div>
    <div class="astra-hero__visual">
      <div class="astra-hero__frame">
        <div class="astra-hero__image-wrap">
          <img class="astra-hero__image" src="./assets/Astraweave_logo.jpg" alt="AstraWeave nebula logo">
        </div>
        <div class="astra-hero__caption">
          <strong>Engine scope at a glance</strong>
          <span>
            AstraWeave is an open MIT-licensed engine for deterministic, agent-heavy games. The
            goal of this front page is practical orientation: show the identity, prove the claims,
            and move you quickly into the parts of the workspace you can build on.
          </span>
          <div class="astra-hero__highlights" aria-label="Hero highlights">
            <span>Deterministic ECS and replay-safe simulation</span>
            <span>AI planning, validation, and runtime tooling</span>
            <span>Benchmarks, docs, and contribution paths up front</span>
          </div>
        </div>
      </div>
    </div>
  </section>
  <section class="astra-proof-strip" aria-label="Key proof points">
    <article class="astra-proof-tile">
      <strong>12,700+</strong>
      <span>validated agents at 60 FPS on modest consumer hardware</span>
    </article>
    <article class="astra-proof-tile">
      <strong>39,000+</strong>
      <span>workspace tests across 128 packages</span>
    </article>
    <article class="astra-proof-tile">
      <strong>977</strong>
      <span>Miri tests with zero undefined behavior</span>
    </article>
    <article class="astra-proof-tile">
      <strong>2.70 ms</strong>
      <span>frame time at 1,000 entities in the current benchmark baseline</span>
    </article>
    <article class="astra-proof-tile">
      <strong>71+</strong>
      <span>Kani proof harnesses across safety-critical crates</span>
    </article>
  </section>
  <section class="astra-section" id="paths">
    <div class="astra-section-heading astra-section-heading--wide">
      <span class="astra-eyebrow">Start in the docs tree</span>
      <h2>The homepage now maps directly to the documentation structure behind it.</h2>
      <p>
        If you prefer navigating by documentation area instead of landing-page copy, start here.
        These paths mirror the mdBook sidebar so the handoff from homepage to reference material is direct.
      </p>
    </div>
    <div class="astra-grid astra-grid--three">
      <article class="astra-card">
        <span class="astra-kicker">Getting started</span>
        <h3>Boot the workspace and get a first system running.</h3>
        <ul class="astra-link-list astra-link-list--stacked">
          <li><span>Quick start</span><a href="./getting-started/quick-start.md">Open quick start</a></li>
          <li><span>Installation</span><a href="./getting-started/installation.md">Install dependencies</a></li>
          <li><span>First companion</span><a href="./getting-started/first-companion.md">Build your first AI companion</a></li>
        </ul>
      </article>
      <article class="astra-card">
        <span class="astra-kicker">Architecture</span>
        <h3>Understand the runtime model before touching code.</h3>
        <ul class="astra-link-list astra-link-list--stacked">
          <li><span>Overview</span><a href="./architecture/overview.md">Read architecture overview</a></li>
          <li><span>AI-native design</span><a href="./architecture/ai-native.md">Inspect AI-native design</a></li>
          <li><span>Validation</span><a href="./architecture/tool-validation.md">Study tool validation</a></li>
        </ul>
      </article>
      <article class="astra-card">
        <span class="astra-kicker">Core systems</span>
        <h3>Jump into the engine subsystems that define actual capability.</h3>
        <ul class="astra-link-list astra-link-list--stacked">
          <li><span>AI system</span><a href="./core-systems/ai/index.md">Open AI systems</a></li>
          <li><span>Physics and fluids</span><a href="./core-systems/physics.md">Inspect simulation systems</a></li>
          <li><span>Terrain and navigation</span><a href="./core-systems/navigation.md">Browse world systems</a></li>
        </ul>
      </article>
      <article class="astra-card">
        <span class="astra-kicker">Examples and performance</span>
        <h3>See working demos, benchmarks, and optimization guidance.</h3>
        <ul class="astra-link-list astra-link-list--stacked">
          <li><span>Examples</span><a href="./examples/index.md">Browse examples</a></li>
          <li><span>Benchmarks</span><a href="./performance/benchmarks.md">Open benchmark dashboard</a></li>
          <li><span>Optimization</span><a href="./performance/optimization.md">Read optimization guide</a></li>
        </ul>
      </article>
      <article class="astra-card">
        <span class="astra-kicker">Engine development</span>
        <h3>Contribute, build from source, and follow the repo workflow.</h3>
        <ul class="astra-link-list astra-link-list--stacked">
          <li><span>Contributing</span><a href="./dev/contributing.md">Open contribution guide</a></li>
          <li><span>Building</span><a href="./dev/building.md">Build from source</a></li>
          <li><span>Testing</span><a href="./dev/testing.md">Review validation workflow</a></li>
        </ul>
      </article>
      <article class="astra-card">
        <span class="astra-kicker">API and reference</span>
        <h3>Use the API index when you already know what subsystem you need.</h3>
        <ul class="astra-link-list astra-link-list--stacked">
          <li><span>API overview</span><a href="./api/index.md">Open API reference</a></li>
          <li><span>Crate map</span><a href="./reference/crates.md">Browse crate documentation</a></li>
          <li><span>CLI and config</span><a href="./reference/cli-tools.md">Inspect tools and configuration</a></li>
        </ul>
      </article>
    </div>
  </section>
  <section class="astra-section" id="proof">
    <div class="astra-section-heading">
      <span class="astra-eyebrow">Why AstraWeave</span>
      <h2>Three reasons developers can trust the claims here.</h2>
      <p>
        The case for AstraWeave is not branding or speculation. It is the combination of
        AI-native architecture, deterministic simulation, open inspection, and measurable
        engineering rigor across the stack.
      </p>
    </div>
    <div class="astra-grid astra-grid--three">
      <article class="astra-card">
        <span class="astra-kicker">AI-first architecture</span>
        <h3>Perception to action is part of the engine contract.</h3>
        <p>
          World snapshots, plan intents, tool validation, behavior trees, GOAP, utility logic,
          and LLM-backed planning all plug into a deterministic runtime loop instead of living in
          disconnected gameplay scripts.
        </p>
      </article>
      <article class="astra-card">
        <span class="astra-kicker">Deterministic and safe</span>
        <h3>Built for replay validation, anti-cheat integrity, and reproducibility.</h3>
        <p>
          The engine emphasizes bit-identical replay, validator-gated actions, and memory-safety
          verification. Core unsafe code paths have already been exercised under Miri, with Kani
          proofs backing critical invariants in the ECS, math, core, and SDK layers.
        </p>
      </article>
      <article class="astra-card">
        <span class="astra-kicker">Benchmarked subsystems</span>
        <h3>Performance claims connect back to current benchmark reports.</h3>
        <p>
          ECS world creation, character movement, simulation tick costs, rendering frame time,
          SIMD math throughput, and high-agent AI validation are all documented with specific
          measurements instead of broad claims about being fast or scalable.
        </p>
      </article>
    </div>
    <div class="astra-band">
      <div class="astra-section-heading astra-section-heading--compact">
        <span class="astra-eyebrow">Engine loop</span>
        <h2>Perception, reasoning, planning, validation, action.</h2>
        <p>
          The engine's differentiator is structural. Agents observe the world, generate plans,
          validate each available tool or action, and only then mutate simulation state.
        </p>
      </div>
      <div class="astra-flow">
        <div class="astra-node">
          <strong>Perception</strong>
          <span>World snapshots built from deterministic ECS state and environment context.</span>
        </div>
        <div class="astra-node">
          <strong>Reasoning</strong>
          <span>Behavior trees, utility systems, GOAP, or LLM-backed logic interpret the current state.</span>
        </div>
        <div class="astra-node">
          <strong>Planning</strong>
          <span>Plan intents and action sequences are assembled with explicit costs, priorities, and fallbacks.</span>
        </div>
        <div class="astra-node">
          <strong>Validation</strong>
          <span>Cooldowns, LOS, pathing, and sandbox rules constrain what the engine will actually execute.</span>
        </div>
        <div class="astra-node">
          <strong>Action</strong>
          <span>Approved commands flow back into simulation, physics, audio, rendering, and UI systems.</span>
        </div>
      </div>
    </div>
  </section>
  <section class="astra-section" id="systems">
    <div class="astra-section-heading astra-section-heading--wide">
      <span class="astra-eyebrow">What ships today</span>
      <h2>A focused stack for intelligent, simulation-heavy games.</h2>
      <p>
        AstraWeave is strongest when the game depends on believable NPCs, systemic simulation,
        deterministic networking, and a codebase developers can actually inspect, modify, and
        embed under an MIT license.
      </p>
    </div>
    <div class="astra-grid astra-grid--three">
      <article class="astra-card">
        <span class="astra-kicker">AI orchestration</span>
        <h3>Six validated modes plus hybrid arbiters.</h3>
        <p>
          Classical planners, behavior trees, utility logic, LLM orchestration, ensemble patterns,
          and hybrid arbiters let teams mix fast deterministic control with richer strategic reasoning.
        </p>
        <div class="astra-chip-row">
          <a class="astra-chip" href="./architecture/ai-native.md">AI-native design</a>
          <a class="astra-chip" href="./api/ai.md">AI API</a>
        </div>
      </article>
      <article class="astra-card">
        <span class="astra-kicker">Deterministic ECS</span>
        <h3>Ordered simulation for replay, tooling, and scale.</h3>
        <p>
          Archetype storage, system staging, entity iteration guarantees, and event channels form a
          reproducible simulation backbone designed for agent-heavy worlds.
        </p>
        <div class="astra-chip-row">
          <a class="astra-chip" href="./architecture/ecs.md">ECS architecture</a>
          <a class="astra-chip" href="./api/ecs.md">ECS API</a>
        </div>
      </article>
      <article class="astra-card">
        <span class="astra-kicker">Rendering</span>
        <h3>wgpu-based rendering tuned for real engine workloads.</h3>
        <p>
          PBR materials, clustered lighting, GPU skinning, post-processing, LOD tooling, and
          rendering benchmarks make the visual stack credible for actual game prototypes and showcases.
        </p>
        <div class="astra-chip-row">
          <a class="astra-chip" href="./core-systems/rendering.md">Rendering systems</a>
          <a class="astra-chip" href="./api/render.md">Render API</a>
        </div>
      </article>
      <article class="astra-card">
        <span class="astra-kicker">Physics and movement</span>
        <h3>Character control, spatial hashing, fluids, cloth, and more.</h3>
        <p>
          AstraWeave wraps robust simulation systems around gameplay needs: collision, character
          motion, destructibles, ragdolls, vehicles, gravity zones, and high-coverage fluid simulation.
        </p>
        <div class="astra-chip-row">
          <a class="astra-chip" href="./core-systems/physics.md">Physics</a>
          <a class="astra-chip" href="./core-systems/fluids.md">Fluids</a>
        </div>
      </article>
      <article class="astra-card">
        <span class="astra-kicker">Navigation and world systems</span>
        <h3>Navmesh, terrain, scene streaming, and gameplay layers.</h3>
        <p>
          Terrain, scene partitioning, procedural generation, navigation meshes, crafting, quests,
          and dialogue systems give the engine enough surface area to support actual vertical slices.
        </p>
        <div class="astra-chip-row">
          <a class="astra-chip" href="./core-systems/navigation.md">Navigation</a>
          <a class="astra-chip" href="./core-systems/terrain.md">Terrain</a>
        </div>
      </article>
      <article class="astra-card">
        <span class="astra-kicker">Tooling and integration</span>
        <h3>Examples, editor work, and a C ABI for embedding.</h3>
        <p>
          The workspace includes a large example suite, tooling for editor workflows, and a stable C SDK
          layer for teams that need to embed AstraWeave systems into a broader stack.
        </p>
        <div class="astra-chip-row">
          <a class="astra-chip" href="./examples/index.md">Examples</a>
          <a class="astra-chip" href="./api/index.md">API docs</a>
        </div>
      </article>
    </div>
  </section>
  <section class="astra-section">
    <div class="astra-split">
      <article class="astra-card astra-card--panel">
        <span class="astra-kicker">Evidence-backed metrics</span>
        <h3>Metrics tied to the current codebase.</h3>
        <p>
          The right way to present AstraWeave is with the figures the codebase can actually support today.
        </p>
        <ul class="astra-list">
          <li><span>Agent capacity at 60 FPS</span><strong>12,700+</strong></li>
          <li><span>AI validation throughput</span><strong>6.48M checks/sec</strong></li>
          <li><span>Frame time at 1,000 entities</span><strong>2.70 ms</strong></li>
          <li><span>ECS world creation</span><strong>25.8 ns</strong></li>
          <li><span>Character move cost</span><strong>114 ns</strong></li>
          <li><span>SIMD batch over 10k entities</span><strong>9.879 us</strong></li>
        </ul>
      </article>
      <article class="astra-bench-card" aria-label="Top benchmark results">
        <div class="astra-bench-header">
          <div>
            <div class="astra-bench-title">Top Benchmarks &middot; Latest Run</div>
            <div class="astra-bench-subtitle">Criterion.rs statistical benchmarking &middot; Feb 2026</div>
          </div>
          <span class="astra-bench-badge"><span class="astra-bench-badge-dot"></span>Stable baseline</span>
        </div>
        <div class="astra-bench-legend">
          <span class="astra-bench-li"><span class="astra-bench-sw" style="background:#4ea8de"></span>Bake 10K Triangles</span>
          <span class="astra-bench-li"><span class="astra-bench-sw" style="background:#e8915a"></span>E2E Plan Gen &middot; Cache Miss</span>
          <span class="astra-bench-li"><span class="astra-bench-sw" style="background:#fbbf24"></span>Cache Latency &middot; 200ms</span>
          <span class="astra-bench-li"><span class="astra-bench-sw" style="background:#a78bfa"></span>Light Binning &middot; High</span>
          <span class="astra-bench-li"><span class="astra-bench-sw" style="background:#22d3ee"></span>Bloom Upsample &middot; Mip 0</span>
          <span class="astra-bench-li"><span class="astra-bench-sw" style="background:#f472b6"></span>Light Binning &middot; 5K</span>
          <span class="astra-bench-li"><span class="astra-bench-sw" style="background:#f87171"></span>Cache Latency &middot; 100ms</span>
          <span class="astra-bench-li"><span class="astra-bench-sw" style="background:#5bc9a0"></span>Game Loop &middot; 5K Stress</span>
        </div>
        <svg class="astra-bench-svg" viewBox="0 0 900 390" xmlns="http://www.w3.org/2000/svg" aria-label="Horizontal bar chart of benchmark results">
          <rect width="900" height="390" fill="#161922"/>
          <line x1="310" y1="14" x2="310" y2="350" stroke="rgba(255,255,255,0.04)"/>
          <line x1="410" y1="14" x2="410" y2="350" stroke="rgba(255,255,255,0.04)"/>
          <line x1="510" y1="14" x2="510" y2="350" stroke="rgba(255,255,255,0.04)"/>
          <line x1="610" y1="14" x2="610" y2="350" stroke="rgba(255,255,255,0.04)"/>
          <line x1="710" y1="14" x2="710" y2="350" stroke="rgba(255,255,255,0.04)"/>
          <text x="310" y="372" fill="#555a6e" font-size="10" text-anchor="middle" style="font-family:Consolas,monospace">100 ms</text>
          <text x="410" y="372" fill="#555a6e" font-size="10" text-anchor="middle" style="font-family:Consolas,monospace">200 ms</text>
          <text x="510" y="372" fill="#555a6e" font-size="10" text-anchor="middle" style="font-family:Consolas,monospace">300 ms</text>
          <text x="610" y="372" fill="#555a6e" font-size="10" text-anchor="middle" style="font-family:Consolas,monospace">400 ms</text>
          <text x="710" y="372" fill="#555a6e" font-size="10" text-anchor="middle" style="font-family:Consolas,monospace">500 ms</text>
          <defs>
            <linearGradient id="blueBarGlow" x1="0" y1="0" x2="1" y2="0"><stop offset="0%" stop-color="#4ea8de"/><stop offset="100%" stop-color="#4ea8de" stop-opacity="0.6"/></linearGradient>
          </defs>
          <text x="200" y="39" fill="#8b90a0" font-size="11" text-anchor="end" style="font-family:'Segoe UI',system-ui,sans-serif">Bake 10K Triangles</text>
          <rect x="210" y="22" width="548" height="28" rx="4" fill="url(#blueBarGlow)" opacity="0.9"/>
          <text x="766" y="40" fill="#e2e4ea" font-size="10" style="font-family:Consolas,monospace">548.16 ms</text>
          <text x="200" y="81" fill="#8b90a0" font-size="11" text-anchor="end" style="font-family:'Segoe UI',system-ui,sans-serif">E2E Plan Gen · Cache Miss</text>
          <rect x="210" y="64" width="219" height="28" rx="4" fill="#e8915a" opacity="0.85"/>
          <text x="435" y="82" fill="#e2e4ea" font-size="10" style="font-family:Consolas,monospace">218.79 ms</text>
          <text x="200" y="123" fill="#8b90a0" font-size="11" text-anchor="end" style="font-family:'Segoe UI',system-ui,sans-serif">Cache Latency · 200ms</text>
          <rect x="210" y="106" width="209" height="28" rx="4" fill="#fbbf24" opacity="0.85"/>
          <text x="425" y="124" fill="#e2e4ea" font-size="10" style="font-family:Consolas,monospace">209.26 ms</text>
          <text x="200" y="165" fill="#8b90a0" font-size="11" text-anchor="end" style="font-family:'Segoe UI',system-ui,sans-serif">Light Binning · High</text>
          <rect x="210" y="148" width="176" height="28" rx="4" fill="#a78bfa" opacity="0.85"/>
          <text x="392" y="166" fill="#e2e4ea" font-size="10" style="font-family:Consolas,monospace">176.06 ms</text>
          <text x="200" y="207" fill="#8b90a0" font-size="11" text-anchor="end" style="font-family:'Segoe UI',system-ui,sans-serif">Bloom Upsample · Mip 0</text>
          <rect x="210" y="190" width="173" height="28" rx="4" fill="#22d3ee" opacity="0.85"/>
          <text x="389" y="208" fill="#e2e4ea" font-size="10" style="font-family:Consolas,monospace">172.55 ms</text>
          <text x="200" y="249" fill="#8b90a0" font-size="11" text-anchor="end" style="font-family:'Segoe UI',system-ui,sans-serif">Light Binning · 5K</text>
          <rect x="210" y="232" width="113" height="28" rx="4" fill="#f472b6" opacity="0.85"/>
          <text x="329" y="250" fill="#e2e4ea" font-size="10" style="font-family:Consolas,monospace">113.34 ms</text>
          <text x="200" y="291" fill="#8b90a0" font-size="11" text-anchor="end" style="font-family:'Segoe UI',system-ui,sans-serif">Cache Latency · 100ms</text>
          <rect x="210" y="274" width="109" height="28" rx="4" fill="#f87171" opacity="0.85"/>
          <text x="325" y="292" fill="#e2e4ea" font-size="10" style="font-family:Consolas,monospace">108.52 ms</text>
          <text x="200" y="333" fill="#8b90a0" font-size="11" text-anchor="end" style="font-family:'Segoe UI',system-ui,sans-serif">Game Loop · 5K Stress</text>
          <rect x="210" y="316" width="94" height="28" rx="4" fill="#5bc9a0" opacity="0.85"/>
          <text x="310" y="334" fill="#e2e4ea" font-size="10" style="font-family:Consolas,monospace">93.91 ms</text>
        </svg>
        <div class="astra-bench-footer">
          <span class="astra-bench-caption">
            All measurements are p50 medians from Criterion.rs with &ge;100 iterations per benchmark and 95% confidence intervals.
          </span>
          <span class="astra-bench-source">criterion &middot; cargo bench</span>
        </div>
      </article>
    </div>
    <div class="astra-split astra-split--offset">
      <article class="astra-card astra-card--panel">
        <span class="astra-kicker">Quality posture</span>
        <h3>Verification, testing, and measurement are part of the product story.</h3>
        <p>
          AstraWeave's coverage numbers are not artificially inflated. Large GPU and async subsystems lower the
          weighted average, but high-value core crates show strong line coverage and extensive mutation, Miri,
          and proof-based validation.
        </p>
        <ul class="astra-list">
          <li><span>Weighted line coverage</span><strong>59.3% across measured crates</strong></li>
          <li><span>High-coverage crates</span><strong>14 crates at 85%+</strong></li>
          <li><span>Miri validation</span><strong>977 tests, 0 UB</strong></li>
          <li><span>Kani verification</span><strong>71+ harnesses</strong></li>
          <li><span>Prompt mutation testing</span><strong>100% adjusted kill rate</strong></li>
          <li><span>Desktop targets</span><strong>Windows, Linux, macOS</strong></li>
        </ul>
      </article>
      <article class="astra-card astra-card--panel">
        <span class="astra-kicker">Developer routes</span>
        <h3>Pick an entry point and go straight to implementation.</h3>
        <p>
          This homepage is meant to work like a front door, not a dead-end banner. The fastest next step depends on
          whether you care about architecture, implementation detail, benchmarking, contribution workflow, or getting
          the workspace running locally.
        </p>
        <ul class="astra-link-list">
          <li><span>System design</span><a href="./architecture/overview.md">Open architecture overview</a></li>
          <li><span>Performance data</span><a href="./performance/benchmarks.md">Open benchmarks</a></li>
          <li><span>Build and run</span><a href="./getting-started/quick-start.md">Read quick start</a></li>
          <li><span>Examples and demos</span><a href="./examples/index.md">Browse examples</a></li>
          <li><span>Contributing workflow</span><a href="./dev/contributing.md">Open contribution guide</a></li>
          <li><span>Reference implementation</span><a href="./veilweaver/overview.md">Inspect Veilweaver</a></li>
        </ul>
      </article>
    </div>
  </section>
  <section class="astra-section" id="audiences">
    <div class="astra-section-heading astra-section-heading--wide">
      <span class="astra-eyebrow">Use cases</span>
      <h2>Four kinds of developer work this engine already supports well.</h2>
      <p>
        AstraWeave is not trying to be every engine for every project. It is most credible where agent behavior,
        deterministic systems, and validation-heavy runtime guarantees are central to the game and the team wants
        source-level control over the stack.
      </p>
    </div>
    <div class="astra-grid astra-grid--four">
      <article class="astra-card">
        <span class="astra-kicker">RPGs and immersive sims</span>
        <h3>Companions, directors, and systemic encounters.</h3>
        <p>
          The engine is well-suited for projects where companions, enemies, or world systems need to observe,
          plan, and react with more structure than state-machine scripting usually allows.
        </p>
      </article>
      <article class="astra-card">
        <span class="astra-kicker">Server-authoritative multiplayer</span>
        <h3>Validation and replayability matter.</h3>
        <p>
          Deterministic simulation and tool-gated actions make AstraWeave a strong fit for projects that need
          anti-cheat discipline, reproducibility, or replay validation.
        </p>
      </article>
      <article class="astra-card">
        <span class="astra-kicker">Research and prototyping</span>
        <h3>Benchmarkable AI-native architecture.</h3>
        <p>
          If the point is to test agent scale, planning strategies, orchestration modes, or hybrid AI control under
          measurable conditions, this workspace already has the right instrumentation story.
        </p>
      </article>
      <article class="astra-card">
        <span class="astra-kicker">Embedded engine teams</span>
        <h3>Rust core with a C ABI.</h3>
        <p>
          The SDK and modular crate structure make it possible to adopt focused subsystems instead of committing to
          the whole stack at once.
        </p>
      </article>
    </div>
  </section>
  <section class="astra-section" id="lineages">
    <div class="astra-section-heading astra-section-heading--wide">
      <span class="astra-eyebrow">Design lineages</span>
      <h2>The kinds of games AstraWeave could help realize more fully.</h2>
      <p>
        This engine is a strong fit for rights-holder ports of classic systemic games, or for new
        original projects inspired by the same design space. The goal here is not to reuse protected
        IP without permission, but to show the kinds of simulation-heavy games that benefit from
        AstraWeave's runtime model.
      </p>
    </div>
    <div class="astra-grid astra-grid--three">
      <article class="astra-card">
        <span class="astra-kicker">Colony and world simulation</span>
        <h3>In the design lineage of Dwarf Fortress or RimWorld.</h3>
        <p>
          Deep agent autonomy, world-state memory, logistics, faction pressure, and emergent story
          generation map naturally onto AstraWeave's perception, planning, and deterministic simulation layers.
        </p>
      </article>
      <article class="astra-card">
        <span class="astra-kicker">4X and grand strategy</span>
        <h3>In the design lineage of Civilization or Crusader Kings.</h3>
        <p>
          Multi-agent diplomacy, simulation turn resolution, advisor systems, strategic planners,
          and explainable AI behavior are exactly the kinds of workloads this architecture can support well.
        </p>
      </article>
      <article class="astra-card">
        <span class="astra-kicker">Tactical command games</span>
        <h3>In the design lineage of X-COM, Jagged Alliance, or Battle Brothers.</h3>
        <p>
          Tool validation, cover awareness, action planning, morale systems, and replay-safe combat
          loops make AstraWeave a good substrate for modern tactical simulation and squad AI.
        </p>
      </article>
      <article class="astra-card">
        <span class="astra-kicker">Systemic sandboxes</span>
        <h3>In the design lineage of Kenshi or Mount and Blade.</h3>
        <p>
          Large numbers of autonomous actors, persistent world consequences, combat behaviors,
          navigation, and faction simulation benefit from an engine where AI is part of the core loop.
        </p>
      </article>
      <article class="astra-card">
        <span class="astra-kicker">Immersive sims and party RPGs</span>
        <h3>In the design lineage of Ultima Underworld, Deus Ex, or Dragon Age.</h3>
        <p>
          Companion decision-making, quest-state reactivity, systemic encounters, dialogue-aware
          behaviors, and director-style orchestration all become easier when runtime reasoning is explicit.
        </p>
      </article>
      <article class="astra-card">
        <span class="astra-kicker">Rights-holder remakes and successors</span>
        <h3>Best used for licensed ports or original games built in the same spirit.</h3>
        <p>
          AstraWeave can help teams rebuild ambitious systemic designs as they were imagined, but only
          where the team owns the IP, holds the necessary rights, or is making an original work rather than
          a derivative one.
        </p>
      </article>
    </div>
  </section>
  <section class="astra-cta">
    <div>
      <span class="astra-eyebrow">Next step</span>
      <h2>Build the workspace, inspect the engine, and decide from evidence.</h2>
      <p>
        Read the architecture, inspect the benchmarks, clone the repository, or move straight into setup. The landing
        page is designed to get developers into the real work quickly instead of trapping them in overview copy.
      </p>
      <p class="astra-note astra-note--light">
        AstraWeave is free and MIT licensed. If the architecture fits your project, you can evaluate it in depth,
        adopt pieces of it, or contribute back without platform lock-in.
      </p>
    </div>
    <div class="astra-actions astra-actions--stacked">
      <a class="astra-button astra-button--primary" href="./getting-started/installation.md">Install dependencies</a>
      <a class="astra-button astra-button--secondary" href="./performance/benchmarks.md">Read benchmark report</a>
      <a class="astra-button astra-button--secondary" href="./dev/contributing.md">Contribute to the engine</a>
    </div>
  </section>
</div>
<!-- markdownlint-enable MD013 MD033 MD041 -->
