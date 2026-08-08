`fragcap-core` carries the type and trait vocabulary from specification
sections 8.4 and 8.5: flow keys and the socket table matching key derived from
them, packets before and after attribution, attributions, timestamps,
statistics, three error types, and the five seams the rest of the workspace is
built against. Nothing captures, attributes, parses, or writes yet; this fixes
the shape those slices are written to.

Three constitution principles are now enforced by the types rather than by
documentation. A UDP attribution key carrying a remote endpoint is
unrepresentable, so the confident-wrong-attribution failure specification
section 8.4 warns about cannot be written. Every discard cause has its own named
counter and every total is computed, so a counter cannot drift from its parts.
An unattributed packet is distinguishable from one nobody tried to attribute.
