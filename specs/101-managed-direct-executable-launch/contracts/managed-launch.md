# Contract: Managed Launch

## Preparation

`prepare_target_launch(target)` is side-effect-free except for filesystem metadata reads. It returns exactly one prepared launch or a typed refusal. It never starts a process, opens Capture, starts a proxy, changes trust, writes target data, or changes ambient environment.

Steam targets retain the existing Steam protocol request. Direct targets require one resolved Windows client and either an install root or an authored absolute executable whose parent supplies that root. The executable must exist as a file beneath that root.

## Environment Overlay

`with_environment(entries)` returns a direct launch with the same program, working directory, and arguments plus explicit child-only environment values. Steam requests refuse an environment overlay because a protocol handler cannot guarantee that a running platform inherits it.

## Execution

`execute()` issues exactly the prepared variant:

- Steam delegates to the existing Steam protocol launcher.
- Direct creates the exact executable with the exact working directory, individual arguments, and individual environment entries.

No shell, script host, association handler, target inspection, system proxy, or target-store resolution occurs in direct execution.

## Failure

Preparation errors are usage-class refusals before effects. Execution errors are runtime failures. If Deep Capture already owns proxy or trust resources, its session coordinator records the launch failure, stops applicable resources, attempts bounded cleanup, and returns a non-complete terminal result.

## Ownership

Execution does not retain a child-process controller. The process and descendants are observed through Capture's existing watcher and attributed through the existing socket-table path.
