$base = "C:\Users\pv2br\AstraWeave-AI-Native-Gaming-Engine"
$results = [System.Collections.ArrayList]::new()

Get-ChildItem -Path $base -Recurse -Filter "*.rs" -File | Where-Object {
    $_.FullName -notmatch '(cov_html|target|\.zencoder|docs[\\/])'
} | ForEach-Object {
    $file = $_.FullName.Replace("$base\", '')
    $lines = [System.IO.File]::ReadAllLines($_.FullName)
    for ($i = 0; $i -lt $lines.Count; $i++) {
        if ($lines[$i] -match '^\s*pub enum\s+(\w+)') {
            [void]$results.Add("$file|$($i+1)|$($Matches[1])")
        }
    }
}

$results | Sort-Object | Set-Content "$base\pub_enum_list.txt"
Write-Host "Found $($results.Count) pub enums"
