The distribution archive now also contains the barebones hint database, placed
beside the binary so the first-run bootstrap can seed the writable per-user copy
from it. The archive previously held only the binary, the license, and the notice.

The `--hint-db` option and the `FRAGCAP_HINT_DB` environment variable now fall
back to a per-user default (`%APPDATA%\fragcap\hint.db`) when neither is set,
rather than leaving hint resolution off. A path named explicitly keeps its exact
previous semantics: it is used as given and, when absent, is neither created nor an
error.
