<!-- spec-impact: 13, 17, 26, 27 -->
Selected TCP direct-route SOCKS admission now runs outside the packet-processing worker under finite pending-flow and packet bounds. Its protected route evidence records the operation stage, elapsed time, and actual error kind, distinguishing delayed success, proxy refusal, and timeout while preserving unrelated packet pass-through and exact selected-packet dispositions.

Direct-route candidate resolution indexes socket events by tuple and reports bounded queue and hold timing. A complete captured first packet can acquire distinct `direct-route` attribution only after exact packet, tuple, time, and causal-generation agreement; unmatched traffic remains unattributed.

The documentation site now uses Next.js 16.4.6 to clear the reported security advisory in its dependency audit.
