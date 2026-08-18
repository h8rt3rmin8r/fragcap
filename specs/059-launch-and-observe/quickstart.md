# Quickstart: Launch-and-observe promotion

## The operator flow

```text
# 1. Register a game whose socket holder you are not sure of.
fragcap targets add "My Game" --exe launcher.exe --socket-holder no
#   -> stored with an unresolved launch chain (names launcher.exe, no client)

# 2. Capture it. Before S059 this was refused. Now it captures in observe mode.
fragcap capture "my-game" --launch          # Steam-anchored: launches via Steam
#   or start the game yourself, then:
fragcap capture "my-game"
#   -> observes the socket-holding process, writes the capture,
#      and promotes the target to a resolved client at verified fidelity.

# 3. Capture it again. The target is now resolved; it addresses the client directly.
fragcap capture "my-game"
```

If the run observes no traffic for the target, the capture still completes and the
target is left exactly as registered. Nothing is promoted on no observation.

## The offline proof (CI-visible)

The whole promotion decision runs without a capture driver, elevation, or a game,
over the scripted-attributor substrate:

```text
# A process tree where a launcher spawns a child that holds the sockets.
cli_capture.rs:
  register "My Game" --exe launcher.exe --socket-holder no  (unresolved)
  capture "my-game" --offline
    --replay-source  <fixture>.pcap
    --attr-script    <child holds the sockets>.script
    --process-script <launcher spawns child>.procscript
  assert: exit 0, packets attributed to the child image
  assert: the stored target now has launch = [{ executable: <child>, role: client }]
          and fidelity = verified

# The no-observe branch: an attr-script that resolves nothing.
  assert: exit 0, the stored target is unchanged (still unresolved)
```

## What stays Tier 2 (not in CI)

The literal `steam://run` launch of an unresolved Steam-anchored target. Every other
line, including the promotion decision and write-back, is offline-verifiable.
