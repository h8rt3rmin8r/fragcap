The target resolution cascade (issue #77) gained its first general-purpose
provider: an engine rule that recognizes a game's socket-holding client from its
game engine's documented on-disk install layout, with no per-title data. It fills
the engine-rule provider slice S027 registered as a declining stub. Unreal Engine
is recognized by a `*-Win64-Shipping.exe` under a `Binaries\Win64` directory (the
client a root launcher stub relaunches), Unity by a `*_Data` directory and a
`UnityPlayer.dll` beside the player executable, and Ren'Py by a `renpy` directory
and `.rpa` archives. The provider reads the filesystem only: it opens no process
handle, reads no process memory, launches nothing, and ignores post-run artifacts
such as per-user AppData, which do not exist before the first launch. Every answer
is stamped `heuristic-unverified` with provenance `engine-rule`, never a higher
tier, because a documented on-disk convention is a good guess rather than an
authored fact. When a rule recognizes a layout but matches more than one candidate
client, the provider declines rather than pick one arbitrarily and records the
ambiguity, so the cascade falls through to runtime observation, which
disambiguates at runtime. A resolution request now carries an optional install
directory (which the S030 platform walker will populate unchanged), and a resolved
target gained an engine-rule origin naming the client executable and the match
rules the pipeline binds it by. No dependency is added, nothing is added to
`fragcap-core`, and the provider lives in `fragcap-profile` beside the rest of the
cascade.
