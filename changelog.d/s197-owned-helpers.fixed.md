<!-- spec-impact: 10, 17.4, 27.3 -->
After client observation, route-owner cleanup now requests bounded close and force for exact receipt-backed session-created helpers, including helpers absent from declared launch stages. Missing ownership proof leaves an unresolved cleanup obligation instead of silently releasing the route.
