---
name: ui-visual-verify
description: >-
  Rebuilds the VectorCraft/Trowel desktop executable, launches it with a TCP control port,
  captures a live UI screenshot (ui.screenshot), and verifies the actual rendered window
  (theme, layout, unclipped controls, and layer state) before reporting UI work as complete.
---

# Live UI Visual Verification Workflow

Always run this workflow after modifying any UI code in `crates/ui-egui` or `apps/vectorcraft`.

## Steps

1. **Stop any running `vectorcraft.exe` instance first** (so exiting an old instance does not overwrite `%APPDATA%\VectorCraft\ui.json`):
   ```powershell
   Stop-Process -Name "vectorcraft" -Force -ErrorAction SilentlyContinue
   ```
2. **Rebuild the desktop binary**:
   ```powershell
   cargo build -p vectorcraft
   ```
3. **Launch with `--control`, capture `ui.screenshot`, and close**:
   ```powershell
   $proc = Start-Process -FilePath "d:\vector\target\debug\vectorcraft.exe" -ArgumentList "--control","7985" -PassThru
   Start-Sleep -Seconds 3
   $shotPath = "$env:TEMP\verify_trowel.png"
   try {
       $client = New-Object System.Net.Sockets.TcpClient("127.0.0.1", 7985)
       $stream = $client.GetStream()
       $writer = New-Object System.IO.StreamWriter($stream)
       $reader = New-Object System.IO.StreamReader($stream)
       $writer.AutoFlush = $true
       $writer.WriteLine('{"id":1,"method":"engine.execute","params":{"command":"trowel.newPlan","params":{}}}')
       $null = $reader.ReadLine()
       Start-Sleep -Milliseconds 600
       $writer.WriteLine("{`"id`":2,`"method`":`"ui.screenshot`",`"params`":{`"path`":`"$($shotPath -replace '\\','/')`"}}")
       $null = $reader.ReadLine()
       $client.Close()
   } finally {
       Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
   }
   ```
4. **Inspect the screenshot with `view_file`** to verify:
   - Warm ivory `Brightness::Plate` theme (`#FAF9F5` / `#F5F4ED` / `#EDEBE2`).
   - Only 1 initial working layer in `범례 · 레이어`.
   - No clipped buttons or overlapping text at the bottom of the toolbar or panels.
