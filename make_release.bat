@echo off
powershell -Command "& {$file1 = 'target\release\nightjar.exe'; $file2 = 'target\release\libnightjar.dll'; $zipfile = 'nightjar_monocular.zip'; Compress-Archive -Path $file1, $file2 -DestinationPath $zipfile;}"