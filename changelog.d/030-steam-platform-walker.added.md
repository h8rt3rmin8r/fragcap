The target-resolution cascade (issue #77) gained its platform-walker provider,
completing #77: Steam is now one optional provider feeding the shared resolver
rather than the spine of targeting. The walker makes a Steam-installed title's
install directory available to the resolver so the higher-precedence engine-rule
provider (S029) can name the socket-holding client from layout, and, when the
engine rule does not recognize the layout, it answers at its own precedence by
classifying the install directory's executables into a single client. It reuses
`fragcap-steam`'s existing library enumeration and the scaffold classifier
predicates, and reads the filesystem and registry only: no process handle, no
memory, no network. Every walker answer is stamped `heuristic-unverified` with
provenance `steam-library`, an honest name for the library walk and
install-directory classification it performs. The walker declines rather than
guess: it resolves only when exactly one plausible client remains after dropping
installers and launcher stubs; zero, several, or an unreadable install is a
decline (with the ambiguity or unreadable path recorded), and the cascade falls
through to runtime observation, which resolves the game from the live
socket-holding process. The provider lives in `fragcap-steam` (which already
depends on `fragcap-profile`; the reverse is forbidden by the dependency-direction
check), implementing the cascade's provider trait; the no-op stub in
`fragcap-profile` is retired and the CLI assembles the resolver with the real
walker. A resolved target gained a platform-walker origin naming the client and
the match rules the pipeline binds it by, and the resolver gained walker ambiguity
and unreadable notes. Steam's `steam://` managed launch is unchanged and stays a
convenience adapter. No dependency is added.
