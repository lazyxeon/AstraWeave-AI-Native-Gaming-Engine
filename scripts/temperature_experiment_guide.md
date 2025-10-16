# Temperature Experiment Guide - Hermes 2 Pro Optimization

**Date**: October 15, 2025  
**Status**: Ready to execute  
**Baseline**: Temperature 0.5 (100% success, 21.2s avg latency)  

---

## Objective

Determine optimal temperature configuration for Hermes 2 Pro in AstraWeave production deployment.

**Research Question**: Does temperature affect success rate, latency, or tactical creativity?

**Hypothesis**:
- **0.3 (deterministic)**: Higher consistency, lower diversity, potentially faster
- **0.5 (balanced)**: Current validated baseline (100% success)
- **0.7 (creative)**: Higher diversity, risk of lower consistency, potentially slower

---

## Configuration Locations

### File to Modify

**Path**: `examples/hello_companion/src/main.rs`  
**Line**: ~726  
**Current Code**:
```rust
let client = Hermes2ProOllama::localhost()
    .with_temperature(0.5)        // <-- MODIFY THIS LINE
    .with_max_tokens(1024);
```

### Temperature Settings to Test

1. **Temperature 0.3** (Deterministic)
   ```rust
   .with_temperature(0.3)
   ```

2. **Temperature 0.5** (Baseline - ALREADY TESTED)
   ```rust
   .with_temperature(0.5)  // ✅ 20 runs complete, 100% success
   ```

3. **Temperature 0.7** (Creative)
   ```rust
   .with_temperature(0.7)
   ```

---

## Test Protocol

### For Each Temperature (0.3, 0.7):

#### Step 1: Modify Code
```powershell
# Open file in editor
code examples\hello_companion\src\main.rs

# Manually change line 726:
# .with_temperature(0.5) → .with_temperature(0.3)  # or 0.7
```

#### Step 2: Recompile
```powershell
cargo build -p hello_companion --release
# Expected time: 15-30 seconds (incremental)
```

#### Step 3: Run 10 Iterations
```powershell
cd scripts
.\test_hermes2pro_validation.ps1 -Iterations 10 -OutputFile "hermes2pro_temp_0.3.csv"
# Expected time: ~3.5 minutes (10 runs × 21s avg)
```

#### Step 4: Collect Results
```powershell
# Review summary in terminal output
# CSV saved to: scripts\hermes2pro_temp_0.3.csv
```

#### Step 5: Repeat for 0.7
```powershell
# Modify temperature to 0.7
# Recompile
.\test_hermes2pro_validation.ps1 -Iterations 10 -OutputFile "hermes2pro_temp_0.7.csv"
```

---

## Expected Metrics to Compare

| Metric | Temp 0.3 | Temp 0.5 (Baseline) | Temp 0.7 | Best Config |
|--------|----------|---------------------|----------|-------------|
| **Success Rate** | ? | **100%** (20/20) | ? | ? |
| **Avg Latency** | ? | **21.2s** | ? | ? |
| **Std Dev** | ? | **±6.3s** | ? | ? |
| **Fallback Rate** | ? | **0%** | ? | ? |
| **Tool Diversity** | ? | **8 unique tools** | ? | ? |
| **Plan Diversity** | ? | **Moderate** | ? | ? |

### Key Comparison Axes

1. **Consistency**: Lower temperature = more deterministic plans
2. **Creativity**: Higher temperature = more varied tool combinations
3. **Latency**: Temperature impact on generation speed (hypothesis: minimal)
4. **Stability**: Success rate variance across temperatures

---

## Analysis Plan

### After All Tests Complete

**Files to Analyze**:
- `scripts/hermes2pro_temp_0.3.csv` (10 runs)
- `scripts/hermes2pro_extended_validation.csv` (20 runs @ temp 0.5)
- `scripts/hermes2pro_temp_0.7.csv` (10 runs)

**Metrics to Calculate**:

#### 1. Success Rate Comparison
```
Temp 0.3: X/10 (Y%)
Temp 0.5: 20/20 (100%)  ✅ Baseline
Temp 0.7: X/10 (Y%)
```

#### 2. Latency Comparison
```
Temp 0.3: Avg Z.Z s (±W.W s)
Temp 0.5: Avg 21.2 s (±6.3 s)  ✅ Baseline
Temp 0.7: Avg Z.Z s (±W.W s)
```

#### 3. Tool Diversity
```
Temp 0.3: X unique tools, Y total invocations
Temp 0.5: 8 unique tools, 52 total invocations  ✅ Baseline
Temp 0.7: X unique tools, Y total invocations
```

#### 4. Plan Diversity (Manual Review)
- **Temp 0.3**: Are plans more repetitive? (e.g., always SCAN → MOVE_TO → THROW_SMOKE)
- **Temp 0.5**: Moderate diversity (5 SCAN-only, 5 SCAN→MOVE_TO→SMOKE, 3 APPROACH, etc.)
- **Temp 0.7**: Are plans more creative? (e.g., rare tools like DODGE, FLANK, MARK_TARGET)

