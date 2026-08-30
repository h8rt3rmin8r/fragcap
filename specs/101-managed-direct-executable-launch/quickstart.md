# Quickstart: Managed Direct-Executable Launch

Register an executable target through the existing target workflow, then launch it under Capture:

```powershell
fragcap targets add --exe 'C:\Games\Example\example.exe'
fragcap capture example --launch --out example.fcapng
```

For a target whose current `direct-exe-cold` compatibility evidence supports scoped proxy routing, launch the same stored target under Deep Capture:

```powershell
fragcap deep-capture example --launch --trust-ca --bundle example-session
```

The direct executable is resolved and validated during preparation. Capture starts it only after the watcher and packet path are armed. Deep Capture applies proxy variables only to that child and its descendants. Warm direct targets, ambiguous launch entries, missing files, and paths outside the stored install root are refused.
