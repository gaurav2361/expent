## 2026-05-11 - [Intl Formatter Constructor Overhead]
**Learning:** Initializing `Intl.NumberFormat`, `Intl.DateTimeFormat`, or `Intl.RelativeTimeFormat` inside a render loop or formatting utility is extremely expensive (O(n) where n is the number of items being formatted). In a `DataTable` with 100 rows and 10 columns, this can lead to 1,000+ constructor calls per render, severely impacting TTI and frame rates.
**Action:** Always cache `Intl` formatters globally or via `useMemo` using a key derived from locale and configuration options. This reduces constructor overhead to O(1) for repeated patterns.

## 2026-05-11 - [Recursive DataTable Memoization]
**Learning:** In complex tables that support mobile "card" views, sub-components like `DataTableAccordionCard` and `SimpleCard` are often recreated during parent re-renders (e.g., when a global filter or sort state changes). Even with virtualization, the initial render and state transitions can be sluggish without explicit memoization of these leaf components.
**Action:** Apply `React.memo` to all table-related sub-components that receive stable props (like `row` and `column` config) to ensure the DOM is only touched when the underlying data actually changes.