#### 5. Tactical Quality (Manual Review)
- Review 5 random plans from each temperature
- Score 1-5 on tactical appropriateness (1=nonsensical, 5=brilliant)
- Expected: All temperatures score 4-5 (Hermes 2 Pro is trained for reasoning)

---

## Decision Matrix

### Recommendation Based on Results

| Scenario | Success @ 0.3 | Success @ 0.7 | Recommendation |
|----------|---------------|---------------|----------------|
| **Both 100%** | ✅ 100% | ✅ 100% | Use **0.3** (most deterministic) |
| **0.3 high, 0.7 low** | ✅ 90%+ | ⚠️ <80% | Use **0.3** (consistency wins) |
| **0.3 low, 0.7 high** | ⚠️ <80% | ✅ 90%+ | Use **0.7** (creativity wins) |
| **Both marginal** | ⚠️ 80-90% | ⚠️ 80-90% | Use **0.5** (baseline proven) |
| **Both fail** | ❌ <80% | ❌ <80% | Investigate prompt issue |

**Tiebreaker Factors**:
1. **Latency**: If success rates equal, choose faster temperature
2. **Diversity**: If latency equal, choose temperature matching gameplay need:
   - **Turn-based strategy**: Prefer 0.7 (creative, varied tactics)
   - **Roguelike**: Prefer 0.7 (replayability)
   - **Puzzle**: Prefer 0.3 (deterministic, predictable)
3. **Production Risk**: If uncertain, prefer 0.5 (20-run validation proven)

---

## Expected Timeline

| Task | Duration | Cumulative |
|------|----------|------------|
| Modify code (0.3) | 2 min | 0:02 |
| Compile | 0.5 min | 0:02.5 |
| Run 10 tests @ 0.3 | 3.5 min | 0:06 |
| Review results | 2 min | 0:08 |
| Modify code (0.7) | 2 min | 0:10 |
| Compile | 0.5 min | 0:10.5 |
| Run 10 tests @ 0.7 | 3.5 min | 0:14 |
| Review results | 2 min | 0:16 |
| **Comparative analysis** | 15 min | **0:31** |
| **Create report** | 15 min | **0:46** |

**Total Time**: ~45-60 minutes

---

## Automated Analysis Script (Optional)

**PowerShell Script** (already created): `scripts/test_temperature_comparison.ps1`

**Usage** (if running all at once):
```powershell
cd scripts
.\test_temperature_comparison.ps1

# This script will:
# 1. Prompt you to modify temperature to 0.3
# 2. Run 10 tests at 0.3
# 3. Prompt you to modify temperature to 0.7
# 4. Run 10 tests at 0.7
# 5. Generate comparative summary
```

**Note**: Still requires manual code modification before each temperature test.

---

## Success Criteria

**Experiment is successful if:**

1. ✅ All 3 temperatures tested (10+ runs each)
2. ✅ Comparative metrics calculated (success, latency, diversity)
3. ✅ Optimal temperature identified for production
4. ✅ Recommendation documented in Phase 7 completion report

**Production deployment is approved if:**

1. ✅ At least ONE temperature achieves 80%+ success rate (preferably 90%+)
2. ✅ Latency remains <60s for 95% of runs
3. ✅ JSON quality remains >90%
4. ✅ Tactical quality validated (manual review of 10+ plans)

---

## Troubleshooting

### Issue: Compilation fails after modifying temperature

**Solution**:
```powershell
# Verify syntax is correct
code examples\hello_companion\src\main.rs

# Check line 726:
.with_temperature(0.3)  # Valid
.with_temperature(0.7)  # Valid
.with_temperature 0.3   # Invalid (missing parentheses)
```

### Issue: Tests fail at new temperature

**Diagnosis**:
1. Check success rate: If 0%, likely prompt/model issue
2. Check parse tier: If all "Fallback", LLM isn't generating valid JSON
3. Check error messages in output

**Resolution**:
- If <50% success: Temperature likely too extreme, use 0.5
- If parse errors: Temperature may affect JSON formatting (unlikely)
- If tactical errors: Temperature may affect reasoning (review plans manually)

### Issue: Tests take too long (>5 min per run)

**Diagnosis**: Network latency or Ollama server issue

**Resolution**:
1. Check Ollama server: `ollama list`
2. Restart Ollama: `ollama serve`
3. Reduce iterations: `-Iterations 5` for faster testing

---

## Next Steps After Temperature Experiments

1. **Document Results** - Create `HERMES2PRO_TEMPERATURE_OPTIMIZATION.md`
2. **Update Configuration** - Set optimal temperature in production config
3. **Test Simplified Prompt** - Validate 59% latency reduction hypothesis
4. **Phase 7 Completion** - Finalize migration documentation
5. **Production Deployment** - Ship Hermes 2 Pro for turn-based games

---

**END OF GUIDE**

**Status**: ✅ **READY TO EXECUTE**  
**Next Action**: Modify temperature to 0.3, compile, run 10 tests  
**ETA to Completion**: ~45-60 minutes
