<!-- spec-impact: none -->
A capture now contains the target's traffic. It did not before: the first real
end-to-end run produced a file that was 91 percent other processes' traffic,
because nothing in the write path made the scope decision that specification
section 12.3 places in userspace, and the narrowed kernel filter cannot engage
until the target opens its first socket, which on a launcher-mediated title is
tens of seconds after capture begins. Output is scoped to the target by default;
`--scope profile` widens it to everything the profile binds and `--scope all`
restores the previous behavior for correlating a target against the rest of a
machine. Everything excluded is counted and reported, in two terms rather than
one, so a packet dropped because it belonged to something else is
distinguishable from a packet dropped because attribution had not yet named it.
The completion summary also now reports what was written per process, which is
the breakdown that would have made this defect visible on the first run rather
than on a manual inspection of the file afterwards.

The run also narrates itself truthfully. It used to print `filter narrowed to 0
endpoint(s)` and then go silent, a line that means the opposite of how it reads
(zero means no narrowing has happened and everything is still being captured),
sampled at the one instant where zero is close to guaranteed, and never updated
afterwards. It now says that capture is machine-wide while the target opens its
first connection, and says again when capture becomes target-only, from the
transition rather than from a sample. Machine-readable output receives an event
per narrowing instead of a single reading of zero.
