$crates = @(
  "astraweave-prompts","astraweave-render","astraweave-core","astraweave-observability",
  "astraweave-fluids","astraweave-embeddings","astraweave-ecs","astraweave-optimization",
  "astraweave-terrain","astraweave-security","astraweave-weaving","astraweave-net-ecs",
  "astraweave-behavior","astraweave-llm","astraweave-memory","astraweave-npc",
  "astraweave-dialogue","astraweave-scripting","astraweave-director","astraweave-steam"
)
$base = "C:\Users\pv2br\AstraWeave-AI-Native-Gaming-Engine"
$out = @()
foreach ($crate in $crates) {
  $srcPath = "$base\$crate\src"
  if (-not (Test-Path $srcPath)) { continue }
  $files = Get-ChildItem $srcPath -Filter *.rs -Recurse
  foreach ($f in $files) {
    # Skip dedicated test files (filename contains "test")
    if ($f.Name -match 'test') { continue }
    $lines = Get-Content $f.FullName
    $testStartLine = $lines.Count
    for ($i = 0; $i -lt $lines.Count; $i++) {
      if ($lines[$i] -match '^\s*#\[cfg\(test\)\]') {
        $testStartLine = $i
        break
      }
    }
    for ($i = 0; $i -lt $testStartLine; $i++) {
      $line = $lines[$i]
      # Match .unwrap() but exclude .unwrap_or variants, comments, doc comments
      if ($line -match '\.unwrap\(\)' -and
          $line -notmatch '\.unwrap_or\(' -and
          $line -notmatch '\.unwrap_or_default\(' -and
          $line -notmatch '\.unwrap_or_else\(' -and
          $line -notmatch '^\s*//' -and
          $line -notmatch '^\s*\*' -and
          $line -notmatch '//!' -and
          $line -notmatch '^\s*///') {
        $lineNum = $i + 1
        $trimmed = $line.Trim()
        $relName = $f.Name
        $out += "$crate|${relName}:${lineNum}: $trimmed"
      }
    }
  }
}
$out | Set-Content "$base\unwrap_audit_20crates.txt" -Encoding UTF8
Write-Host "Found $($out.Count) production unwrap calls"
Write-Host "Saved to unwrap_audit_20crates.txt"
