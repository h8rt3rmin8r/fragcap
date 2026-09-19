<!-- spec-impact: 13.7, 17.2.1, 25.5 -->

**2026-09-19** Keep the existing application queue, performance registry and four-field authorization deadline plan. Establish writer readiness before sink publication, and make structured bounded drain completion the authority for terminal observations instead of enlarging buffers or adding timing grace. Preserve the stable observation method and add a defaulted structured drain method so existing adapter implementations and callers remain source-compatible.
