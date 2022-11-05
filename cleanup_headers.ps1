Get-ChildItem './libobs-sys/libobs_headers' -File -Recurse | Where-Object { $_.extension -ne '.h' } | Remove-Item
