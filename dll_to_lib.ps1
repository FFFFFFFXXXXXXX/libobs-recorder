# run in VisualStudio Developer Powershell
Write-Output "LIBRARY obs" > obs.def;
Write-Output "EXPORTS" >> obs.def;
foreach ($line in (dumpbin /exports obs.dll | Select-Object -skip 19 )) { Write-Output (-split $line | Select-Object -Index 3) >> obs.def };
lib /def:obs.def /machine:x64 /out:obs.lib
