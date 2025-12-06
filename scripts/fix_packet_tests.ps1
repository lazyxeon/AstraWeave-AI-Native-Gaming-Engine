$content = Get-Content 'c:\Users\pv2br\AstraWeave-AI-Native-Gaming-Engine\astraweave-net\tests\integration\packet_loss_tests.rs' -Raw
# Fix the malformed pattern
$content = $content -replace 'MoveTo \{ speed: None,\s+x: ', 'MoveTo { x: '
# Add speed: None at the end of each MoveTo
$content = $content -replace 'MoveTo \{ x: (\d+), y: (\d+)(,)?\s*\}', 'MoveTo { x: $1, y: $2, speed: None }'
$content = $content -replace 'MoveTo \{ x: ([^\}]+), y: ([^\}]+)\}', 'MoveTo { x: $1, y: $2, speed: None }'
Set-Content 'c:\Users\pv2br\AstraWeave-AI-Native-Gaming-Engine\astraweave-net\tests\integration\packet_loss_tests.rs' -Value $content -NoNewline

Write-Host "Fixed packet_loss_tests.rs"
