# Web Booking & Play Integration Plan

This document outlines the scope, current progress, and technical rollout plan for introducing "Booking" and "Play" functionalities directly into the Expent web application (`apps/web`).

---

## 1. Overview & Vision

Currently, Expent focuses primarily on financial telemetry: parsing receipts, reconciling bank statements, and tracking P2P cash flows. The **Booking & Play** integration marks a shift from purely *tracking* expenses to *initiating* active transactions natively on the platform.

### Feature Definitions:
- **Booking**: Enabling users to natively reserve temporal assets (e.g., Hotel stays, Flights, Trains, or Workspaces) with the ledger immediately recognizing and splitting the cost beforehand rather than retroactively.
- **Play**: Integrating sporting and recreational reservations (e.g., Turf bookings, generic sport events) to seamlessly split event costs among friends prior to the actual activity occurring.

---

## 2. What We Have Done (Current State)

Within the `apps/web` environment, the foundational infrastructure for such modular extensions is already stable:

1. **Robust UI Libraries**: `shadcn/ui` components combined with Tailwind CSS inside the Vite/React architecture are fully deployed, meaning modal pickers for dates or sports venues can be rapidly prototyped.
2. **P2P Split Engine**: Expent’s ledger already flawlessly resolves `p2p_requests` and `split_transaction_handler` operations. If a turf booking costs ₹2000, requesting ₹500 from 3 other `users` or `contacts` is natively supported.
3. **Frontend Routing**: The implementation of TanStack router inside `apps/web/src/routes` provides a highly scalable way to establish parallel routes (like `/booking` isolated from `/dashboard`).

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

### Phase 3: Web App Integration (`apps/web`)

**1. The "Booking & Play" Hub Router**
Create a new primary navigational route.
- `/booking` Main Dashboard
- `/booking/play` -> Sub-route primarily prioritizing turf maps, available sports, and squad formations.
- `/booking/travel` -> Sub-route for transport modalities.

**2. State & Hooks Layer**
- Author dedicated TanStack-Query hooks (`useBookings`, `useCreateEvent`).
- Ensure any `play` session created immediately populates the Notifications side-nav if a P2P contribution is required from the user's friend.

**3. The UI/UX Flow**
- Revert traditional tracking: Instead of uploading an OCR receipt *after* playing, the user clicks "Book Turf" -> selects 5 friends -> Turf is booked, and 5 separate `PENDING` transactions automatically render on each friend’s dashboard.

---

## 4. Open Architecture Questions for Future Strategy

As we move toward development, key architecture choices remain:
- **Third-Party API Integrations**: Are we directly integrating with real booking API aggregators (e.g., MakeMyTrip integrations, local Turf API gateways)? Or building an internal mock engine first?
- **Financial Gateway**: When booking directly, who processes the core gateway fee initially? Does Expent need a Stripe / PayPal Native integration to handle initial upfront card holds?
