# S211 candidate evidence

## Source and local checks

The first PR review on commit `9c047e7` identified three valid defects: pending helpers were omitted from flow capacity, a fragmented SOCKS reply could outlast the absolute helper deadline, and a timed-out physical test could leave descendants running. The candidate changes count pending flows before issuing a receipt, bound each socket read and write by the remaining deadline, and assign the gated physical test worker to a kill-on-close Windows Job Object before it can start descendants. A driver-free job probe verified descendant termination. The fragmented-reply test sends one byte every 30 ms against a 120 ms budget and rejects late completion. The full local gate and final-head CI are recorded separately once complete.

## Packaged driver observation

An optimized executable with the official runtime feature set and statically linked CRT was staged beside the signed WinDivert DLL, driver, license, and source archive. The staged executable SHA-256 was `8F333865FBC29B8FC96EAFA472AEAFC0011FBA2148B80DA7FB2A4CAEA2FF6E67`. The driver signature and component hashes matched the repository package contract. This staged folder is a portable candidate test arrangement, not an installed-product qualification or a certified release package.

The first elevated attempt stopped after a disposable peer fixture error: its accepted socket remained nonblocking and returned Windows error 10035 while reading the client request. The fixture was corrected, and its complete request/reply protocol passed a driver-free dry run before the next elevated test. The failed attempt remains diagnostic evidence and does not count toward acceptance.

One hidden elevated runner then completed three finite local-peer attempts. The peer was bound to this PC's non-loopback LAN address, but source and destination were on the same PC. Each selected client sent a 22-byte request, received the exact 20-byte reply, and sent a 21-byte final message. The peer counted one selected exchange and nine unrelated sentinels per attempt. The candidate reported one exact final-client generation, one complete bidirectional route, 43 upstream bytes, 20 downstream bytes, a successful helper outcome, zero route-event loss, and successful cleanup each time. Active sentinel samples were `[7,10,10]`, `[2,10,10]`, and `[10,10,10]` ms across the three attempts.

Each native bundle's compatibility observation and manifest agree on one transferred route, one consumed selected packet, original packet `unavailable`, and packet history `incomplete`. Each compatibility observation reports zero passive packet observations. A read-only pcapng block count on the first bundle found one section header, seven interface descriptions, seven interface statistics blocks, and zero enhanced packet blocks. Thus these runs prove the original-absent case and exact route transfer, but do not prove an original-present packet. The retained route, compatibility, manifest, cleanup, and debug-report readbacks are in the private local test area. Public evidence omits account paths, session identifiers, process IDs, and raw packet data.

## Acceptance still open

Issue #594 requires a controlled packaged original-present case, an original-absent case, and history truncation, including TCP/UDP negative cases and full artifact reconciliation. This candidate has physical original-absent TCP evidence only. A non-loopback IP literal on the same PC does not make the peer an independent external host; Windows treats same-machine traffic as loopback. The current test must not be used to close #594 or claim installed ESO success. The exact final PR head, hosted integration tests, and security findings also require terminal readback.
