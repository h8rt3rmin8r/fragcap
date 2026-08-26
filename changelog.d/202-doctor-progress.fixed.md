<!-- spec-impact: 26.3 -->

Fixed `fragcap doctor` looking hung during slow first-run readiness checks by
printing named interactive progress on stderr while keeping the final human and
JSON report outputs unchanged.
