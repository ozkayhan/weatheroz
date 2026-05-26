# Weatheroz CLI - Core Principles & Unchanging Goal

This document defines our core principles and ultimate goal that will never change, regardless of the application's version (from v1.0 to v10.0 and beyond). Every contribution, every line of code added, and every architectural change must be evaluated in accordance with these goals.

---

## 🎯 Our Unchanging Goal

No matter what version of the application is running, our goal is always and under all conditions to ensure:

1. **Higher Performance:** Maximum efficiency in CPU and resource utilization.
2. **Higher Uptime:** Uninterrupted service, high fault-tolerance, and operational resilience under any conditions.
3. **More Accurate & Blended Data:** Harmonizing and blending weather data from different sources to produce the most accurate results, minimizing anomalous errors.
4. **Faster Execution:** Sub-millisecond execution and network response times through fully optimized parallel queries.
5. **Zero Dependency Focus:** Minimizing third-party dependencies to keep a lightweight, secure, and easily maintainable codebase.
6. **Lower Memory Footprint:** Optimizing memory management, preventing leaks, and ensuring the absolute lowest RAM consumption.
7. **Richer Features:** Innovative features that enhance user experience without ever compromising on performance.

---

## 🛡️ Adhering to the Principles

For every new feature or refactoring step, we must ask ourselves the following questions:

* *Does this change increase RAM or CPU consumption?*
* *Can we use the standard library instead to reduce or eliminate third-party dependencies?*
* *How can we increase fault tolerance and uptime?*
* *How can we optimize the consensus blending mechanism to improve data accuracy even further?*
