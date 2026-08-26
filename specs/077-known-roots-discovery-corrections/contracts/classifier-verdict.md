# Contract: Container-Aware Classifier Verdict

`DirectoryClassifier::classify` returns one exhaustive control verdict plus coverage warnings.

1. `Hit` means emit exactly one candidate and do not enumerate beneath it.
2. `Container` means emit no candidate. The caller attempts descent only within its declared bound.
3. `Miss` means emit no candidate and follows the existing miss accounting and bounded descent behavior.
4. `SignatureClassifier` returns `Container` only when observed findings name at least two distinct engine-category products.
5. Repeated same-product findings and non-engine findings do not satisfy rule 4.
6. Coverage warnings and incomplete scan state survive independently of all three verdicts.

The verdict does not persist a container or claim that its children were examined. Traversal and the discovery account make that separate statement.
