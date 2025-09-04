# Veilweaver • AstraWeave • AstraCore

**Veilweaver: Threads of Eternity** — a flagship **AI‑native action RPG** where your **AI companion** learns across playthroughs and **AI endbosses** evolve against your tactics.  
**AstraWeave** — a custom, AI‑first game engine that makes agents (companions & bosses) first‑class citizens.  
**AstraCore** — a companion‑centric, AI‑accelerated console platform designed to run AI gameplay locally with low latency.

> High‑level concepts for Veilweaver were originally defined in a pitch that explored a quantum‑inspired archipelago, fate‑weaving mechanics, and 100+ hours of emergent RPG gameplay. We carry those pillars forward and make them AI‑native here. :contentReference[oaicite:0]{index=0} :contentReference[oaicite:1]{index=1}


---

## Two‑minute pitch

- **Your AI companion** is persistent. It learns your style, remembers your victories, and travels with you across titles built on AstraWeave.
- **AI endbosses** act like living game masters: they scout you, alter environments (fortify, flood, collapse, lure), and counter your favorite tactics.
- **Every run is personal**: fate‑weaving + adaptive agents reshape levels, quests, and boss playbooks so replays never feel the same. :contentReference[oaicite:2]{index=2} :contentReference[oaicite:3]{index=3}

---

## Quick links

- Start with the **game**: [`veilweaver.md`](Game/Veilweaver.md)  
- Deep dive the **engine**[`astraweave.md`](:https://github.com/lazyxeon/Veilweaver-Threads-of-Eternity/blob/e61cb05624d60bb89157ede7b9d3699e7c3f134d/AI%20Engine/AstraWeave.md)
- See the **console** vision: [`astracore.md`]([Console/astracore.md](https://github.com/lazyxeon/Veilweaver-Threads-of-Eternity/blob/2e391db5c6017df720737db6453daab47fab38b8/Console/AstraCore.md))

---

## What’s Under the Hood (one-liners)

- Deterministic simulation: fixed-tick rules so results are repeatable and fair.
- Tool sandbox: AI can only use allowed verbs (e.g., MoveTo, Throw smoke); engine enforces cooldowns, line-of-sight, and pathing.
- Pathfinding & cover: A* pathing and simple cover sampling are built in.
- Portable memory: .cprof files store the companion’s persona, skills, and distilled memories.
- Scriptable design: Encounter budgets/hints come from small scripts—no rebuild needed.
- Local-first AI: Works with mock or local models now; you can add edge/cloud later—engine keeps it safe either way.

## What’s Not Included (yet)

- Full 3D rendering/gameplay or content for a shipping RPG.
- The dedicated console hardware (AstraCore) is a spec target; today is the PC dev kit.

---

## License & ethos

- **No pay‑to‑win. No loot boxes.** Expansions must be meaty, optional, and never fragment core content.  
- Player data & companion memory stay on‑device by default. Cloud is optional and transparent.  
- Creator marketplace allows personas/modules/maps with strict anti‑exploit curation.

