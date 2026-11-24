# Strips one leading '+' from every line: undoes a copy-paste from a diff view.
# No -Path: rewrites the clipboard. With -Path: rewrites that file in place.
param([string]$Path)

$utf8 = [System.Text.UTF8Encoding]::new($false)

if ($Path) {
    $full = (Resolve-Path -LiteralPath $Path).ProviderPath
    $text = [System.IO.File]::ReadAllText($full)
} else {
    $text = Get-Clipboard -Raw
}

if ([string]::IsNullOrEmpty($text)) {
    Write-Error 'nothing to process (empty clipboard or file)'
    exit 1
}

$count = [regex]::Matches($text, '(?m)^\+').Count
$result = [regex]::Replace($text, '(?m)^\+', '')

if ($Path) {
    [System.IO.File]::WriteAllText($full, $result, $utf8)
    Write-Host "removed $count leading '+' in $Path"
} else {
    Set-Clipboard -Value $result
    Write-Host "removed $count leading '+' on the clipboard"
}
