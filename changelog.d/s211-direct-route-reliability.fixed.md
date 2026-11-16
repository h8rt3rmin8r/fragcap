<!-- spec-impact: 17, 26, 27 -->
Selected TCP direct-route SOCKS admission now runs outside the packet-processing worker under finite pending-flow and packet bounds. Its protected route evidence records the operation stage, elapsed time, and actual error kind, distinguishing delayed success, proxy refusal, and timeout while preserving unrelated packet pass-through and exact selected-packet dispositions.

The documentation site now uses Next.js 16.4.6 to clear the reported security advisory in its dependency audit.
