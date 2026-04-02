# Dashboard Booking & Play Integration Plan

This document outlines the scope, current progress, and technical rollout plan for introducing "Booking" and "Play" functionalities directly into the Expent Next.js Dashboard (`apps/dashboard`).

---

## 1. Overview & Vision

Currently, Expent focuses primarily on financial telemetry: parsing receipts, reconciling bank statements, and tracking P2P cash flows. The **Booking & Play** integration marks a shift from purely *tracking* expenses to *initiating* active transactions natively on the platform.

### Feature Definitions:
- **Booking**: Enabling users to natively reserve temporal assets (e.g., Hotel stays, Flights, Trains, or Workspaces) with the ledger immediately recognizing and splitting the cost beforehand rather than retroactively.
- **Play**: Integrating sporting and recreational reservations (e.g., Turf bookings, generic sport events) to seamlessly split event costs among friends prior to the actual activity occurring.

---

## 2. What We Have Done (Current State)

Within the `apps/dashboard` environment, the foundational infrastructure is built on the Next.js App Router:

1.  **Modern UI Architecture**: Built using Next.js 16, React 19, and Tailwind CSS 4. The application uses `@expent/ui` workspace packages for shared components, ensuring design consistency.
2.  **Auth-Protected Dashboard**: A centralized auth guard is implemented in `src/app/(dashboard)/layout.tsx`, handling session validation and redirects seamlessly for all sub-routes.
3.  **P2P Split Engine Ready**: The dashboard already manages `transactions` and `shared-ledgers`. New booking features will leverage existing `P2P & Sharing` logic for splitting costs upstream.
4.  **SPA Routing**: Navigational links use Next.js `<Link>` for instant, no-reload transitions, providing a premium "app-like" experience for complex booking flows.

---

## 3. What We Are Going to Do (Execution Plan)

To bring **Booking & Play** to life, we will execute across three different layers incrementally.

### Phase 1: Data Modeling & Schema Expansion (`crates/db`)
Before the frontend interacts with APIs, the persistence layer needs enhancement.
- Create a `bookings` table explicitly linking a `transaction_id` to external merchant reservation data (e.g., Dates, Venue Name, Duration).
- Define `BookingCategory` Enum: `TRAVEL`, `PLAY`, `HOTEL`.
- Establish `play_events` detailing participant caps (e.g., Max 10 players for Turf) tied natively to internal `groups`.

### Phase 2: Core Server Routing (`apps/server`)
Exposing the capabilities securely to the frontend.
- `GET /api/booking/vendors` -> Pulling partnered vendor APIs or a mocked internal vendor list for play.
- `POST /api/booking/initiate` -> The handler takes a request, generates a `transaction`, splits it among the targeted friends, creates the `p2p_requests`, and places the booking in a "PENDING CANCELLATION/SETTLEMENT" status state.

### Phase 3: Dashboard Integration (`apps/dashboard`)

**1. The "Booking & Play" Route Structure**
Create new routes within the `(dashboard)` group:
- `src/app/(dashboard)/booking/page.tsx` -> Main Discovery Hub.
- `src/app/(dashboard)/booking/play/page.tsx` -> Turf maps, available sports, and squad formations.
- `src/app/(dashboard)/booking/travel/page.tsx` -> Transport modalities.

**2. State & Hooks Layer**
- Utilize React Query (already configured) for async data fetching (`useBookings`, `useCreateEvent`).
- Implement real-time updates for `play` sessions to notify friends when a contribution is required.

**3. The UI/UX Flow**
- Proactive Splitting: User clicks "Book Turf" -> selects 5 friends -> Turf is booked, and 5 separate `PENDING` transactions automatically render on each friend’s dashboard.
- Integrated Sidebar: Update `src/components/app-sidebar.tsx` to include "Booking" as a primary navigational group.

---

## 4. Open Architecture Questions for Future Strategy

As we move toward development, key architecture choices remain:
- **Third-Party API Integrations**: Are we directly integrating with real booking API aggregators (e.g., local Turf API gateways)? Or building an internal mock engine first?
- **Financial Gateway**: When booking directly, does Expent need a Stripe / PayPal Native integration to handle initial upfront card holds?
