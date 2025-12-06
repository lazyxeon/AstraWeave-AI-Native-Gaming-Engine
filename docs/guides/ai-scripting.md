# AI Scripting (Rhai)

The engine calls into Rhai for:
- **Encounter budgets** (fortify/collapse/spawn)
- **Boss phase logic** (HP/time driven)
- **Persona profile distillation**

Contract:
- Input: world snapshot (read-only) + limited op budget
- Output: allowed verbs only; engine validates/corrects

See: `content/encounters/*.rhai` in your repo.