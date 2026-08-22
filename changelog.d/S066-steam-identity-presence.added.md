<!-- spec-impact: 15.8 -->
A registered Steam title now keeps all three of its names: the storefront
display name, the raw installed folder name, and the observed launch
executable, none reconstructed from another. A title whose folder name diverges
from its current storefront title, which happens more often than expected (11
of 34 titles on a sampled library), is now findable by any of the three: typing
a substring of the folder name or the executable resolves it exactly as a
handle or an exact name already did. `targets show` names both the display name
and the folder name when they genuinely diverge, and stays quiet when the
difference is only casing, whitespace, or a truncated subtitle. Handle
derivation also stops silently dropping `&`; a name like `Trapped with Ivy &
Piper` now derives `trapped_with_ivy_and_piper` rather than reading as if a word
were missing.
