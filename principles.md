# Weather CLI - Core Principles & Our Unalterable Mission

This document defines the core principles and ultimate goals of our project, which shall remain constant regardless of the application's version (from v1.0 to v10.0 and beyond). Every contribution, code addition, and architectural modification must be evaluated with respect to these principles.

---

## 🎯 Our Unalterable Mission (Goals)

No matter what version the application is on, our ultimate target is to constantly provide:

1. **Higher Performance:** Maximum efficiency in CPU usage and physical system resources.
2. **Superior Uptime:** Uninterrupted service, exceptional fault tolerance, and resilience in any operational environment.
3. **Accurate & Real Data:** Blending and reconciling data from diverse external weather sources to filter out anomalous errors and achieve optimal precision.
4. **Blazing Speed:** Sub-millisecond initialization and processing times, fully utilizing optimized asynchronous network patterns.
5. **Pruned Dependencies (Minimal Footprint):** Striking a careful balance by minimizing external crate dependencies to only core, active libraries (e.g. `tokio`, `ratatui`) to ensure a lightweight, secure, and easily maintainable codebase.
6. **Optimized RAM Footprint:** Active memory management to completely prevent leaks and run with the lowest possible RAM footprint.
7. **Premium Features:** Value-adding features that enrich the developer experience without sacrificing stability or performance.

---

## 🛡️ Adhering to the Principles

For every new feature or refactoring task, we must continuously ask ourselves:
* *Does this change increase CPU or RAM consumption unnecessarily?*
* *Can we reduce our dependency tree by leveraging standard library features or lightweight built-ins?*
* *How can we enhance error resilience and keep the system active in offline scenarios?*
* *How can we refine the consensus-blending mechanism to improve the authenticity of the reported data?*
