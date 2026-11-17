# Timesheet — UI/UX Wireframe & Material 3 Design System

**Status:** Design proposal for the next UI iteration (app v0.14+)
**Last updated:** 2026-11-17
**Companion mockup:** [wireframe.html](wireframe.html) — open in a browser for the layered, clickable reference.
**Applies to:** `src/ui/` (egui/eframe), `src/tray.rs`, `src/settings.rs`

---

## 1. Purpose

This document defines a modern, layered desktop UI for Timesheet. It is the reference for
future UI work, not a description of the current implementation. The goals are:

- Make time entry feel like a **two-click micro-interaction**, not a form-filling chore.
- Replace the “flat panel” look with **Material 3 tonal surfaces, outlines, elevation, and
  restrained shadows**.
- Move all create/edit forms into **focused popup dialogs** (basic dialogs on desktop).
- Give large and ultra-wide windows a **purposeful multi-pane layout** instead of stretched,
  empty cards.
- Offer a **full Material 3 theme** for the whole app, alongside the existing compact
  “Classic” style.
- Treat the **live timer** and the **Windows notification-area (tray) experience** as
  first-class product surfaces, backed by platform and UX research.

### Non-goals

- No online sync, accounts, or collaboration.
- No mobile or touch-first layout; the target is Windows desktop (mouse + keyboard first,
  touch-capable at 48 dp targets).
- The OS tray menu remains native Win32; it can be configured but not visually themed.

---

## 2. Design principles

| # | Principle | What it means in Timesheet |
|---|-----------|----------------------------|
| 1 | **One primary action per surface** | Dashboard promotes Start/Stop; Entries promotes New entry; timer dialog promotes Save. |
| 2 | **Layer, don’t decorate** | Depth comes from surface-container roles + 1 dp outlines; shadows only for floating layers (dialogs, menus, snackbars). |
| 3 | **Fast path first** | The omnibox and recent projects are always one click away. Every extra field is optional until review. |
| 4 | **Never trust a running timer blindly** | Stop opens a review dialog; long sessions and idle periods get calm, actionable prompts. |
| 5 | **Calm status, strong state** | Running, paused, billable, and error states use icon + text + colour, never colour alone. |
| 6 | **Width is a feature** | Wider windows reveal an insights rail, detail pane, or extra columns instead of stretching cards. |
| 7 | **Windows-native where it counts** | Tray menu order, close-to-tray behaviour, and notifications follow Microsoft’s notification-area guidance. |
| 8 | **Themeable by token** | Every colour, radius, and type size comes from a token; the full Material 3 theme is a token swap, not a rewrite. |

---

## 3. Research summary

### 3.1 Material Design 3

Key findings used in this wireframe:

