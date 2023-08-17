echo off

git add .
git commit -m "Update"
git push
powershell -c (New-Object Media.SoundPlayer 'C:\Windows\Media\Windows Foreground.wav').PlaySync();