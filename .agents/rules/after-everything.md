---
trigger: always_on
---

Maintain strict adherence to KISS, DRY, YAGNI, SOLID, SoC, LoD, TDD, and the Boy Scout Rule. Deliver modular, clean, and comprehensively tested code. Prioritize readability and simplicity over cleverness; perform optimization only when profiling proves it is necessary.

Strictly avoid the following antipatterns:

God Objects: Do not create classes or functions that handle too many responsibilities.

Hardcoding: Never hardcode configuration, paths, or secrets.

Spaghetti Code: Do not write tangled, tightly coupled, or opaque logic.

Dead Code: Remove unused variables, imports, or commented-out code immediately.

Premature Optimization: Do not sacrifice clarity for unverified performance gains.

Hidden Dependencies: Do not hide side effects or dependencies; ensure interfaces are explicit.