- **Breakpoints:** compact `< 600 dp`, medium `600–839 dp`, expanded `840–1199 dp`,
  large `1200–1599 dp`, extra-large `1600 dp+`. Layouts move from one pane to two or
  three panes as width grows. ([M3 breakpoints](https://m3.material.io/foundations/layout/breakpoints))
- **Navigation:** use a navigation rail for medium and larger windows; collapsed rails are
  ~80–96 dp, expanded rails are 220–360 dp, with 64 dp items and a 56 × 32 dp active
  indicator. ([M3 navigation rail](https://m3.material.io/components/navigation-rail/specs))
- **Dialogs:** basic dialogs are 280–560 dp wide, 28 dp corner radius, 24 dp padding,
  24 dp between body and actions, 8 dp between buttons. Desktop uses basic dialogs, not
  full-screen dialogs. ([M3 dialogs](https://m3.material.io/components/dialogs/specs))
- **Elevation:** six levels (0, +1, +2, +3, +4, +5) mapping to 0/1/3/6/8/12 dp. M3 prefers
  **tonal surface roles** over shadows; shadows/scrims are for protection and focus.
  ([M3 elevation](https://m3.material.io/styles/elevation))
- **Controls:** buttons are 40 dp high (small/default), text fields 56 dp, cards use 12 dp
  radius and 16 dp padding, FAB 56 dp. Minimum interactive target is 48 × 48 dp.
  ([M3 buttons](https://m3.material.io/components/buttons/specs),
  [M3 text fields](https://m3.material.io/components/text-fields/specs))
- **Type scale:** 15 baseline roles from display (57/45/36) through headline, title, body,
  to label (14/12/11). Use **tabular numerals** for clocks and durations.
  ([M3 typography](https://m3.material.io/styles/typography))

### 3.2 Timer UX research

Findings from time-tracking product research and UX guidance:

- Start/stop must be **the largest, most obvious target**; users should be able to start in
  one decision and roughly one click. ([Koder.ai](https://koder.ai/blog/build-mobile-app-time-tracking-productivity))
- **Quick start without a project** is better than forcing setup; let people categorise
  afterwards. Show 5–10 **recent projects** at the top of pickers and offer **one-tap
  resume** from history. ([Koder.ai](https://koder.ai/blog/build-mobile-app-time-tracking-productivity))
- A tracker is only as good as its timer: persist a **start timestamp** and compute elapsed
  time from the clock instead of relying on a background tick. Handle sleep/restart and
  long sessions explicitly. ([Koder.ai](https://koder.ai/blog/build-mobile-app-time-tracking-productivity))
- Show a clear **“currently tracking” state**; use text + icon, not colour alone. Provide
  gentle reminders for forgotten timers and missing entries.
  ([Koder.ai](https://koder.ai/blog/build-mobile-app-time-tracking-productivity),
  [Ignite Solutions](https://www.ignitesol.com/design-thinking-process-and-the-ux-design-of-our-time-tracking-app/))
- **Elapsed time is psychologically heavy**; show seconds only when useful and use
  anchors/ranges for longer waits. For a work timer, a calm `HH:MM` display with seconds
  available on hover or in the mini timer is enough.
  ([Prototypr — expressing time](https://blog.prototypr.io/expressing-time-in-ui-ux-design-5-rules-and-a-few-other-things-eda5531a41a7))
- **Inline duration pickers** and natural-language entry reduce friction; keep the duration
  field forgiving (`90`, `1.5`, `1:30`, `1h30m`, `45m`).
  ([Ignite Solutions](https://www.ignitesol.com/design-thinking-process-and-the-ux-design-of-our-time-tracking-app/))

### 3.3 Windows notification area (tray)

Microsoft’s guidance for notification-area icons:

- **Left click** should show what users most likely want: a flyout, dialog, or the main
  window. **Right click** shows the context menu; the default command is first and bold.
  ([Microsoft — Notification Area](https://learn.microsoft.com/en-us/windows/win32/uxguide/winenv-notification))
- Prefer a **lightweight popup/flyout** over a dialog for common settings; dismiss it when
  clicking outside. Show common tasks directly in the flyout.
- The context menu should contain **Open (default, bold), secondary commands, Options, Exit**.
  Remove items that do not apply instead of disabling them.
- Don’t flash the icon. Use a notification only when the event deserves interruption.
- Offer **“Minimize to notification area”** and **“Display icon in notification area”** as
  opt-in settings. Removing the icon should not stop the underlying feature.
  ([Microsoft — Notification Area](https://learn.microsoft.com/en-us/windows/win32/uxguide/winenv-notification))
- In user-facing copy, Microsoft prefers **“notification area”** rather than “system tray”.
  Keep “system tray” only as a search alias in settings.

Rust implementation: the [`tray-icon`](https://crates.io/crates/tray-icon) crate supports
Windows + winit/egui, with `TrayIconEvent` and `MenuEvent` receivers. The current app already
uses it; the design below refines the menu and state model rather than replacing it.

---

## 4. Material 3 foundation

### 4.1 Layered surface model

Depth is communicated by **role + outline + elevation**, not by giving every card a shadow.

```text
┌─────────────────────────────────────────────────────────────────────┐
│ L0  Window canvas            surface / surface-dim                  │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │ L1  Navigation rail      surface-container-low                 │  │
│  │  ┌─────────────────────────────────────────────────────────┐  │  │
│  │  │ L2  Content cards      surface-container-low/high       │  │  │
│  │  │     1 dp outline-variant · 12 dp radius · 16 dp padding │  │  │
│  │  │  ┌───────────────────────────────────────────────────┐  │  │  │
│  │  │  │ L3  Sticky filter/timer bars                     │  │  │  │
│  │  │  │     surface-container · 1 dp outline · 16 dp      │  │  │  │
│  │  │  └───────────────────────────────────────────────────┘  │  │  │
│  │  └─────────────────────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────────────────────┘  │
│        L4  Dialog / menu: surface-container-high · 28 dp · shadow   │
│        L5  Snackbar / tooltip: inverse-surface · 4–8 dp             │
└─────────────────────────────────────────────────────────────────────┘
```

| Layer | Component | Colour role | Elevation | Shape |
|-------|-----------|-------------|-----------|-------|
| L0 | Window background | `surface` | 0 | — |
| L1 | Nav rail / side panel | `surface-container-low` | 0 | 0 dp (rail), 16 dp if modal |
| L2 | Cards, table container, KPI tiles | `surface-container-low` / `surface-container` | 0–1 dp | 12 dp |
| L3 | Sticky filter bar, timer bar, bulk-action bar | `surface-container` / `surface-container-high` | 1–2 dp | 12–16 dp |
| L4 | Dialog, menu, popover | `surface-container-high` | 3 (6 dp) | 28 dp (dialog), 12 dp (menu) |
| L5 | Snackbar, tooltip | `inverse-surface` | 4 (8 dp) | 4–8 dp |

**Rules**

- Never place two same-colour surfaces directly on top of each other without a 1 dp
  `outline-variant` or a tonal step.
- Do not use drop shadows on ordinary cards; reserve them for floating layers.
- A modal always gets a scrim (`scrim` at 32 % light / 60 % dark) and a focus trap.
- Use `surface-container-highest` sparingly: table headers, selected rows, hovered menus.

### 4.2 Colour roles

Use M3 role names, not raw hex, in code. Suggested seed: **blue** (`#2563EB`) with a teal
tertiary for timer/billable success.

| Role | Light | Dark | Used for |
|------|-------|------|----------|
| `primary` | `#2563EB` | `#60A5FA` | Primary actions, active nav icon, links |
| `on-primary` | `#FFFFFF` | `#0B1220` | Text on primary |
| `primary-container` | `#DBEAFE` | `#1E3A8A` | Active nav indicator, selected chips |
| `secondary` | `#4F46E5` | `#818CF8` | Secondary actions, chart series |
| `tertiary` | `#0D9488` | `#2DD4BF` | Billable/success, timer accent |
| `error` | `#DC2626` | `#F87171` | Destructive actions, validation |
| `surface` | `#F8FAFC` | `#0F1420` | Window canvas |
| `surface-container-low` | `#FFFFFF` | `#121A28` | Cards |
| `surface-container` | `#F1F5F9` | `#182030` | Filter bars, table header |
| `surface-container-high` | `#E2E8F0` | `#1E293B` | Dialogs, menus |
| `surface-container-highest` | `#CBD5E1` | `#273449` | Hover/selected row |
| `on-surface` | `#0F172A` | `#E2E8F0` | Primary text |
| `on-surface-variant` | `#475569` | `#94A3B8` | Secondary text, labels |
| `outline` | `#94A3B8` | `#64748B` | Strong borders, icons |
| `outline-variant` | `#E2E8F0` | `#334155` | Card/table dividers |

Accent presets for “Material You” flavour: **Blue**, **Teal**, **Violet**, **Green**,
**Amber**. Generate containers/on-colours from the seed (Material Color Utilities) or ship a
small static palette per preset.

### 4.3 Type scale

Use **Inter** or **Roboto** (both OFL/Apache-2.0) bundled with the binary; fall back to
`Segoe UI`. Use **tabular figures** for every duration, clock, rate, and amount.

| Role | Size / line | Weight | Use |
|------|-------------|--------|-----|
| Display medium | 45 / 52 | 400 | Hero timer in focus mode (optional) |
| Headline large | 32 / 40 | 400 | Page title on wide screens |
| Headline small | 24 / 32 | 400 | Dialog title, page title on compact |
| Title large | 22 / 28 | 400 | Card titles, KPI value |
| Title medium | 16 / 24 | 500 | Section titles, list primary text |
| Body large | 16 / 24 | 400 | Form values, reading text |
| Body medium | 14 / 20 | 400 | Table cells, secondary copy |
| Label large | 14 / 20 | 500 | Buttons, nav labels |
| Label medium | 12 / 16 | 500 | Chips, captions, overlines |
| Label small | 12 / 16 | 500 | Dense table meta, keyboard hints |

### 4.4 Shape, spacing, and sizing

- **Spacing grid:** 4 dp base; 8 dp inline, 16 dp card padding, 24 dp section gap,
  32 dp page margin, 48 dp major section separation.
- **Radii:** 4 dp inputs/badges · 8 dp chips/small controls · 12 dp cards · 16 dp sheets/FAB ·
  28 dp dialogs · full pill for buttons and nav indicators.
- **Density:** Comfortable (default) and Compact (for 1080p laptop users).

| Component | Size |
|-----------|------|
| Window (default) | **1440 × 900** logical px |
| Window (minimum) | **1024 × 700** |
| Mini timer | **380 × 120**, always-on-top |
| Top app bar | 64 dp |
| Nav rail collapsed | 96 dp (narrow 80) |
| Nav rail expanded | 280 dp (M3 range 220–360) |
| Right insights rail | 320 dp |
| Content max width | 1440 dp (centered) |
| Card | 12 dp radius, 16 dp padding |
| Button | 40 dp high, 20 dp icon, 16 dp horizontal padding |
| Text field | 56 dp high, 16 dp horizontal padding |
| Table header / row | 40 dp / 52 dp |
| Icon | 20 dp inline, 24 dp navigation |
| Touch/click target | 48 × 48 dp minimum |

### 4.5 Motion and state

- **Standard easing** for size/position; **emphasized decelerate** for dialogs.
- Durations: 100 ms hover/press · 200 ms dialog/panel · 300 ms page/rail transition.
- Dialog: scrim fades 150 ms, surface scales 0.96 → 1.0 + fades 200 ms.
- Running timer: a 2 s **breathing dot**, not a flash. Reduced-motion setting disables it.
- State layers: hover 8 %, focus 10 %, pressed 10 %, drag 16 %.
- Never animate the timer digits; update once per second and keep them tabular.

---

## 5. Adaptive layout and window sizes

### 5.1 Breakpoints

| Width | Class | Navigation | Body | Right rail |
|-------|-------|------------|------|------------|
| `< 840` | Compact/medium | Modal navigation drawer (hamburger) | Single pane, 16 dp margins | Hidden; opens as sheet |
| `840–1199` | Expanded | Collapsed rail 96 dp | Single pane, max 960 dp | Hidden |
| `1200–1599` | Large | Expanded rail 280 dp | Main pane, max 1120 dp | Hidden; “Details” opens as side sheet |
| `1600+` | Extra-large | Expanded rail 280 dp | Main pane 900–1000 dp | **Visible 320 dp insights rail** |

Desktop-first: the default is 1440 × 900, so most users see the large layout. The app should
remember window size/position and never open wider than the work area.

### 5.2 Grid and anti-empty rules

- Use a **12-column grid**: 32 dp outer margin, 24 dp gutters.
- KPI cards span 3 columns each; main chart spans 8, side card spans 4.
- When the window exceeds 1440 dp, **centre the content** and reveal the right rail instead
  of stretching cards. When it exceeds 1920 dp, cap the app content at 1440 dp and let the
  right rail grow to 360 dp; the outer background remains `surface`.
- A page must never have a single column wider than ~900 dp of reading content. Tables and
  charts may span wider, but text blocks stay within 75 characters.

### 5.3 App shell — 1440 × 900 (large)

```text
┌──────────────────────────────────────────────────────────────────────────────────────────────┐
│ L1 NAV RAIL 280            │ L3 TOP APP BAR 64                                                 │
│ ┌──────┐                   ├───────────────────────────────────────────────────────────────────┤
│ │ ⏱    │ Timesheet         │ Dashboard · Tuesday, Nov 17            [⏱ 01:23:45]  [+ New entry]│
│ │      │ Alex Smith        │                                                                   │
│ └──────┘                   ├───────────────────────────────────────────────────────────────────┤
│                            │                                                   [Search Ctrl+K] │
│  ⌂ Dashboard               │  ┌─────────────────────────────────────────────┐  ┌─────────────┐ │
│  ▤ Entries          128    │  │ L2 TIMER HERO                              │  │ L2 TODAY    │ │
│  ◫ Projects           6    │  │  ● Running · Acme · Website redesign       │  │  5:12 / 8:00 │ │
│  ▦ Reports                 │  │  01:23:45          [Pause] [Stop & save]   │  │  ██████░░░  │ │
│  ⚙ Settings                │  │  Started 09:12 · Billable · $104.25 est.   │  │             │ │
│                            │  └─────────────────────────────────────────────┘  │  WEEK       │ │
│  ─────────────────────     │  ┌────────────┐ ┌────────────┐ ┌────────────┐     │  Mon ▁▃▅█   │ │
│  ▶ Start timer             │  │ TODAY      │ │ THIS WEEK  │ │ BILLABLE   │     │  Tue ▂▄▆█   │ │
│                            │  │ 5:12       │ │ 26:40      │ │ 78%        │     │  Wed ▁▂▄▅   │ │
│  ─────────────────────     │  └────────────┘ └────────────┘ └────────────┘     │  Thu ▃▅▇█   │ │
│  [Mini] [Tray] [⌨]         │  ┌─────────────────────────────────────────────┐  │  Fri ▂▃▅▆   │ │
│  v0.14.0 · ☀ Light         │  │ TODAY'S TIMELINE                            │  │             │ │
│                            │  │ 09:12 ━━ Acme · Website redesign   1:45    │  │  RECENT     │ │
│                            │  │ 11:00 ━━ Internal · Standup        0:30    │  │  Acme       │ │
│                            │  │ 13:30 ━━ Acme · API integration    2:57    │  │  Internal   │ │
│                            │  └─────────────────────────────────────────────┘  └─────────────┘ │
│                            │  ┌─────────────────────────────────────────────┐                  │
│                            │  │ WEEKLY OVERVIEW           [bars + legend]   │                  │
│                            │  └─────────────────────────────────────────────┘                  │
└────────────────────────────┴───────────────────────────────────────────────────────────────────┘
```

### 5.4 App shell — 1920 × 1080 (extra-large, three panes)

```text
┌──────────────────────────────────────────────────────────────────────────────────────────────────────┐
│ NAV 280 │                        MAIN 1000                         │        INSIGHTS 320          │
│         │  Top app bar 64                                        │                              │
│  Timer  │  ┌──────────────────────────────────────────────────┐  │  ┌────────────────────────┐  │
│  chip   │  │  Page title + primary action                     │  │  │ TODAY AT A GLANCE      │  │
│         │  └──────────────────────────────────────────────────┘  │  │ Goal ring · 5:12/8:00  │  │
│  nav    │  ┌──────────────────────┐ ┌──────────────────────┐    │  │ Billable · $104        │  │
│  items  │  │ KPI                  │ │ KPI                  │    │  └────────────────────────┘  │
│         │  └──────────────────────┘ └──────────────────────┘    │  ┌────────────────────────┐  │
│         │  ┌──────────────────────────────────────────────────┐  │  │ TIMER HISTORY          │  │
│         │  │ Main chart / timeline / table                    │  │  │ 09:12 Acme      1:45   │  │
│         │  │ (uses the extra height, not extra empty margin)  │  │  │ 11:00 Internal  0:30   │  │
│         │  └──────────────────────────────────────────────────┘  │  └────────────────────────┘  │
│         │                                                        │  ┌────────────────────────┐  │
│         │                                                        │  │ QUICK ACTIONS          │  │
│         │                                                        │  │ + New entry · Resume   │  │
│         │                                                        │  └────────────────────────┘  │
└─────────┴────────────────────────────────────────────────────────┴──────────────────────────────┘
```

The right rail is **content**, not decoration: goal ring, billable value, recent sessions,
recent projects, and contextual tips. On Entries it becomes a **details/editor pane**; on
Reports it becomes **report options**.

---

## 6. Screen wireframes

### 6.1 Dashboard

**Purpose:** answer “What am I doing, how much have I logged, and what should I do next?”
in one glance.

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Dashboard · Tue, Nov 17                         [Search Ctrl+K]  [+ New entry]│
├──────────────────────────────────────────────────────────────────────────────┤
│ ┌──────────────────────────────────────────────────────────────────────────┐ │
│ │ L2 TIMER HERO — idle state                                               │ │
│ │  What are you working on?                                                │ │
│ │  [ e.g. Design mockup @Acme #ui 45m                            ] [Start]  │ │
│ │  Recent: Acme · Internal · Admin        or  [Start empty timer]           │ │
│ └──────────────────────────────────────────────────────────────────────────┘ │
│ ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐                  │
│ │ TODAY      │ │ THIS WEEK  │ │ BILLABLE   │ │ EST. VALUE │                  │
│ │ 5:12       │ │ 26:40      │ │ 78%        │ │ $1,240.00  │                  │
│ │ ██████░░░  │ │ +12% vs LW │ │ 20:48      │ │ this month │                  │
│ └────────────┘ └────────────┘ └────────────┘ └────────────┘                  │
│ ┌────────────────────────────────────────────┐ ┌───────────────────────────┐ │
│ │ TODAY'S TIMELINE                           │ │ WEEKLY OVERVIEW           │ │
│ │ 09:12  Acme · Website redesign      1:45   │ │ ▁▃▅█▆▅▂  Mon–Sun          │ │
│ │ 11:00  Internal · Standup           0:30   │ │ Billable vs non-billable  │ │
│ │ 13:30  Acme · API integration       2:57   │ │ [View reports →]          │ │
│ │ [+ Add missing time]                       │ │                           │ │
│ └────────────────────────────────────────────┘ └───────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────────────┘
```

**Rules**

- The timer hero is always the first card. In idle state it contains the omnibox and a
  prominent **Start** button; in running state it becomes the live timer with **Pause**,
  **Stop & save**, **+5 m**, and **More**.
- “+ New entry” opens the entry dialog; the omnibox remains the fastest path.
- KPI cards are clickable and deep-link to Reports with the relevant period.
- Empty timeline uses an educational empty state with one action: **Start your first timer**.
- At 1600+, the right rail shows the goal ring, recent projects, and recent sessions.

### 6.2 Entries

**Purpose:** review, filter, edit, and bulk-manage time.

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Entries · 128 records                                      [+ New entry]     │
├──────────────────────────────────────────────────────────────────────────────┤
│ L3 FILTER BAR                                                                │
│ [This month ▾] [All projects ▾] [Billable: All ▾] [Tags ▾] [Search…] [Reset]│
│ Active: [Acme ×] [Billable ×]                                                │
├──────────────────────────────────────────────────────────────────────────────┤
│ ┌──────────────────────────────────────────────────────────┐ ┌─────────────┐ │
│ │ ☐  DATE      PROJECT        TIME        DUR    BILLABLE │ │ DETAILS     │ │
│ │ ☐  Nov 17    ● Acme         09:12–10:57 1:45   [$]      │ │ Entry #482  │ │
│ │ ☐  Nov 17    ● Internal     11:00–11:30 0:30   [ ]      │ │ [Edit]      │ │
│ │ ☐  Nov 16    ● Acme         13:30–16:27 2:57   [$]      │ │ Project     │ │
│ │ …                                                        │ │ Acme        │ │
│ │                                                          │ │ Duration    │ │
│ │ Sticky header · 52 dp rows · tabular numerals · hover    │ │ 1:45        │ │
│ └──────────────────────────────────────────────────────────┘ │ Tags        │ │
│                                                              │ #design     │ │
│                                                              │ [Duplicate] │ │
│                                                              │ [Delete]    │ │
│                                                              └─────────────┘ │
│ L5 BULK BAR (when rows selected)                                             │
│ 3 selected   [Set billable] [Add tags] [Duplicate] [Delete]        [Clear]   │
└──────────────────────────────────────────────────────────────────────────────┘
```

**Rules**

- Clicking a row selects it and fills the details pane; double-click (or **Edit**) opens the
  **Entry dialog**. Inline editing is reserved for duration only.
- Table uses 1 dp `outline-variant` dividers; no zebra striping. Hover/selection uses
  `surface-container-highest` at 40–60 %.
- Bulk actions appear in a floating L5 bar, never as permanent buttons that clutter the
  header.
- Filters persist per session; “Reset” clears all. Show active filters as removable chips.
- At `< 1200`, hide the details pane and open it as a side sheet or dialog.
- Keyboard: `Ctrl+N` new, `Enter` open selected, `Delete` bulk-delete with confirm, `/` focus
  search, `Esc` clear selection.

### 6.3 Projects

**Purpose:** manage clients, rates, budgets, and project health.

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Projects · 6 active                                    [Grid|List] [+ Project]│
├──────────────────────────────────────────────────────────────────────────────┤
│ ┌─────────────────────┐ ┌─────────────────────┐ ┌─────────────────────┐      │
│ │ ● Acme              │ │ ● Internal          │ │ ● Northwind         │      │
│ │ Website redesign    │ │ Operations          │ │ Audit               │      │
│ │ $120/h · Billable   │ │ Non-billable        │ │ $95/h · Billable    │      │
│ │ Budget ███████░ 72% │ │ Budget —            │ │ Budget ███░░░░ 31%  │      │
│ │ 42:10 this month    │ │ 8:25 this month     │ │ 12:40 this month    │      │
│ │                    ⋮│ │                    ⋮│ │                    ⋮│      │
│ └─────────────────────┘ └─────────────────────┘ └─────────────────────┘      │
│ ┌─────────────────────┐ ┌─────────────────────┐                              │
│ │ + New project       │ │ Archived (2)  →     │                              │
│ └─────────────────────┘ └─────────────────────┘                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

**Rules**

- Default to a responsive card grid (min 320 dp); at 1600+ show 4 columns.
- Card actions live in a `⋮` menu: Edit, Duplicate, Archive, Delete. **Archive** is the
  primary “remove” action; Delete is destructive and confirmed.
- The Project dialog is a 480 dp basic dialog.
- Show budget progress with a tonal bar plus text (`72 % · $8,640 of $12,000`), not colour
  alone.

### 6.4 Reports & Export

**Purpose:** answer “Where did the time go, what is it worth, and how do I share it?”

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Reports & Export                                      [Export…]  [Print]     │
├──────────────────────────────────────────────────────────────────────────────┤
│ L3 PERIOD BAR                                                                │
│ [This week] [Last week] [This month] [Last month] [Custom: 11/01 – 11/17]   │
│ [All projects ▾] [Billable: All ▾] [Group by project ☑] [Include summary ☑] │
├──────────────────────────────────────────────────────────────────────────────┤
│ ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐                  │
│ │ TOTAL      │ │ BILLABLE   │ │ NON-BILL   │ │ EST. VALUE │                  │
│ │ 26:40      │ │ 20:48      │ │ 5:52       │ │ $2,496.00  │                  │
│ └────────────┘ └────────────┘ └────────────┘ └────────────┘                  │
│ ┌────────────────────────────────────────────┐ ┌───────────────────────────┐ │
│ │ HOURS BY DAY                               │ │ BY PROJECT                │ │
│ │  ▁▃▅█▆▅▂                                  │ │ Acme       18:20   $2,200 │ │
│ │  Mon Tue Wed Thu Fri Sat Sun               │ │ Internal    5:10   —       │ │
│ │                                            │ │ Northwind   3:10   $296   │ │
│ └────────────────────────────────────────────┘ └───────────────────────────┘ │
│ ┌──────────────────────────────────────────────────────────────────────────┐ │
│ │ ENTRY PREVIEW (first 8 rows)                               [View all →]  │ │
│ └──────────────────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────────────┘
```

**Rules**

- **Export…** opens the Export dialog (format cards + options + destination), never a raw
  file picker as the first step.
- Report presets are chips; custom range uses the Material date range picker.
- The preview table is read-only; clicking a row opens the entry dialog.
- At 1600+, the right rail shows report options and a “what’s included” summary.

### 6.5 Settings

**Purpose:** make preferences discoverable without a wall of switches.

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Settings                                                                     │
├────────────────┬─────────────────────────────────────────────────────────────┤
│ General        │ APPEARANCE                                                  │
│ Appearance  ●  │ Theme mode     ( ) System   (•) Light   ( ) Dark            │
│ Timer & Tray   │ Theme style    ( ) Classic  (•) Material 3 (full)           │
│ Data & Backup  │ Accent         [● Blue] [Teal] [Violet] [Green] [Amber]     │
│ Shortcuts      │ Density        (•) Comfortable   ( ) Compact                │
│ About          │                                                             │
│                │ ┌─────────────────────────────────────────────────────────┐ │
│                │ │ LIVE PREVIEW                                            │ │
│                │ │ ┌─────────────────────────────────────────────────────┐ │ │
│                │ │ │ Card · primary button · input · chip               │ │ │
│                │ │ └─────────────────────────────────────────────────────┘ │ │
│                │ └─────────────────────────────────────────────────────────┘ │
│                │ [Reset to defaults]                         [Save changes]  │
└────────────────┴─────────────────────────────────────────────────────────────┘
```

**Rules**

- Use a secondary nav inside the content area (not the main rail) with clear sections.
- Save is immediate for display-only preferences (theme preview), but profile/data changes
  use an explicit **Save changes** to avoid surprise writes.
- **Theme style: Material 3 (full)** is the feature that applies the complete M3 token set
  to every view (see §10).
- **Timer & Tray** contains all desktop behaviour: close-to-tray, start minimized, tray icon,
  notifications, idle detection, and pomodoro defaults.

### 6.6 Mini timer (always-on-top)

```text
┌──────────────────────────────────────────┐
│ ● Acme · Website redesign          [↗]  │
│ 01:23:45                      [⏸] [⏹]  │
│ Billable · started 09:12                 │
└──────────────────────────────────────────┘
```

- 380 × 120 default, resizable height only.
- Drag anywhere on the surface; `↗` restores the main window at the Dashboard.
- `⏸` toggles pause; `⏹` opens the review/save dialog (or saves directly if “Quick stop” is
  enabled).
- Always uses the timer accent and tabular numerals; supports reduced motion.

---

## 7. Dialogs and overlays

All create/edit forms are **basic desktop dialogs** (M3: 280–560 dp, 28 dp radius, 24 dp
padding). They appear above a scrim and trap focus. `Esc` cancels; `Ctrl+Enter` or the
primary button saves. On windows narrower than 600 dp, use a full-screen dialog.

### 7.1 Entry dialog — 560 dp

```text
        ┌──────────────────────────────────────────────────────┐
        │  New time entry                                  ✕   │ 24 dp
        │  ──────────────────────────────────────────────────  │
        │  Date *              Project                         │
        │  [ Tue, Nov 17 📅 ]  [ ● Acme · Website redesign ▾ ] │ 56 dp fields
        │                                                      │
        │  Start        End          Duration *                │
        │  [ 09:12 ]   [ 10:57 ]     [ 1:45 ]  (auto)          │
        │                                                      │
        │  Description                                         │
        │  [ Website redesign — hero section                 ] │
        │                                                      │
        │  Tags                                                │
        │  [ #design × ] [ #client × ]  [ Add tag… ]           │
        │                                                      │
        │  ☑ Billable                    Rate: $120/h          │
        │  ──────────────────────────────────────────────────  │
        │                         [Cancel]   [Save entry]      │ 24 dp gap
        └──────────────────────────────────────────────────────┘
```

**Behaviour**

- Duration auto-calculates from start/end; editing duration updates end time or clears it.
- Project picker puts **Recent** first, then searchable active projects.
- Validation appears inline under the field, not in a toast.
- On save: close, refresh the current view, show a snackbar with **Undo**.

### 7.2 Project dialog — 480 dp

```text
┌──────────────────────────────────────────────┐
│  Edit project                            ✕   │
│  ──────────────────────────────────────────  │
│  Name *        [ Acme                     ]  │
│  Client        [ Acme Corp                ]  │
│  Hourly rate   [ 120.00        ] USD         │
│  Colour        [ ● ][ ● ][ ● ][ ● ][ ● ]     │
│  Budget hours  [ 120           ] (optional)  │
│  ☑ Active      ☐ Billable by default         │
│  ──────────────────────────────────────────  │
│                    [Cancel]  [Save project]  │
└──────────────────────────────────────────────┘
```

### 7.3 Destructive confirmation — 400 dp

```text
┌──────────────────────────────────────────────┐
│  Delete 3 entries?                       ✕   │
│  This cannot be undone.                      │
│  Tip: archive projects instead of deleting.  │
│  ──────────────────────────────────────────  │
│                    [Cancel]  [Delete]        │  Delete = error colour
└──────────────────────────────────────────────┘
```

**Rules**

- Destructive primary action uses `error` and is **not** the default focus.
- The default focus is **Cancel**.
- For bulk delete, show the exact count and a preview link.

### 7.4 Export dialog — 560 dp

```text
┌──────────────────────────────────────────────────────────┐
│  Export timesheet                                    ✕   │
│  ────────────────────────────────────────────────────    │
│  Format                                                  │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐     │
│  │ CSV   ●  │ │ Excel    │ │ PDF      │ │ Markdown │     │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘     │
│                                                          │
│  Period        [ This month ▾ ]                          │
│  Include       ☑ Summary  ☑ Metadata  ☑ Project groups   │
│  Destination   [ C:\Users\Alex\Documents\Timesheets ] …  │
│  File name     [ timesheet-2026-11-17 ]                  │
│  ────────────────────────────────────────────────────    │
│  Preview: 128 entries · 26:40 · 4 columns                │
│                              [Cancel]  [Export]          │
└──────────────────────────────────────────────────────────┘
```

- Format cards are selectable tiles (not a dropdown).
- The OS file picker only appears after pressing **Export** (or via “Choose…” for the folder).
- Remember the last format and destination.

### 7.5 Command palette — 640 dp

```text
┌──────────────────────────────────────────────────────────┐
│  🔍  Type a command or search…                     Esc   │
│  ────────────────────────────────────────────────────    │
│  RECENT                                                  │
│  ▶ Start timer · Acme · Website redesign                 │
│  + New entry                                             │
│  ⚙ Open Settings                                         │
│  ────────────────────────────────────────────────────    │
│  NAVIGATE                                                │
│  ⌂ Dashboard        ▤ Entries        ◫ Projects          │
│  ▦ Reports          ⚙ Settings                           │
└──────────────────────────────────────────────────────────┘
```

- `Ctrl+K` opens; type to filter actions and recent projects; `Enter` runs; `Esc` closes.
- This is the keyboard-first counterpart to the omnibox.

### 7.6 Idle / long-session prompt

```text
┌──────────────────────────────────────────────┐
│  Still working?                          ✕   │
│  The timer has been running for 47 min       │
│  without keyboard or mouse activity.         │
│                                              │
│  [ Keep running ]  [ Stop at 10:12 ]  [Stop & save] │
└──────────────────────────────────────────────┘
```

- Trigger after 15 min idle (configurable) or 3 h / 8 h long-running thresholds.
- Keep it non-blocking and dismissible; never auto-discard without confirmation.

---

## 8. Timer UX specification

### 8.1 Model

```text
 Idle ──Start──▶ Running ──Pause──▶ Paused ──Resume──▶ Running
                   │                  │
                   └──Stop────────────┴──▶ Review ──Save──▶ Idle
                                             │
                                             └──Discard──▶ Idle

 Optional: Running ──Pomodoro target──▶ Focus complete ──▶ Break ──▶ Running
```

Persist the **start instant** and accumulated paused duration, not a ticking counter.
On restart, reconstruct elapsed time from the clock. This is already the app’s model and
must remain so.

### 8.2 Surfaces

| Surface | Content | Behaviour |
|---------|---------|-----------|
| Top app bar chip | status dot, `HH:MM` (hover `HH:MM:SS`), project | click opens timer popover; always visible while running |
| Dashboard hero | `HH:MM:SS`, project, description, billable, earnings | primary controls: Pause/Resume, Stop & save, +5 m, More |
| Sidebar/rail | compact timer card | quick Pause/Stop; hidden when rail is collapsed to icons |
| Mini timer | always-on-top 380 × 120 | drag, pause, stop, restore |
| Tray icon + tooltip | icon state + “Running 01:23:45 · Acme” | tooltip updates every 10 s, not every tick |
| Tray menu | Pause/Resume, Stop & save | dynamic label based on state |

### 8.3 Display rules

- **Idle:** `Start timer` is the largest button on the Dashboard; recent projects are chips.
- **Running:** show `● Running` text + accent; use tabular numerals. Do not pulse the digits.
- **Paused:** show `⏸ Paused` text + amber dot; the elapsed value is frozen.
- **Pomodoro:** show `Focus 18:42` countdown and a progress ring; break state uses a
  different icon and `Break 04:12`.
- **Long session:** at 3 h show an amber inline banner with **Review**; at 8 h show a red
  banner and a confirmation before saving.
- **Earnings:** show `≈ $104.25` only when a project rate exists; label it “estimated”.

### 8.4 Start/stop flow

1. **Start** from omnibox, recent project chip, timer button, tray, or `Ctrl+Space`.
2. Project and description are optional; the timer starts immediately.
3. **Stop & save** opens the Entry dialog prefilled with start/end/duration, project,
   description, billable, and tags. Primary action: **Save entry**. Secondary: **Discard**,
   **Keep running**.
4. If the user enabled **Quick stop**, save directly and show an undo snackbar.
5. **One-tap resume** from an Entry starts a new timer with the same project/description.

### 8.5 Notifications

- Pomodoro focus complete: one Windows notification with **Start break** / **Dismiss**.
- Idle prompt: in-app dialog first; optionally a notification if the window is hidden.
- Never notify for every start/stop; the tray icon and chip are enough.
- Respect Windows Focus Assist/quiet time where possible.

---

## 9. System tray (Windows notification area) specification

### 9.1 Behaviour

| Event | Action |
|-------|--------|
| Left click | Show/focus the main window (or the mini timer if that was the last surface). |
| Double click | Open the Dashboard and focus the window. |
| Right click | Show the native context menu near the icon. |
| Close button | Hide to notification area if **Close to tray** is enabled; otherwise quit with confirmation. |
| Second launch | Focus the existing instance (single-instance mutex), do not open a second database. |
| Start with Windows | Optional, off by default; uses the current user’s Startup folder or Run key. |

### 9.2 Menu wireframe

```text
┌──────────────────────────────────────────────┐
│ Open Timesheet                         ⏎     │  ← default, bold
│ ──────────────────────────────────────────── │
│ ⏸ Pause timer                 01:23:45       │  ← dynamic
│ ⏹ Stop & save                                │
│ ──────────────────────────────────────────── │
│ Today: 5:12 · Billable 4:01                  │  ← disabled summary line
│ ──────────────────────────────────────────── │
│ ✓ Start with Windows                         │
│ ✓ Minimize to notification area              │
│ Notifications                         ▸      │  ← submenu: Pomodoro, Idle, Off
│ Settings…                                    │
│ ──────────────────────────────────────────── │
│ Exit                                         │
└──────────────────────────────────────────────┘
```

**Rules**

- Order follows Microsoft’s guidance: Open, secondary commands, separator, opt-in check
  items, Options, Exit.
- Use **Pause/Resume** and **Stop & save** dynamically; hide them when no timer is running.
- “Minimize to notification area” is a check item; turning it off removes the icon but does
  not stop tracking.
- Keep the tooltip under 127 characters: `Timesheet — Running 01:23:45 · Acme`.
- Do not flash the icon; use a notification only for pomodoro completion or idle.

### 9.3 Tray icon states

| State | Icon | Tooltip |
|-------|------|---------|
| Idle | teal clock | `Timesheet — Idle · Today 5:12` |
| Running | blue clock + dot | `Timesheet — Running 01:23:45 · Acme` |
| Paused | amber clock | `Timesheet — Paused 01:23:45 · Acme` |
| Pomodoro | tomato/clock | `Timesheet — Focus 18:42 · Acme` |
| Error | red badge | `Timesheet — Needs attention` |

Implement icon variants by drawing the 32 × 32 RGBA icon with a state colour; no external
assets required.

### 9.4 Implementation notes (Rust / egui)

- Keep `tray-icon` (already used) and forward `TrayIconEvent`/`MenuEvent` into the egui event
  loop; the current polling loop is acceptable but should be event-driven where possible.
- Add a `TrayMenuState` that is rebuilt only when timer state changes (label text, enabled
  state), not every frame.
- Add a single-instance guard (named mutex via `windows-sys`, or a small crate) before
  opening the database.
- Add `autostart` support via the per-user Startup folder (`%APPDATA%\Microsoft\Windows\
  Start Menu\Programs\Startup\timesheet.lnk`) or `HKCU\Software\Microsoft\Windows\
  CurrentVersion\Run`; make it explicit in Settings.
- Mini timer: use a second viewport or a compact mode with `WindowLevel::AlwaysOnTop`; persist
  its position.

---

## 10. Full Material 3 theme option

### 10.1 Settings model

```rust
pub enum ThemeStyle { Classic, Material3 }
pub enum ThemeMode  { System, Light, Dark }
pub enum Density    { Comfortable, Compact }

pub struct AppearanceSettings {
    pub style: ThemeStyle,
    pub mode: ThemeMode,
    pub accent: Accent,      // Blue, Teal, Violet, Green, Amber
    pub density: Density,
    pub reduced_motion: bool,
}
```

- **Classic** preserves the current slate/blue look.
- **Material 3 (full)** applies M3 tokens to every view and component, including tables,
  dialogs, nav rail, chips, snackbars, and the mini timer.
- **System** follows the Windows app theme; on Windows 10/11 read the app theme preference
  (registry `AppsUseLightTheme`) and react to changes.
- **Accent** generates `primary`, `primary-container`, `secondary`, `tertiary` roles; for a
  true “Material You” feel, optionally seed from the Windows accent colour.

### 10.2 Token mapping (egui)

| egui / app element | Material token |
|--------------------|----------------|
| `panel_fill` | `surface` |
| card frame | `surface-container-low` + `outline-variant` 1 dp + 12 dp radius |
| `window_fill` | `surface-container-high` |
| `extreme_bg_color` | `surface-container-highest` |
| `widgets.inactive.bg_fill` | `surface-container-high` |
| `widgets.hovered.bg_fill` | `surface-container-highest` |
| `widgets.active.bg_fill` | `primary-container` |
| `selection.bg_fill` | `primary-container` |
| `selection.stroke` | `primary` |
| text | `on-surface` |
| weak text | `on-surface-variant` |
| error | `error` |
| success/billable | `tertiary` |
| nav active indicator | `secondary-container`, 56 × 32 dp pill |
| buttons | filled / tonal / outlined / text, 40 dp height |
| inputs | 56 dp, 4 dp top radius (filled) or 1 dp outline (outlined) |
| dialogs | 28 dp radius, 24 dp padding, level 3 |
| snackbar | `inverse-surface` / `inverse-on-surface` |

Also update `src/ui/material.rs` so the `egui_material3` global theme receives the same
palette. The current code already pushes a blue/slate palette; extend it with accent presets
and `ThemeMode::System`.

### 10.3 Implementation plan

1. Add `AppearanceSettings` to `Settings` with serde defaults (`Classic`, `Dark`, `Blue`,
   `Comfortable`, `false`).
2. Build `src/ui/theme.rs` with `MaterialTheme::from_settings()` returning a token struct
   (colours, spacing, radius, type sizes).
3. `TimesheetApp::apply_theme` maps tokens to `egui::Visuals`, `Style::spacing`, text styles,
   and the `egui_material3` global theme.
4. Replace hard-coded colours in `ui/mod.rs`, `app.rs`, and view files with token lookups.
5. Add the Appearance settings section with a live preview card and a “Material 3 (full)”
   switch.
6. Add visual regression screenshots for light/dark and Classic/Material 3.

---

## 11. Accessibility and keyboard

- Contrast: body text ≥ 4.5:1, large text ≥ 3:1, icons ≥ 3:1.
- Do not encode state with colour alone: use `● Running`, `⏸ Paused`, `$ Billable`, `✓`.
- All interactive targets ≥ 48 × 48 dp even if the visual is smaller.
- Focus ring: 2 dp `primary` outline with 2 dp offset; never remove focus.
- Keyboard:
  - `Ctrl+K` command palette · `Ctrl+N` new entry · `Ctrl+Space` start/pause
  - `Ctrl+Shift+Space` stop & save · `Ctrl+M` mini timer · `Ctrl+1..5` navigate
  - `Esc` cancel/close · `Enter` primary action · `Delete` delete selection
- Screen readers: label icon-only buttons; announce timer state changes politely, not every
  second.
- Reduced motion: disable breathing dots, scale transitions, and chart animations.
- Text scaling: support 100–200 % without clipping; dialogs grow vertically and scroll.

---

## 12. Implementation roadmap

| Phase | Work | Files |
|-------|------|-------|
| **A. Tokens & shell** | Add theme tokens, rail/app bar, content max width, 3-pane shell | `src/ui/theme.rs`, `src/ui/app.rs`, `src/ui/mod.rs` |
| **B. Layered components** | Card, section header, filter bar, table shell, bulk bar, snackbar, dialog primitive | `src/ui/components/*` |
| **C. Dialogs** | Entry, project, export, confirm, idle, command palette | `src/ui/dialogs/*`, view files |
| **D. Timer** | Hero card, review-on-stop, top-bar chip, timer popover, mini timer polish | `src/ui/dashboard.rs`, `src/ui/app.rs` |
| **E. Tray** | Menu state model, dynamic labels, autostart, single instance, tooltip throttle | `src/tray.rs`, `src/ui/app.rs`, `src/settings.rs` |
| **F. Full theme** | Appearance settings, accent presets, system theme, visual regression | `src/settings.rs`, `src/ui/settings_view.rs`, `src/ui/material.rs` |
| **G. Reports/export** | Export dialog, format tiles, options, preview | `src/ui/reports.rs`, `src/export/*` |

Keep the app shippable after each phase; do not break the Classic theme while the Material
theme is being built.

---

## 13. Acceptance checklist

- [ ] Default window opens at 1440 × 900; minimum 1024 × 700; size/position persisted.
- [ ] At 1600+ the right insights rail is visible and useful; no card stretches beyond 1440 dp.
- [ ] Every create/edit form opens in a dialog with 24 dp padding, 28 dp radius, and focus trap.
- [ ] Timer is visible in the app bar, dashboard, mini timer, and tray; state is readable
      without colour.
- [ ] Stop opens a review dialog; long-session and idle prompts are actionable.
- [ ] Tray menu follows Windows order; left click opens the window; right click shows the menu;
      close-to-tray is configurable.
- [ ] Theme style can be switched between Classic and full Material 3; all views update.
- [ ] Light/dark/system and accent presets work; contrast passes WCAG AA.
- [ ] Keyboard shortcuts work; focus rings are visible; reduced motion is respected.
- [ ] `cargo fmt`, `cargo check`, and `cargo test` pass.

---

## 14. References

**Material Design 3**

- Breakpoints — https://m3.material.io/foundations/layout/breakpoints
- Applying layout (large & extra-large) — https://m3.material.io/foundations/layout/applying-layout/large-extra-large
- Navigation rail — https://m3.material.io/components/navigation-rail/specs
- Dialogs — https://m3.material.io/components/dialogs/specs
- Elevation — https://m3.material.io/styles/elevation
- Buttons — https://m3.material.io/components/buttons/specs
- Text fields — https://m3.material.io/components/text-fields/specs
- Typography — https://m3.material.io/styles/typography/type-scale-tokens

**Timer / time tracking UX**

- Koder.ai — How to build a time-tracking app — https://koder.ai/blog/build-mobile-app-time-tracking-productivity
- Ignite Solutions — UX design of a time-tracking app — https://www.ignitesol.com/design-thinking-process-and-the-ux-design-of-our-time-tracking-app/
- Eleken — Time tracking app design — https://www.eleken.co/blog-posts/time-tracking-app-design-how-to-make-an-app-that-increases-productivity
- Prototypr — Expressing time in UI/UX — https://blog.prototypr.io/expressing-time-in-ui-ux-design-5-rules-and-a-few-other-things-eda5531a41a7

**Windows notification area**

- Microsoft — Notification area guidelines — https://learn.microsoft.com/en-us/windows/win32/uxguide/winenv-notification
- Microsoft — Notifications and the notification area — https://learn.microsoft.com/en-us/windows/win32/shell/notification-area
- Microsoft — Menus and context menus — https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/menus-and-context-menus
- tray-icon crate — https://crates.io/crates/tray-icon
