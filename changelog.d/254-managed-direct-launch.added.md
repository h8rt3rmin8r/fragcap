<!-- spec-impact: 7.1, 16.4, 17.2.1, 19.2, 19.4, 29 -->
Capture and Deep Capture can now start an eligible stored direct executable from one prevalidated path, working directory, and argument vector. Deep Capture applies proxy variables only to that child, while warm direct targets and unsafe or stale paths remain side-effect-free refusals.
