# Deterministic Quality Contract

AI agents must provide actual test and build evidence, test changes in expected behavior, and must never disable tests, change thresholds, or suppress failures to fabricate success. CI and policy changes require human review.

**Automated now:** patch whitespace check and headless CMake build and CTest regression suite.

Requiring status `quality` in branch protection is necessary to prevent bypass. This is not a claim of mutation-testing or coverage enforcement, which requires an instrumented baseline.
