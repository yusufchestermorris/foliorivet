param([Parameter(Mandatory=$true)][string]$Source,[Parameter(Mandatory=$true)][string]$Output)
$ErrorActionPreference='Stop'
$sourcePath=(Resolve-Path -LiteralPath $Source).Path
$outputPath=[IO.Path]::GetFullPath($Output)
if(Test-Path -LiteralPath $outputPath){throw 'Choose a new review directory'}
New-Item -ItemType Directory -Path $outputPath | Out-Null
$hash=(Get-FileHash -LiteralPath $sourcePath).Hash
$word=$null;$doc=$null
try {
 $word=New-Object -ComObject Word.Application
 $word.Visible=$false
 $word.DisplayAlerts=0
 $doc=$word.Documents.Open($sourcePath,$false,$true,$false)
 $saved=Join-Path $outputPath 'review.docx'
 $doc.SaveAs2($saved,16)
 $doc.Close(0);[void][Runtime.InteropServices.Marshal]::ReleaseComObject($doc);$doc=$null
 $doc=$word.Documents.Open($saved,$false,$true,$false)
 $doc.Repaginate();$pages=$doc.ComputeStatistics(2)
 $doc.ExportAsFixedFormat((Join-Path $outputPath 'review.pdf'),17)
 if((Get-FileHash -LiteralPath $sourcePath).Hash -ne $hash){throw 'Original changed'}
 @{word_version=$word.Version;word_build=$word.Build;source_sha256=$hash;pages=$pages;automation='open/save/reopen/export PASS';independent_prompt_review='PENDING'} | ConvertTo-Json | Set-Content -Encoding utf8 (Join-Path $outputPath 'result.json')
 Write-Output "PASS $pages pages"
} finally {
 if($doc){$doc.Close(0);[void][Runtime.InteropServices.Marshal]::ReleaseComObject($doc)}
 if($word){$word.Quit();[void][Runtime.InteropServices.Marshal]::ReleaseComObject($word)}
}
