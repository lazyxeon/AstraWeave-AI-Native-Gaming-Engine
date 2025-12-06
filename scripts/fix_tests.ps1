$content = Get-Content 'c:\Users\pv2br\AstraWeave-AI-Native-Gaming-Engine\astraweave-net\tests\integration\sync_tests.rs' -Raw
$content = $content -replace 'MoveTo \{ x: (\d+), y: (\d+) \}', 'MoveTo { x: $1, y: $2, speed: None }'
Set-Content 'c:\Users\pv2br\AstraWeave-AI-Native-Gaming-Engine\astraweave-net\tests\integration\sync_tests.rs' -Value $content -NoNewline

$content2 = Get-Content 'c:\Users\pv2br\AstraWeave-AI-Native-Gaming-Engine\astraweave-net\tests\integration\packet_loss_tests.rs' -Raw
$content2 = $content2 -replace 'MoveTo \{', 'MoveTo { speed: None,'
Set-Content 'c:\Users\pv2br\AstraWeave-AI-Native-Gaming-Engine\astraweave-net\tests\integration\packet_loss_tests.rs' -Value $content2 -NoNewline

Write-Host "Fixed ActionStep::MoveTo in test files"
