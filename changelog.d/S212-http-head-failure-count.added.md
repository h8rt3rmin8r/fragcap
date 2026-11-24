<!-- spec-impact: 1, 13.7, 26.3, 27.3 -->
Native and terminal connection diagnostics now show the exact number of HTTP/1.1 head bytes observed before a read error, partial EOF, or header-size refusal. The bundle debug report summarizes counted failures from native evidence without retaining partial head content; complete heads and clean zero-byte EOF omit the failure-only count and report row.
