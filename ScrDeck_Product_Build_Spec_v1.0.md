# SCRDECK
## Product, UX/UI, Software Requirements & Engineering Build Specification

**Document ID:** SD-PBS-001  
**Version:** 1.0  
**Status:** Build baseline / implementation-ready draft  
**Date:** 18 November 2026  
**Initial platform:** Windows desktop (x64 first)  
**Product category:** Local-first desktop utility / independent GUI companion  
**Working tagline:** *Your Android. Your Workspace.*

---

## Document purpose

This document is the single source of truth for designing, building, testing, packaging, and maintaining ScrDeck. It combines the product brief, product requirements (PRD), software requirements (SRS), information architecture, UX and visual design system, technical architecture, security requirements, release plan, acceptance criteria, and execution rules for a human or AI-assisted engineering team.

The specification is intentionally detailed enough to guide implementation without requiring the builder to infer product behavior, invent UI patterns, or substitute decorative mockups for working functionality. Where an upstream tool's behavior can change between versions, the installed executable and its current official documentation take precedence over this document.

> **Product classification:** ScrDeck is not a SaaS product. It is a local-first Windows desktop application. It does not need a ScrDeck account, cloud backend, subscription, analytics service, or internet connection for its core mirroring workflow. The term “full SaaS-style specification” describes the completeness of this product document, not the deployment model.

## Table of contents

1. Executive summary  
2. Product identity, positioning, and boundaries  
3. Goals, non-goals, personas, and success measures  
4. Research baseline and technical assumptions  
5. Scope and release strategy  
6. Information architecture and navigation  
7. End-to-end user journeys  
8. Functional requirements  
9. Screen-by-screen UX specification  
10. Visual design system  
11. Interaction, content, and accessibility standards  
12. Technical architecture  
13. Data models and configuration formats  
14. Dependency, ADB, driver, and device setup behavior  
15. Process lifecycle, command generation, and error handling  
16. Security, privacy, licensing, and trust  
17. Non-functional requirements  
18. Testing and quality assurance  
19. Packaging, release, and maintenance  
20. Risks, constraints, and decisions  
21. Definition of done and acceptance criteria  
22. Engineering execution contract  
Appendix A. Initial preset catalogue  
Appendix B. Example configuration schema  
Appendix C. Manual smoke-test checklist  
Appendix D. Official references and design inspiration

---

# 1. Executive summary

ScrDeck is an independent, open-source graphical companion for the original **scrcpy** project and Android Debug Bridge (**ADB**). It gives users a clear desktop interface for discovering Android devices, preparing ADB, configuring scrcpy, saving presets, launching mirroring sessions, and understanding connection or process errors.

The core mirroring, video/audio transport, and device-control functionality remain provided by the original scrcpy project developed by Genymobile. ADB communication remains provided by Android SDK Platform-Tools. ScrDeck is the orchestration and user-experience layer: it detects, configures, invokes, observes, and explains these existing tools. It must not imply that ScrDeck owns, replaces, reimplements, or officially represents their core functionality.

### Product promise

**For beginners:** connect an Android phone and start mirroring without learning command-line syntax.  
**For advanced users:** keep access to the real scrcpy arguments, a transparent generated command, custom arguments, and useful diagnostics.  
**For everyone:** know which device is selected, what settings will be applied, what process is running, and what to do when something fails.

### Core product pillars

1. **Guided setup:** identify available tools, guide Android USB debugging, detect ADB authorization, and help users find the correct OEM USB driver only when required.
2. **Device clarity:** show connected devices and meaningful states such as authorized, unauthorized, offline, and unavailable.
3. **Visual configuration:** expose common scrcpy options as understandable controls, with Basic and Advanced modes.
4. **Transparent execution:** generate an inspectable command and launch the official executable as a child process without passing user input through a shell.
5. **Reusable workflows:** save, duplicate, import, and export local presets.
6. **Actionable diagnostics:** translate common failures into precise next steps while preserving raw logs for technical users.

### v0.1 product outcome

A Windows user can install ScrDeck, complete first-run setup, connect one Android device over USB or supported Wireless Debugging, select the device, configure a valid scrcpy session, start and stop mirroring, save a preset, and inspect meaningful logs. The application must be functional end-to-end; it must not ship as a static mockup with non-functional buttons.

---

# 2. Product identity, positioning, and boundaries

## 2.1 Name and identity

- **Product name:** ScrDeck.
- **Tagline:** Your Android. Your Workspace.
- **Descriptor:** Independent GUI companion for scrcpy and ADB.
- **Personality:** precise, calm, capable, transparent, approachable, and technically honest.
- **Visual direction:** Industrial Utility — Refined Desktop.

The name is a working product identity. Before a public launch, maintainers should check repository-name availability, domain availability if needed, relevant trademarks, and possible confusion with existing products. Do not claim trademark ownership or affiliation without legal review.

## 2.2 Positioning statement

ScrDeck makes established Android device workflows easier to access through a well-designed Windows desktop interface. It is a graphical control layer around upstream command-line tools, not a new screen-mirroring engine.

## 2.3 Required product disclaimer

Use the following copy as the canonical baseline in the About page, README, onboarding attribution step, and release notes where relevant:

> **ScrDeck is an independent, open-source GUI companion for scrcpy and ADB.** It provides a graphical interface for configuring, launching, and managing operations performed by these third-party tools. Core screen mirroring, media transport, and device-control functionality are provided by the original scrcpy project developed by Genymobile. ADB is provided by Android SDK Platform-Tools. ScrDeck does not own, replace, reimplement, or claim authorship of these upstream tools or their core functionality. ScrDeck is not affiliated with, endorsed by, or officially supported by Genymobile or Google. scrcpy, Android SDK Platform-Tools, and bundled dependencies remain subject to their respective licenses and notices.

The UI must never describe scrcpy features as “ScrDeck's mirroring engine”. Prefer “Powered by scrcpy”, “Launch scrcpy”, “scrcpy session”, and “ADB connection”.

## 2.4 Product boundaries

| Area | ScrDeck responsibility | Upstream responsibility |
|---|---|---|
| Device discovery | Invoke ADB, parse device state, present device list | ADB client/server and device daemon communication |
| USB authorization | Explain and detect state | Android's USB debugging and RSA authorization mechanisms |
| USB driver assistance | Diagnose likely driver state and link to verified OEM sources | OEM/Windows USB driver package and installation behavior |
| Wireless pairing | Present pairing workflow and invoke ADB pairing commands | ADB wireless debugging protocol and Android settings |
| Screen mirroring | Configure, launch, monitor, and stop process | scrcpy's screen capture, encoding, streaming, rendering, and input control |
| Audio/video options | Display supported options and build arguments | scrcpy's implementation and device/OS compatibility |
| Presets | Store local ScrDeck configuration | No upstream responsibility |
| Logs | Capture process output and translate known errors | Raw diagnostic output generated by ADB/scrcpy |
| ADB utilities | Expose selected commands in a controlled UI | ADB operations and Android system behavior |

## 2.5 Explicit non-goals

ScrDeck v0.1 must not:

- Implement its own Android screen-capture or mirroring protocol.
- Fork or modify scrcpy source as a prerequisite for basic functionality.
- Install an APK or persistent Android-side service for the basic mirroring workflow unless upstream scrcpy itself requires a temporary server component as part of its normal operation.
- Require root access on the Android device.
- Require an account, cloud connection, analytics opt-in, or subscription.
- Claim official partnership, endorsement, or product ownership by Genymobile or Google.
- Automatically install an arbitrary USB driver, disable Windows driver-signature enforcement, or bypass Windows security controls.
- Silently kill a shared ADB server or terminate unrelated ADB/scrcpy processes.
- Treat a custom scrcpy argument field as a general shell terminal.
- Promise that every scrcpy flag works on every Android device or every scrcpy version.

---

# 3. Goals, non-goals, personas, and success measures

## 3.1 Product goals

- Reduce the setup barrier for users who have never used ADB or scrcpy.
- Make the most common scrcpy settings discoverable through visual controls.
- Keep the full power of scrcpy accessible through Advanced Mode and custom arguments.
- Give clear visibility into ADB state, selected device, active session, and errors.
- Preserve upstream compatibility by inspecting the installed tools rather than assuming a fixed feature set.
- Package the Windows application and tool dependencies responsibly.
- Establish a maintainable foundation for future ADB utilities without bloating the initial release.

## 3.2 User personas

### Persona A — First-time Android mirroring user

- Understands how to connect a phone by USB but may not know what ADB, USB Debugging, Developer Options, or OEM drivers mean.
- Needs guided instructions, clear confirmation, and safe defaults.
- Should be able to complete the standard path without opening a terminal.

### Persona B — Everyday power user / Android enthusiast

- Knows how to enable debugging and may already have Platform-Tools installed.
- Wants quick device selection, reusable presets, bitrate/size/FPS controls, audio toggles, and predictable launch behavior.
- Needs to select an existing ADB installation instead of being forced to install another copy.

### Persona C — Advanced user / modder / developer

- Understands serial IDs, USB vs TCP/IP transports, scrcpy flags, ADB versions, and process output.
- Wants custom arguments, command preview, raw stdout/stderr, executable-path control, and the ability to inspect or copy generated commands.
- Must not be blocked by beginner-oriented abstraction.

### Persona D — Support / troubleshooting user

- May not know whether the problem is the cable, USB mode, driver, ADB authorization, daemon, device, or scrcpy process.
- Needs a guided diagnostic tree and a sanitized support bundle.

## 3.3 Success measures

These are product targets to validate in usability testing, not claims about existing performance.

| Measure | Initial target |
|---|---:|
| First-run USB setup completion, using a supported device and working data cable | ≥ 85% in moderated usability tests |
| First successful mirroring session after tools are available | ≤ 5 minutes for a familiar Android user; guided completion for novice users |
| Common errors mapped to a useful explanation and next action | 100% of documented common states |
| Core workflow requires terminal use | 0 required terminal steps |
| Scrcpy command preview matches the actual argv passed to the process | 100% |
| Primary UI controls have loading, success, empty, and error states | 100% |
| Core workflow works offline after dependencies are present | Yes |
| Unrelated ADB processes killed by ScrDeck | 0 |

---

# 4. Research baseline and technical assumptions

## 4.1 Verified upstream baseline at document date

As of **18 November 2026**, the official scrcpy repository and Windows documentation identify **scrcpy v5.0.1** as the current release surfaced by the upstream release page. The official Windows distribution provides architecture-specific archives and warns users to download only from the official Genymobile repository. [R1][R2]

The official Windows scrcpy package includes ADB and related dependencies in its distribution path. Therefore, ScrDeck should prefer a single managed scrcpy Windows bundle for its default setup rather than installing a duplicate ADB package by default. [R2]

The Android SDK Platform-Tools release notes list 37.0.1 as a July 2026 release and 37.0.3 as a November 2026 Canary build. ScrDeck should prefer the latest stable Platform-Tools channel, not Canary, unless an advanced user deliberately chooses a prerelease build. Re-check official release notes at implementation and release time. [R3]

## 4.2 scrcpy capabilities to expose carefully

The upstream project documents USB and TCP/IP connectivity, keyboard and mouse control, audio forwarding, recording, virtual display, screen-off mirroring, bidirectional copy/paste, configurable quality, camera mirroring on supported Android versions, HID-based keyboard/mouse simulation, gamepad support, and OTG mode. Compatibility varies by Android version, device implementation, host platform, and scrcpy release. [R1]

The official project specifies Android API 21 or later as a prerequisite; the upstream README describes audio forwarding as supported from Android 11 / API 30 and camera mirroring from Android 12 / API 31. ScrDeck must display capability hints and must not promise device support solely because a control exists in the app. [R1]

## 4.3 ADB behavior

ADB is a client-server tool with a host-side client, host-side server, and Android-side daemon. The ADB server is shared by other development tools and normally listens on TCP port 5037. ScrDeck must treat that server as shared system infrastructure, not as a process it owns exclusively. [R4]

USB debugging must be enabled in Android Developer Options. On supported Android versions, the user must unlock the phone and approve the RSA debugging authorization prompt before the computer can use ADB. [R4]

Wireless Debugging pairing requires the Android device and workstation to be on the same network. The documented pairing-code flow uses the IP address and pairing port shown by Android, then verifies the resulting connection. Paired devices can remain paired until the user forgets the workstation or revokes debugging authorizations. [R5]

## 4.4 Driver behavior

USB driver packages are device/OEM-specific. Android's official documentation instructs users to obtain the appropriate driver from the device manufacturer and provides an OEM driver directory. ScrDeck must not assume that one generic driver works for all Android devices. [R6]

ScrDeck should diagnose before recommending driver installation. A phone charging or appearing as an MTP/file-transfer device does not, by itself, prove that its ADB interface is installed and usable.

## 4.5 Windows support posture

- Primary target: Windows 11 x64.
- Compatibility target: Windows 10 22H2 where technically possible, but Windows 10 Home/Pro and several mainstream editions reached end of support on 14 October 2025. ScrDeck must not present an unsupported Windows version as a security-recommended environment. [R7]
- Windows ARM64 packaging is a later release target after the x64 pipeline and dependency availability are validated.
- WebView2 availability and installer behavior must be handled through Tauri's supported Windows installer configuration. [R8]

## 4.6 Assumptions to validate during implementation

- Official upstream assets remain available at the time a release is prepared.
- The application can inspect `scrcpy --version` and `scrcpy --help` output reliably enough to build a local capability registry; if parsing is incomplete, the UI must fail safely and allow a clearly marked advanced override.
- OEM driver detection may be partly heuristic. ScrDeck must label uncertain diagnoses as “possible cause”, not definitive fact.
- Some device metadata (battery, model, Android version, resolution, refresh rate) may be unavailable or vendor-restricted. Missing data must be represented honestly, not filled with invented values.

---

# 5. Scope and release strategy

## 5.1 v0.1 — Functional foundation

**Required for first public release:**

- Windows desktop shell and persistent local settings.
- First-run Setup Assistant.
- Dependency detection and official tool installation/selection workflow.
- Scrcpy and ADB version/path detection.
- USB debugging instructions and ADB authorization handling.
- USB device discovery, selection, refresh, and state display.
- Wireless Debugging pairing-code flow if implementation and device testing are stable; otherwise include it behind an explicit preview feature flag and ship full USB support as the release gate.
- OEM driver assistant with verified official links; no silent driver installation.
- Mirroring Studio Basic Mode and Advanced Mode.
- Live generated command preview.
- Safe custom argument editor.
- Start, monitor, and stop a ScrDeck-owned scrcpy process.
- Local preset create/edit/duplicate/delete/import/export.
- Activity & Logs with common error explanations.
- About page, upstream attribution, dependency notices, and disclaimer.
- NSIS installer `.exe` and portable folder build.
- Automated tests for argument building, device parsing, preset schema, and core process lifecycle.

## 5.2 v0.2 — ADB toolbox expansion

- APK installation and uninstall.
- File push/pull with progress and destination selection.
- Screenshot capture.
- Device reboot actions with explicit confirmation and clear target device.
- Logcat viewer with filters and export.
- More complete Wireless Debugging management and reconnect UX.
- Saved connection profiles and improved multiple-device session workflows.

## 5.3 v0.3 — Advanced workflows

- Multi-session mirroring and per-device independent presets.
- Rich capability inspector and more granular scrcpy options.
- Custom toolchain profiles for users who maintain multiple ADB/scrcpy versions.
- Optional command history with privacy-aware retention.
- Advanced ADB Shell terminal (expert-only, clearly marked, with risk disclosure).
- Optional import/export of support bundles and sanitized diagnostics.

## 5.4 Explicitly deferred

- Native embedded video rendering inside ScrDeck's own window. The initial release opens the official scrcpy window; embedding video is a separate technical project and must not delay the functional MVP.
- Cloud sync, user accounts, team workspaces, telemetry dashboards, or remote relay infrastructure.
- Root-only tools, bootloader flashing, partition management, bypasses, or driver-signature workarounds.
- macOS/Linux ports until the Windows product and process model are stable.

---

# 6. Information architecture and navigation

## 6.1 Primary navigation

1. **Overview** — system readiness, selected/connected device, last-used preset, quick actions.
2. **Devices** — discovered devices, connection state, USB/wireless details, selected device.
3. **Mirroring** — settings, command preview, launch controls, active session summary.
4. **Presets** — built-in and user presets, save/edit/import/export.
5. **ADB Tools** — v0.1 connection utilities; richer toolbox in later releases.
6. **Activity & Logs** — recent operations, process output, diagnostics, export support bundle.
7. **Settings** — toolchain paths, appearance, behavior, privacy, update preferences.
8. **About ScrDeck** — version, license notices, upstream attribution, disclaimer, links.

## 6.2 Navigation principles

- Keep the primary sidebar stable across pages.
- Use a compact top bar for current page title, selected device context, and a small global status indicator.
- Do not create a separate top-level page for every setting category. Mirroring settings belong in Mirroring Studio.
- Command Builder is a contextual panel or drawer, not a permanently dominant full-screen terminal.
- Show device identity and connection state wherever an action depends on the device.
- Preserve the selected device when navigating between Devices, Mirroring, Presets, and Logs.

## 6.3 Navigation behavior

- On narrow window widths, collapse the sidebar to an icon rail with tooltips and accessible names.
- The selected navigation item must remain visually and programmatically identifiable.
- Back navigation should preserve unsaved form state where safe; destructive operations require confirmation.
- The app should restore the last page only after initial environment checks. If setup is incomplete, show the setup continuation state rather than a broken dashboard.

---

# 7. End-to-end user journeys

## 7.1 First launch — new user, USB path

1. App launches into the Setup Assistant.
2. User chooses USB connection.
3. ScrDeck checks OS architecture, dependency files, scrcpy, ADB, and runtime compatibility.
4. If the official scrcpy package is absent, offer to download it from the official upstream release source or let the user locate an existing folder.
5. Verify the downloaded archive against a trusted checksum when upstream publishes one; otherwise clearly disclose that checksum verification was unavailable and do not claim cryptographic verification.
6. User enables Developer Options and USB Debugging on the phone using device-appropriate instructions.
7. User connects a data-capable cable and unlocks the phone.
8. ScrDeck refreshes ADB devices and detects whether the state is `device`, `unauthorized`, `offline`, or absent.
9. If `unauthorized`, explain the RSA dialog and ask the user to approve it on the device.
10. If absent but Windows sees a USB device, run driver/cable troubleshooting and show the official OEM driver route when justified.
11. On successful ADB state, inspect selected device information and validate that scrcpy is callable.
12. Show success summary and open Overview.

## 7.2 First launch — existing developer setup

1. User selects “I already have ADB/scrcpy”.
2. ScrDeck offers browse-to-folder and auto-detect known install locations.
3. User may select an existing scrcpy folder, Android SDK Platform-Tools folder, or Android Studio SDK path.
4. ScrDeck validates executable presence and version and displays which ADB binary will be used.
5. If a conflicting ADB server/version is detected, explain the conflict and offer a controlled restart only after confirmation.
6. Preserve the user's PATH and existing SDK configuration; do not rewrite environment variables silently.

## 7.3 Returning user — connect and mirror

1. Overview shows current toolchain readiness.
2. Device discovery updates device cards.
3. User selects a device and selects a preset.
4. Mirroring Studio displays resolved settings and command preview.
5. User starts mirroring.
6. ScrDeck creates a tracked process, displays “Starting…” then “Running” after process start; process start is not falsely equated with successful video rendering.
7. Process exit, stderr, and known failure messages update the activity panel.
8. User stops the session; ScrDeck stops only the process it launched.

## 7.4 Advanced custom arguments

1. User opens Advanced Mode.
2. User views the generated argv list and may add custom scrcpy arguments.
3. ScrDeck checks for duplicate/conflicting options and flags not present in the installed `scrcpy --help` capability registry.
4. User previews the final command.
5. User explicitly confirms any unsupported/unknown argument override.
6. ScrDeck passes an argv array directly to the executable. It never sends the text through `cmd.exe`, PowerShell, or a shell interpreter.

## 7.5 Wireless Debugging pairing

1. User selects Wireless connection.
2. ScrDeck explains that the phone and PC must be on the same local network and that the user must trust this computer.
3. User enables Developer Options and Wireless Debugging.
4. User chooses “Pair using pairing code” on Android.
5. ScrDeck asks for the displayed IP address, pairing port, and temporary pairing code (or scans a QR code only if a secure, well-tested QR flow is implemented).
6. ScrDeck invokes `adb pair <ip:port>` and handles the pairing code through a secure input path; the code must not be written to ordinary logs.
7. After pairing, ScrDeck discovers the connection endpoint through supported ADB/mDNS mechanisms or asks the user to provide the connection endpoint where needed.
8. ScrDeck runs/observes the connection and verifies the device appears in the ADB device list.
9. Explain how to forget the workstation in Android settings.

## 7.6 Driver troubleshooting

1. ScrDeck checks ADB output and Windows device-enumeration hints.
2. If the device is absent, first ask the user to verify cable/data mode, USB port, phone unlock, USB Debugging, and RSA prompt.
3. If Windows recognizes a phone but no usable ADB interface appears, show “Possible OEM driver issue”.
4. Display detected manufacturer/model only where reliable.
5. Link to the Android OEM driver directory or a verified vendor-specific official support page.
6. Driver installation happens through the OEM installer or Windows Device Manager, with user consent and any required administrator elevation.
7. ScrDeck asks the user to reconnect and re-test.
8. If no trustworthy OEM link exists, show manufacturer support search guidance without inventing a direct download URL.

---

# 8. Functional requirements

Priority codes: **P0** = release-blocking; **P1** = important for v0.1; **P2** = planned follow-up.

## 8.1 Application shell and state

| ID | Priority | Requirement | Acceptance condition |
|---|---|---|---|
| FR-SHELL-01 | P0 | Provide persistent desktop shell with Overview, Devices, Mirroring, Presets, Activity & Logs, Settings, About | All routes load and preserve application state correctly |
| FR-SHELL-02 | P0 | Display global toolchain readiness and current device context | Status is derived from backend state, not hardcoded |
| FR-SHELL-03 | P1 | Persist last selected device and last active page | Restored state is validated on next launch |
| FR-SHELL-04 | P0 | Provide loading, success, empty, error, and disabled states | No primary feature relies on a dead or fake button |

## 8.2 Dependency and toolchain manager

| ID | Priority | Requirement | Acceptance condition |
|---|---|---|---|
| FR-DEP-01 | P0 | Detect scrcpy executable and version | Missing, valid, and unsupported states are distinct |
| FR-DEP-02 | P0 | Detect ADB executable and version | UI identifies exact ADB path selected for ScrDeck |
| FR-DEP-03 | P0 | Prefer the official scrcpy Windows package when it provides a compatible ADB binary | No duplicate ADB download is required on the standard path |
| FR-DEP-04 | P1 | Allow user to locate an existing scrcpy or Platform-Tools folder | Folder selection validates expected binaries |
| FR-DEP-05 | P0 | Download dependencies only from verified official sources | URL host allowlist, version, architecture, and integrity result are displayed |
| FR-DEP-06 | P0 | Keep tool binaries in an app-managed directory when installed by ScrDeck | Do not overwrite unrelated SDK installations |
| FR-DEP-07 | P1 | Show update availability without forcing an update | User controls update timing |
| FR-DEP-08 | P0 | Provide dependency and license notices | Bundled components and versions are inspectable |

## 8.3 ADB device discovery and connection

| ID | Priority | Requirement | Acceptance condition |
|---|---|---|---|
| FR-ADB-01 | P0 | Start/observe ADB server when needed | Start action does not kill or overwrite an unrelated server |
| FR-ADB-02 | P0 | Enumerate devices and parse serial, transport, and ADB state | Correctly handles `device`, `unauthorized`, `offline`, and no-device output |
| FR-ADB-03 | P0 | Refresh device list manually | Loading indicator and updated results are shown |
| FR-ADB-04 | P1 | Update device list when USB devices are attached/detached | Event monitor or conservative polling updates UI without excessive CPU usage |
| FR-ADB-05 | P0 | Let user select one device explicitly | Actions never ambiguously target the first device when multiple devices exist |
| FR-ADB-06 | P0 | Display device model/product and Android version when available | Missing fields show “Unavailable”, never invented values |
| FR-ADB-07 | P1 | Display battery/charging/resolution/refresh rate where supported | Metadata calls are optional and failures do not break discovery |
| FR-ADB-08 | P0 | Support `adb pair` for Wireless Debugging | Pairing code is transient and excluded from ordinary logs |
| FR-ADB-09 | P0 | Support `adb connect` and disconnect/forget guidance | The app confirms the target endpoint and does not silently disconnect unrelated devices |
| FR-ADB-10 | P1 | Show transport type (USB / TCP-IP / emulator) | Type is parsed from actual device identity/connection data where possible |

## 8.4 USB setup and driver assistant

| ID | Priority | Requirement | Acceptance condition |
|---|---|---|---|
| FR-USB-01 | P0 | Explain how to enable Developer Options and USB Debugging | Guidance clearly says settings names vary by OEM |
| FR-USB-02 | P0 | Detect ADB authorization requirement | Unauthorized state produces RSA authorization instructions |
| FR-USB-03 | P0 | Guide users through cable/port/USB-mode checks | Basic troubleshooting is shown before driver download suggestion |
| FR-USB-04 | P0 | Identify possible OEM driver issue | Diagnosis is labelled as likely/possible when not conclusive |
| FR-USB-05 | P0 | Provide verified OEM driver links | Links come from an allowlisted, reviewable source catalogue |
| FR-USB-06 | P0 | Never install unsigned/unknown driver packages silently | Driver installation requires explicit user action and Windows consent/elevation |
| FR-USB-07 | P1 | Provide “re-test connection” action | Discovery reruns and explains whether state changed |

## 8.5 Mirroring Studio and scrcpy launcher

| ID | Priority | Requirement | Acceptance condition |
|---|---|---|---|
| FR-MIR-01 | P0 | Provide Basic Mode for common video, audio, control, and window options | Basic controls map to supported flags discovered from installed scrcpy |
| FR-MIR-02 | P0 | Provide Advanced Mode exposing a broader supported option set | Unsupported options are not presented as guaranteed |
| FR-MIR-03 | P0 | Generate a live command/argv preview | Preview exactly matches final arguments passed to process |
| FR-MIR-04 | P0 | Launch scrcpy for the selected serial | The selected serial is always targeted explicitly when needed |
| FR-MIR-05 | P0 | Track ScrDeck-launched process and session state | Process exit code, PID, elapsed session time, and stderr are captured |
| FR-MIR-06 | P0 | Stop only the selected ScrDeck-owned process | Unrelated scrcpy instances and ADB server are unaffected |
| FR-MIR-07 | P1 | Allow open-in-native-scrcpy-window mode | v0.1 uses upstream native rendering window |
| FR-MIR-08 | P0 | Prevent invalid option/value combinations where known | Validation appears before launch, with explanation |
| FR-MIR-09 | P1 | Allow optional custom recording path | Path is validated and file overwrite behavior is explicit |
| FR-MIR-10 | P1 | Show capability/compatibility hints | Android OS and installed scrcpy version are considered where known |

## 8.6 Preset manager

| ID | Priority | Requirement | Acceptance condition |
|---|---|---|---|
| FR-PRESET-01 | P0 | Ship a small set of built-in presets | Presets map only to known options and are editable through “Save as” |
| FR-PRESET-02 | P0 | Create, edit, duplicate, rename, and delete user presets | Deletion is confirmed; built-ins cannot be overwritten directly |
| FR-PRESET-03 | P0 | Import/export preset JSON | Schema version and validation prevent malformed imports |
| FR-PRESET-04 | P0 | Bind presets to settings, not an opaque command string | Preview is regenerated from structured values and custom args |
| FR-PRESET-05 | P1 | Support per-device last-used preset | Device identity is stored safely and may be reset |

## 8.7 Command Builder

| ID | Priority | Requirement | Acceptance condition |
|---|---|---|---|
| FR-CMD-01 | P0 | Display executable and generated arguments in readable form | User can inspect and copy the preview |
| FR-CMD-02 | P0 | Allow custom arguments | Arguments are passed as argv; no shell interpolation occurs |
| FR-CMD-03 | P0 | Detect known duplicates/conflicts | Conflicts are explained and resolved deterministically |
| FR-CMD-04 | P1 | Offer a one-argument-per-row editor and paste parser | Quoting and escaping tests cover Windows argument semantics |
| FR-CMD-05 | P0 | Preserve unknown advanced arguments when explicitly allowed | User receives a warning before execution |
| FR-CMD-06 | P0 | Block attempts to override ScrDeck-managed target selection without clear resolution | One final device target is used |

## 8.8 Activity, logs, and diagnostics

| ID | Priority | Requirement | Acceptance condition |
|---|---|---|---|
| FR-LOG-01 | P0 | Capture ADB/scrcpy stdout, stderr, exit code, and process metadata | Logs remain available after a failed session |
| FR-LOG-02 | P0 | Translate documented common errors to next actions | Each mapped error has cause, confidence, and next-step wording |
| FR-LOG-03 | P1 | Allow raw log inspection and copy | Technical output is not hidden behind a generic toast |
| FR-LOG-04 | P1 | Export a sanitized diagnostic bundle | Device serials and local paths can be redacted before export |
| FR-LOG-05 | P0 | Avoid logging pairing codes or secrets | Automated tests search for accidental secret leakage |
| FR-LOG-06 | P1 | Filter by process, device, severity, and time | Filtering does not alter source logs |

## 8.9 Settings and About

- Dark mode default; Light and System options supported.
- Configure tool paths and update channel.
- Set behavior on app close (default: stop sessions launched by ScrDeck; optional opt-in to leave them running if safely supported).
- Choose log retention and diagnostic redaction defaults.
- Show app version, build identifier, dependency versions, third-party notices, licenses, disclaimer, and source links.

---

# 9. Screen-by-screen UX specification

## 9.1 Setup Assistant

**Purpose:** get a first-time user from installation to a verified ADB-ready device with as little terminology burden as possible.

### Structure

- Compact ScrDeck identity and step indicator.
- Short title explaining the current task.
- One primary action per step.
- Secondary action for “I already have this set up”, “Skip for now”, or “Back” where safe.
- Context-specific illustration/screenshot only when it materially clarifies an Android or Windows action.
- “Why is this needed?” disclosure for USB debugging, RSA trust, and driver installation.

### Steps

1. Welcome and choose USB or Wireless.
2. Check Windows, scrcpy, ADB, architecture, and dependencies.
3. Prepare Android: Developer Options and USB Debugging / Wireless Debugging.
4. Connect device and check cable/USB mode.
5. Authorize ADB / diagnose driver.
6. Verify connection and open workspace.

Wireless pairing may branch from step 1 or 3. USB remains the recommended first connection because it is easier to diagnose and usually provides a stable baseline.

### UX rules

- Do not ask the user to install Android Studio just to get ADB.
- Do not force a driver installation when ADB already detects the device.
- Do not label a phone “driver broken” based only on an empty ADB list.
- Distinguish “tool missing”, “phone not attached”, “USB driver not ready”, and “authorization pending”.
- Keep a visible “Show technical details” disclosure for experienced users.
- Allow setup to be resumed from Settings or Overview.

## 9.2 Overview

**Purpose:** answer “Is ScrDeck ready, what device is connected, and what can I do next?”

### Layout

- Page heading: “Good evening” or neutral time-independent greeting; greeting is optional and must not dominate.
- System readiness strip: Scrcpy ready, ADB ready, device count.
- Main device panel: selected device identity, transport, ADB state, last seen, and primary action.
- Quick preset row: 2–4 recently used presets.
- Recent activity: last few meaningful actions or errors.
- If no device is connected: a well-designed empty state with “Connect via USB” and “Connect wirelessly”.

Do not add decorative charts, fabricated battery graphs, fake performance metrics, or irrelevant statistics.

## 9.3 Devices

**Purpose:** show discovered Android devices and let the user choose the correct target.

### Device card/list fields

- Display name / model if available.
- Friendly device type and transport.
- Serial ID in secondary/technical text (copyable, optionally masked in support exports).
- ADB state badge: Authorized/Ready, Authorization needed, Offline, or Unavailable.
- Android version and API level if available.
- Battery and charging status only if queried successfully.
- Last seen timestamp for wireless devices where useful.
- Actions: Select, Connect/Disconnect where appropriate, Device details, Troubleshoot.

### Behavior

- Multiple devices must be selectable explicitly.
- When no device is available, show refresh and guided setup actions.
- When two devices share a similar name, show serial or transport differentiators.
- Do not infer manufacturer from a serial string unless a verified device property exists.
- Device details should not block core actions if a metadata query fails.

## 9.4 Mirroring Studio

**Purpose:** configure the upstream scrcpy process through understandable controls.

### Basic Mode categories

- **Display:** max size, FPS, orientation/rotation where supported, fullscreen, always-on-top, window title.
- **Video:** codec selector where supported, quality/bitrate control only when the installed version exposes it, advanced encoder options hidden by default.
- **Audio:** audio forwarding toggle, source/codec options where supported; show Android compatibility hints.
- **Control:** control enablement, keyboard/mouse behavior, stay-awake, screen power options, clipboard behavior where supported.
- **Recording:** enable recording, destination path, filename, and available format/codec options.

### Advanced Mode categories

- Connection/transport.
- Video encoder and codec details.
- Audio source and encoder details.
- Input/keyboard/mouse/HID/OTG controls.
- Window/display/crop/orientation parameters.
- Recording/streaming parameters.
- Server/tunnel parameters.
- Debug/diagnostic flags.
- Custom arguments.

Each option must include a human label, short explanation, supported-version/capability state, source flag, validation rule, and default behavior. Advanced controls must not expose undocumented assumptions as guaranteed behavior.

### Primary actions

- Start Mirroring.
- Stop Session (only while ScrDeck owns a live process).
- Save as Preset.
- Reset to Preset Defaults.
- Copy Command.

## 9.5 Preset Manager

- Built-in presets visually distinct from user-created presets.
- Each preset shows purpose, key settings summary, and last-used metadata if available.
- Create flow starts from current Mirroring settings or a blank configuration.
- Import flow shows a validation summary before saving.
- Export uses a schema-versioned JSON file.
- Do not store executable paths or arbitrary shell scripts in preset JSON.

## 9.6 ADB Tools

v0.1: connection actions, restart ADB server with warning/confirmation if truly required, pair/connect/disconnect wireless, device info refresh, and open troubleshooting.

Later versions: APK install/uninstall, file push/pull, screenshot, reboot, and Logcat. Every destructive action must identify the selected device and request confirmation with consequences clearly written.

## 9.7 Activity & Logs

- Timeline for high-level events: connected, pairing started, device authorized, mirroring started, mirroring exited, recording saved, error occurred.
- Separate raw process log viewer.
- Severity levels: Info, Success, Warning, Error, Debug.
- Filters and text search.
- “Explain this error” panel for known patterns.
- Copy selected text and export sanitized diagnostics.
- Log retention default: rolling local retention with user-configurable duration/size; no cloud upload.

## 9.8 Settings and About

- Appearance: System / Dark / Light.
- Toolchain: managed bundle, custom scrcpy path, custom ADB path, validate versions.
- Behavior: start at Overview or resume setup; app-close process policy; confirmation preferences.
- Privacy: local log retention, redact identifiers by default in exported bundle.
- Updates: stable channel; manual check; official source links.
- About: open-source status, upstream credits, versions, notices, project links, disclaimer.

---

# 10. Visual design system

## 10.1 Design direction

**Industrial Utility — Refined Desktop.** ScrDeck should feel like a mature Windows utility for people who value clarity and control. It should not look like a generic AI-generated SaaS dashboard, a gaming launcher, a neon cyberpunk terminal, or a server-monitoring template.

The visual language should be calm, precise, compact, and intentional. It may have a little personality through the lime accent and strong typography, but decoration must never compete with device status, settings, or actions.

## 10.2 Reference products and design principles

Use the following as research references for specific interaction principles, not as templates to copy:

- **LocalSend:** simple device discovery and an action-centered device workflow. Reference: https://localsend.org/
- **KDE Connect:** the idea of paired/known devices and explicit trust state. Reference: https://kdeconnect.kde.org/
- **Android Studio Device Manager:** structured technical metadata and clear device configuration. Reference: https://developer.android.com/studio
- **Linear:** disciplined spacing, hierarchy, and information density. Reference: https://linear.app/
- **Microsoft Fluent / Windows design guidance:** familiar window behavior, readable typography, navigation, accessibility, calm visuals, and purpose-driven motion. References: [R9][R10]

Do not reproduce any reference app's exact layout, iconography, logo, or proprietary visual assets.

## 10.3 Color tokens

### Dark theme (default)

| Token | Hex | Usage |
|---|---|---|
| `bg.canvas` | `#171B20` | Main window background |
| `bg.surface` | `#222830` | Main cards and panels |
| `bg.elevated` | `#2B333D` | Popovers, menus, elevated panels |
| `bg.subtle` | `#303943` | Secondary interactive surface |
| `text.primary` | `#F1F3EF` | Main text |
| `text.secondary` | `#AAB3BC` | Supporting text |
| `text.muted` | `#7F8A95` | Low-priority metadata; never for essential content |
| `accent.lime` | `#C5E66A` | Primary action, focus-adjacent accent, selected indicator |
| `status.success` | `#55BE91` | Successful/authorized/connected state |
| `status.warning` | `#E4A84F` | Needs attention / possible issue |
| `status.error` | `#E27676` | Failure / blocked action |
| `border.default` | `#39424C` | Dividers and panel outlines |

Use lime with dark foreground text for filled primary buttons. Do not use lime as long-form text on a bright surface. Status must never rely on color alone: include label and/or icon.

### Light theme (secondary)

| Token | Suggested value | Usage |
|---|---|---|
| `bg.canvas` | `#F4F6F2` | Main background |
| `bg.surface` | `#FFFFFF` | Panels and cards |
| `bg.elevated` | `#FFFFFF` | Menus/dialogs with subtle shadow or border |
| `text.primary` | `#18211D` | Main text |
| `text.secondary` | `#5D6862` | Supporting text |
| `accent.lime` | `#C5E66A` | Primary accent paired with dark text |
| `status.success` | `#237A55` | Success with label/icon |
| `status.warning` | `#966019` | Warning with label/icon |
| `status.error` | `#B33B42` | Error with label/icon |
| `border.default` | `#D8DED8` | Dividers and outlines |

Validate all foreground/background combinations against WCAG 2.2 AA targets for normal text (4.5:1) and large text (3:1). The token values are a starting palette, not a substitute for contrast testing.

## 10.4 Typography

- Primary UI font: **Segoe UI Variable** where available; fall back to Segoe UI and system sans-serif.
- Technical text: **IBM Plex Mono** where bundled/licensed appropriately; fallback to Cascadia Mono / Consolas.
- Avoid using more than two font families.
- Use tabular numerals for version numbers, FPS, bitrate, timestamps, and device values.
- Suggested sizes: page title 24–28 px; section title 16–18 px; body 13–14 px; secondary/help 12–13 px; technical code 12–13 px.
- Avoid ultra-light weights and excessively tight tracking.

## 10.5 Geometry and spacing

- Base spacing unit: 4 px.
- Common gaps: 8, 12, 16, 20, 24, 32 px.
- Standard control height: 36–40 px.
- Compact control height: 30–32 px only for dense technical tables.
- Common radius: 8–12 px; larger 14–16 px for major panels only.
- Border: 1 px subtle separators; use shadows sparingly.
- Content max width should adapt to window size; avoid stretching forms across ultrawide monitors.
- Initial window target: approximately 1280 × 800 px; minimum usable size target: 960 × 640 px.

## 10.6 Iconography and brand mark

- Use Lucide icons consistently, preferably 18–20 px for navigation and 16 px for inline actions.
- Do not mix multiple icon families.
- Icons must be paired with visible labels for high-impact or ambiguous actions.
- Brand concept: two offset device outlines plus a small connection point/bridge. Keep the mark geometric, legible at 16–32 px, and original.
- Do not use the official Android robot or scrcpy logo as ScrDeck's own mark.

## 10.7 Motion

- Use motion only for feedback, spatial transition, and state communication.
- Typical duration: 140–220 ms for hover/focus/expand; 180–260 ms for page or drawer transitions.
- Prefer opacity/position/height transitions that remain subtle.
- Respect Windows reduced-motion/accessibility settings where available.
- No ambient particle effects, looping decorative animation, animated gradients, or oversized celebration effects.
- Loading indicators must represent real asynchronous work.

## 10.8 Anti-patterns — prohibited design outcomes

- Generic dashboard with meaningless cards or fabricated metrics.
- Giant hero headline occupying most of the workspace after onboarding.
- Neon/glow outlines around every control.
- Glassmorphism and blur used as a substitute for hierarchy.
- Gradient backgrounds used as the primary visual identity.
- Randomly mixed corner radii, shadows, and icon styles.
- Overly large empty cards with little useful information.
- Color-only device states with no text label.
- Fake buttons, decorative sliders, non-functional toggles, or inert menus.
- Excessive confirmation dialogs for harmless actions.
- Hidden critical errors behind a toast that disappears too quickly.

---

# 11. Interaction, content, and accessibility standards

## 11.1 UX principles

1. **One dominant action per screen:** make the next step obvious.
2. **Progressive disclosure:** beginner-friendly Basic Mode; power-user depth in Advanced Mode.
3. **Feedback at the point of action:** show loading and resulting state beside the control that triggered it.
4. **Error recovery over blame:** tell the user what likely happened and what to do next.
5. **Transparency:** show selected device, target serial, executable path, active preset, and command preview when relevant.
6. **Consistency:** the same action has the same label, icon, and confirmation behavior throughout the app.
7. **No surprise side effects:** no silent driver installation, environment variable mutation, ADB server shutdown, device reboot, or destructive file operation.

## 11.2 Content design / microcopy

Prefer plain, direct language:

- “Your phone needs permission” instead of “ADB unauthorized error”.
- “No Android device detected” instead of “Device not found” without context.
- “Possible USB driver issue” instead of “Driver broken”.
- “Scrcpy exited with an error” with the actual exit code and details available.
- “This computer is allowed to debug this phone” to explain RSA authorization.

Technical wording may appear in expandable details. Avoid promising a fix when the diagnosis is uncertain.

## 11.3 Accessibility

- All primary workflows must be usable by keyboard.
- Visible focus indicator for every interactive control.
- Logical tab order matching visual order.
- Buttons and icon-only actions have accessible names.
- Do not use color alone to communicate status.
- Ensure text contrast against the actual rendered surface.
- Respect text scaling and avoid fixed-height controls that clip labels.
- Provide tooltips for icon-only navigation on collapsed sidebar.
- Announce asynchronous state changes through accessible status/live regions where supported.
- Support reduced-motion preference.
- Test at 100%, 125%, 150%, and 200% display scaling.

---

# 12. Technical architecture

## 12.1 Recommended stack

| Layer | Technology | Responsibility |
|---|---|---|
| Desktop shell | Tauri 2 | Native window, installer, local app packaging, IPC boundary |
| Frontend | React + TypeScript + Vite | UI, routing, forms, settings views |
| Styling | Tailwind CSS + explicit design tokens | Consistent visual system |
| Icons | Lucide | Single icon family |
| UI state | Zustand or a similarly small typed store | Selected device, active page, modal state, preferences |
| Async data/state | Tauri events + typed service hooks; use a query library only if justified | Device refresh, process status, async command results |
| Backend | Rust | Process manager, ADB/scrcpy invocation, file operations, validation |
| Serialization | Serde + JSON | Configuration, presets, event payloads |
| Local persistence | SQLite for structured records, or versioned JSON for a small initial footprint | Settings, presets, bounded history |
| Tests | Vitest + React Testing Library; Rust unit/integration tests | UI behavior, parsers, argument generation, process logic |
| CI | GitHub Actions | Build/test/package/release artifacts |

Do not add a framework or state library merely because it is popular. Keep the dependency graph lean and explain any additional dependency in the architecture decision record.

## 12.2 Architecture overview

```text
┌─────────────────────────────────────────────────────┐
│                   ScrDeck Desktop                   │
│                                                     │
│  React + TypeScript + Tailwind                      │
│  Overview | Devices | Mirroring | Presets | Logs     │
│                    │ Tauri IPC                      │
├────────────────────┼────────────────────────────────┤
│                 Rust Application Core               │
│                                                     │
│  Toolchain Manager  Device Service  Preset Service  │
│  Process Manager   Diagnostics     Settings Store  │
│                    │                                │
├────────────────────┼────────────────────────────────┤
│            Managed External Tool Resources          │
│                                                     │
│         scrcpy.exe + its companion files             │
│         adb.exe + required Platform-Tools files      │
│                                                     │
└────────────────────┬────────────────────────────────┘
                     │ USB / TCP-IP / ADB server
                     ▼
                Android device
```

## 12.3 Responsibility boundaries

### React frontend

- Renders views and forms.
- Holds transient presentation state.
- Requests typed operations from Rust through Tauri commands.
- Subscribes to typed backend events for device changes, process output, and session status.
- Does not directly spawn arbitrary local executables.
- Does not decide tool version compatibility without backend-provided capability data.

### Rust backend

- Resolves approved executable paths.
- Invokes ADB and scrcpy using argument arrays.
- Owns child process handles and process lifecycle.
- Parses device lists and version/help output.
- Validates preset schemas and command arguments.
- Emits typed events to the UI.
- Persists settings and logs.
- Enforces executable allowlists, URL allowlists, and safe filesystem paths.

### Upstream binaries

- Scrcpy and ADB are treated as external tools.
- Never patch the binary at runtime.
- Never invent replacement behavior for a missing upstream feature.
- Keep executable version and package provenance visible.

## 12.4 Process invocation and Tauri security

Prefer a narrowly scoped Rust process service using `std::process::Command` or `tokio::process::Command` over giving the frontend broad shell access. Construct the executable path from validated toolchain configuration and pass arguments as separate argv entries. Never concatenate user input into a shell command and execute it through `cmd.exe`, PowerShell, or another interpreter.

Tauri's sidecar/resource mechanisms may be used for packaging managed binaries, but the scrcpy Windows release is a folder/archive containing companion libraries and files. Package the complete required upstream folder rather than copying only `scrcpy.exe`. Review Tauri's external-binary and shell permission model before implementation. [R11][R12]

Grant only the Tauri capabilities the frontend actually needs. The frontend should not receive unrestricted process execution, arbitrary filesystem access, or arbitrary URL-opening privileges.

## 12.5 Suggested repository layout

```text
scrdeck/
├─ README.md
├─ LICENSE
├─ THIRD_PARTY_NOTICES.md
├─ docs/
│  ├─ PRODUCT_SPEC.md
│  ├─ ARCHITECTURE.md
│  ├─ UX_GUIDELINES.md
│  ├─ SECURITY.md
│  └─ RELEASE_PROCESS.md
├─ public/
│  ├─ brand/
│  └─ illustrations/
├─ src/
│  ├─ app/
│  ├─ components/
│  ├─ features/
│  │  ├─ onboarding/
│  │  ├─ devices/
│  │  ├─ mirroring/
│  │  ├─ presets/
│  │  ├─ adb-tools/
│  │  ├─ activity/
│  │  └─ settings/
│  ├─ hooks/
│  ├─ services/
│  ├─ stores/
│  ├─ styles/
│  └─ types/
├─ src-tauri/
│  ├─ src/
│  │  ├─ commands/
│  │  ├─ services/
│  │  ├─ process/
│  │  ├─ parsers/
│  │  ├─ models/
│  │  ├─ storage/
│  │  └─ main.rs
│  ├─ capabilities/
│  ├─ resources/
│  │  └─ tools/ (managed tool folder; populated by packaging pipeline)
│  ├─ icons/
│  └─ tauri.conf.json
├─ tests/
│  ├─ fixtures/
│  ├─ integration/
│  └─ e2e/
└─ .github/workflows/
```

The exact folder names may evolve, but keep frontend features, backend services, process execution, parsers, storage, and third-party resources clearly separated.

## 12.6 Backend command surface (proposed)

Use typed Tauri commands such as:

- `get_environment_status()`
- `detect_toolchain()`
- `select_toolchain_paths(paths)`
- `install_managed_tools(channel, architecture)`
- `get_devices()`
- `refresh_devices()`
- `start_adb_server()`
- `pair_wireless_device(address, pairing_code)`
- `connect_wireless_device(address)`
- `disconnect_wireless_device(serial)`
- `get_device_details(serial)`
- `get_scrcpy_capabilities()`
- `build_scrcpy_command(request)`
- `start_mirroring(request)`
- `stop_mirroring(session_id)`
- `get_active_sessions()`
- `get_presets()`
- `save_preset(preset)`
- `import_preset(file_path)`
- `export_preset(preset_id, file_path)`
- `get_activity_logs(filter)`
- `export_diagnostics(options)`

Names are illustrative. The API must be typed, validated, and minimal. Do not expose one generic `execute_command(string)` IPC endpoint to the UI.

## 12.7 Capability registry

At startup or when a toolchain changes:

1. Resolve the selected `scrcpy.exe` path.
2. Run version detection.
3. Run `scrcpy --help` using a controlled process timeout.
4. Parse supported long/short options and relevant value forms where stable.
5. Build a typed capability registry with source version and detection confidence.
6. Map visual controls to capabilities.
7. Hide or clearly disable unsupported controls.
8. Keep a manual “advanced override” path for flags not parsed by the registry.

Do not assume current master documentation is identical to the installed release. The installed executable's help output is the primary runtime feature source. The official repository's documentation remains the source for semantic interpretation. [R1]

---

# 13. Data models and configuration formats

## 13.1 Core entities

### Toolchain

- `scrcpy_path`
- `scrcpy_version`
- `adb_path`
- `adb_version`
- `source_type` (managed bundle / user-selected / detected SDK)
- `architecture`
- `last_validated_at`
- `integrity_status`

### Device

- `serial`
- `model` (optional)
- `product` (optional)
- `transport` (USB / TCP-IP / emulator / unknown)
- `adb_state` (device / unauthorized / offline / no-permission / unknown)
- `android_release` (optional)
- `api_level` (optional)
- `battery_percent` (optional)
- `charging_state` (optional)
- `last_seen_at`

### MirrorSettings

- `max_size`
- `max_fps`
- `video_codec`
- `audio_enabled`
- `audio_source`
- `fullscreen`
- `always_on_top`
- `stay_awake`
- `turn_screen_off`
- `control_enabled`
- `recording_enabled`
- `recording_path`
- `recording_container`
- `custom_args[]`

Only persist fields supported by the selected configuration version; keep unsupported user fields if safely possible during migration rather than silently deleting them.

### Preset

- `id` (UUID)
- `name`
- `description`
- `is_builtin`
- `schema_version`
- `created_at`
- `updated_at`
- `settings`
- `custom_args[]`
- `compatibility_notes[]`

### Session

- `session_id`
- `device_serial`
- `preset_id` (optional)
- `started_at`
- `ended_at` (optional)
- `process_id`
- `executable_path`
- `argv[]`
- `exit_code` (optional)
- `status` (starting / running / stopping / exited / failed)

Do not persist sensitive pairing codes in any of these entities.

## 13.2 Preset storage

Use a versioned JSON format for portability. SQLite may be used for application history and settings, but presets should remain human-readable and easy to import/export.

Example shape:

```json
{
  "schemaVersion": 1,
  "id": "preset-balanced-001",
  "name": "Balanced",
  "description": "A sensible everyday mirroring profile.",
  "settings": {
    "maxSize": 1920,
    "maxFps": 60,
    "videoCodec": "h264",
    "audioEnabled": true,
    "fullscreen": false,
    "stayAwake": true,
    "turnScreenOff": false,
    "recordingEnabled": false
  },
  "customArgs": []
}
```

Example values are starting defaults only. At runtime, confirm that the installed scrcpy version supports the mapped options. Do not interpret a preset as a fully portable guarantee across every scrcpy version or Android device.

## 13.3 Migration behavior

- Every persisted configuration has a schema version.
- Migrations must be deterministic and covered by tests.
- Keep a backup of prior configuration before a destructive migration.
- If a preset contains a flag no longer supported by the installed scrcpy, retain it as an unsupported custom argument and warn the user.
- Do not silently drop settings or replace them with a different value.

---

# 14. Dependency, ADB, driver, and device setup behavior

## 14.1 Dependency strategy

Default path:

1. Download or use the official scrcpy Windows archive.
2. Extract the complete package into ScrDeck-managed storage.
3. Validate required binaries and companion files.
4. Use the ADB binary bundled with that package unless the user deliberately selects a different validated Platform-Tools installation.
5. Record package version, source URL, architecture, hash/checksum availability, and date.

Alternative path:

- Let advanced users select an existing scrcpy folder or Android SDK Platform-Tools directory.
- Do not overwrite the user's files.
- Do not silently modify PATH or registry values.
- If multiple ADB binaries are discovered, display which one ScrDeck will invoke and why.

## 14.2 Download and integrity policy

- Fetch binaries only from official upstream URLs or official release assets.
- Maintain an allowlist of permitted upstream hosts.
- Prefer official release checksums or signed release metadata when available.
- If a checksum is not published, state “Checksum not provided by upstream; authenticity relies on the official source connection” rather than claiming verification.
- Do not run downloaded binaries before extraction and validation complete.
- Do not follow arbitrary redirect destinations outside the allowlist without validation.
- Store downloaded archive and extracted versioned directory separately until verification succeeds.
- Provide rollback to the previous managed tool version.

## 14.3 USB debugging instructions

Provide general instructions with an OEM caveat:

1. Unlock Android and open Settings.
2. Open About phone / About device.
3. Find Build number (sometimes inside Software information).
4. Tap Build number seven times or until Developer Options is enabled; device authentication may be required.
5. Return to Settings and open Developer Options.
6. Enable USB debugging.
7. Connect with a known data-capable cable and unlock the phone.
8. Accept the RSA authorization dialog shown by Android.

Menu labels differ across Samsung, Xiaomi/Redmi, Motorola, Pixel, Oppo, Vivo, OnePlus, and other OEMs. The UI must label this as a common path and offer a manufacturer-specific help link where a verified source exists.

## 14.4 Driver assistant policy

A driver assistant is a guided helper, not an automatic driver installer.

Detection sequence:

1. Query ADB device list.
2. If device state is `device`, do not suggest installing a driver.
3. If `unauthorized`, request RSA approval; driver installation is not the first recommendation.
4. If `offline`, recommend unlock/reconnect/ADB recovery checks before driver replacement.
5. If ADB sees no device, check if a compatible USB device/interface is visible through supported Windows enumeration methods.
6. Ask about cable, USB port, data-transfer mode, and USB debugging.
7. If evidence suggests missing OEM ADB interface, show official OEM driver guidance.

Do not:

- Bundle random driver packs.
- Force-install a driver with no user consent.
- Disable driver signature enforcement.
- Ask users to run an unknown executable as administrator.
- Claim that MTP detection proves ADB is ready.

## 14.5 Wireless Debugging

- Explain same-network requirement.
- Use Android's pairing-code workflow for supported devices (commonly Android 11+ Wireless Debugging).
- Use the pairing IP/port shown by the phone; do not assume the pairing port equals the later connect port.
- Keep pairing code in memory only as long as required.
- Never store pairing code in logs, analytics, preset, or diagnostic bundle.
- After pairing, re-discover the connection endpoint and verify ADB state.
- Provide “Forget this device” instructions and explain revoking ADB authorizations.
- Warn users to avoid pairing on public/untrusted Wi-Fi.

## 14.6 Shared ADB server policy

- Use the selected ADB binary consistently.
- Detect if the server is already running where practical.
- If server incompatibility is detected, explain the issue before restarting.
- Do not call `adb kill-server` on normal app startup or exit.
- If a restart is needed, ask for confirmation and explain that other Android development tools may temporarily lose ADB connectivity.
- Never terminate all processes named `adb.exe` indiscriminately.

---

# 15. Process lifecycle, command generation, and error handling

## 15.1 Command builder pipeline

1. Receive a typed settings object from the UI.
2. Resolve selected device serial.
3. Resolve installed scrcpy capability registry.
4. Validate values and option compatibility.
5. Convert structured settings to ordered argv entries.
6. Append custom arguments after conflict checking.
7. Produce a human-readable preview and a machine-readable argv array.
8. Require an explicit user override for unknown/unsupported flags.
9. Execute the binary directly with argv.
10. Capture stdout/stderr and exit status.

The preview must be derived from the same argv array passed to the process, not separately reconstructed by frontend string templates.

## 15.2 Safe argument handling

- Never pass custom text through `cmd.exe /c`, PowerShell `-Command`, or a shell.
- Use an argument array with each argument separated.
- For a paste-style field, parse Windows command-line quoting rules and show the parsed argument list before execution.
- Provide a one-argument-per-row editor as the unambiguous advanced alternative.
- Detect user attempts to set duplicate device-target flags.
- Do not allow custom arguments to silently override critical ScrDeck-managed settings; show the conflict and let the user choose one source.
- Distinguish “unknown flag” from “invalid value” and “known but unsupported by this device”.

## 15.3 Process lifecycle

- Each ScrDeck-launched scrcpy process receives an internal session ID and tracked PID/handle.
- Track start time, state, exit code, and output.
- Stop action targets only the selected session.
- Prefer a Windows Job Object or an equivalent controlled process-tree lifecycle mechanism for processes owned by ScrDeck; do not use broad image-name termination.
- Do not stop or kill the shared ADB server when a mirroring process exits.
- On application exit, default behavior is to stop ScrDeck-owned scrcpy sessions. If “keep sessions running after app close” is implemented, it must be explicit, tested, and visible in Settings.
- The app must handle device unplug while mirroring and report that the session ended or failed.

## 15.4 Session states

`idle → validating → starting → running → stopping → exited`

Failure paths:

- `validating → blocked` (unsupported or invalid options)
- `starting → failed` (binary missing, process could not start, immediate failure)
- `running → failed` (process exits unexpectedly or device disconnects)

Never display “Mirroring” merely because a button was clicked. Reflect actual process state and, where available, meaningful upstream output.

## 15.5 Common diagnostic catalogue

| Condition | User-facing title | Suggested next step |
|---|---|---|
| scrcpy executable missing | Scrcpy was not found | Locate an existing folder or install from official upstream |
| ADB executable missing | ADB is not available | Use the managed scrcpy bundle or select Platform-Tools |
| No ADB device | No Android device detected | Check cable/data mode, USB debugging, and device connection |
| `unauthorized` | Your phone needs permission | Unlock phone and approve the RSA debugging prompt |
| `offline` | Device connection is not responding | Reconnect, unlock phone, and re-test; show technical details |
| multiple devices | Choose a device | Select one device explicitly |
| server version mismatch | ADB versions may conflict | Show client/server versions and offer confirmed restart |
| unsupported flag | This option is not supported by the installed version | Remove it, update the tool, or explicitly override if safe |
| unsupported Android capability | This option may not work on this device | Explain version/device constraint and allow disabling the option |
| scrcpy exits non-zero | Scrcpy could not complete the session | Show exit code, stderr excerpt, and diagnostic actions |
| wireless pair fails | Pairing did not complete | Re-check pairing code, IP, port, network, and Wireless Debugging state |
| wireless device not connected after pair | Pairing completed, but connection is not active | Discover the connect endpoint and run connection verification |

Each error mapping must store: detection pattern, confidence, user-facing message, explanation, next steps, technical detail, source reference, and tested versions. Unknown errors must remain visible as raw output.

---

# 16. Security, privacy, licensing, and trust

## 16.1 Security principles

- Local-first by default.
- Least privilege.
- Explicit consent for driver installation, app install/uninstall, reboot, wireless pairing, and any destructive action.
- No broad arbitrary shell execution from the frontend.
- No silent privilege elevation.
- No hidden data collection.
- No cloud upload of device identifiers or logs.

## 16.2 Local data and retention

- Settings and presets are stored locally in the user's application data directory.
- Logs are local, bounded, and user-configurable.
- Pairing codes and other temporary credentials are memory-only and redacted.
- Diagnostic export is opt-in and sanitized by default.
- Serial numbers, local paths, usernames, and IP addresses should be redacted from shared support bundles unless the user explicitly includes them.
- Do not add telemetry in v0.1. If opt-in diagnostics/telemetry are proposed later, require a separate product/privacy decision and clear disclosure.

## 16.3 Network behavior

Core mirroring does not require a ScrDeck cloud service. Network access is needed only for explicit actions such as downloading/updating official tool packages, opening official driver links, or using wireless ADB on the user's local network.

- Do not route Android screen content through ScrDeck servers.
- Do not create a remote relay or remote-control tunnel.
- Do not upload recording files.
- Clearly separate local wireless debugging from internet/remote access.

## 16.4 Licensing and attribution

- Scrcpy's repository identifies Apache License 2.0. [R1]
- ADB/Platform-Tools and every dependency must be reviewed under their actual applicable terms at the time they are redistributed. Do not assume all dependencies share scrcpy's license.
- Keep a third-party notices directory/file with component name, version, source URL, license identifier, copyright notice, and any redistribution obligations.
- Preserve upstream notices and license texts when redistributing.
- Review use of names and marks. Use wording such as “for scrcpy” and “powered by scrcpy” without implying endorsement.
- The final license for ScrDeck itself is a project-owner decision. A permissive license such as MIT or Apache-2.0 may fit the community-tool goal, but it must be selected deliberately and checked against bundled dependencies.

## 16.5 Trust and installer behavior

- Provide publisher information and digital signing when a code-signing certificate is available.
- If the release is unsigned, say so in release notes; do not hide SmartScreen warnings or advise users to bypass security blindly.
- Publish SHA-256 hashes for ScrDeck release artifacts.
- Build release artifacts through reproducible/traceable CI where feasible.
- Keep dependency versions and their upstream origin visible in About.

---

# 17. Non-functional requirements

## 17.1 Performance

Targets for a mid-range Windows 11 x64 computer (to be measured and adjusted with real hardware):

- Cold application launch to interactive Overview: target ≤ 3 seconds after first-run initialization.
- Device refresh: target ≤ 3 seconds on a typical USB-connected device, excluding slow vendor-specific responses.
- No blocking UI thread during process invocation, downloads, device queries, or file operations.
- Log streaming must remain responsive during long-running sessions.
- Idle device monitoring must have low CPU overhead; prefer ADB event tracking or conservative polling with backoff.

## 17.2 Reliability

- ADB or scrcpy failure must not crash the entire UI.
- Device disconnect must update state and release the relevant process/session resources.
- Malformed preset import must not corrupt existing presets.
- Tool update failure must preserve the last working managed version.
- App restart must recover configuration and not invent an active session if the process is gone.

## 17.3 Compatibility

- Windows 11 x64 is the primary supported target.
- Windows 10 22H2 may be supported on a best-effort basis, with a visible OS-support caveat because mainstream Microsoft support ended 14 October 2025. [R7]
- Scrcpy/ADB architecture must match host architecture.
- Android capability depends on installed scrcpy version and device API level; the UI must expose limitations.

## 17.4 Maintainability

- Keep process execution, parsing, UI rendering, and storage separated.
- All external commands must be wrapped in testable services.
- Use typed structures, not unvalidated string maps, for primary settings.
- Maintain a compatibility test matrix for supported scrcpy and ADB versions.
- Document architecture decisions and migrations.

## 17.5 Localization readiness

- Ship English as the first UI language unless project owner specifies otherwise.
- Keep all user-facing strings in a localization resource layer.
- Design labels to accommodate longer translations.
- Indonesian localization can be added without re-layout or hardcoded strings.

---

# 18. Testing and quality assurance

## 18.1 Test layers

### Unit tests

- ADB output parser (`device`, `unauthorized`, `offline`, multiple devices, whitespace/encoding variations).
- scrcpy version parser and help/capability parser.
- Settings-to-argv mapper.
- Windows command-line argument parser.
- Conflict and duplicate flag detector.
- Preset validation, import/export, and migration.
- Error classifier.
- Log redaction and support-bundle sanitizer.

### Integration tests

- Resolve and run toolchain executables from managed and user-selected paths.
- Start/observe ADB server without killing unrelated processes.
- Enumerate devices using fixture outputs and a real device test bench.
- Launch and stop a scrcpy process.
- Handle app close and device unplug.
- Validate package extraction and checksum flow.

### UI tests

- Setup Assistant branches.
- Device selection with zero, one, and multiple devices.
- Unauthorized/offline states.
- Basic and Advanced Mirroring settings.
- Command preview updates when controls change.
- Preset creation and selection.
- Log filters and error explanations.
- Keyboard navigation and focus states.

### Manual hardware tests

- At least one Google Pixel or AOSP-like device.
- At least one Samsung device.
- At least one Xiaomi/Redmi device.
- At least one other OEM where available.
- USB data cable and multiple USB ports.
- USB Debugging off/on.
- RSA not accepted/accepted/revoked.
- Wireless Debugging pairing on a supported Android version.
- Device lock/unlock and disconnect during active mirroring.

## 18.2 Test matrix

| Scenario | Expected behavior |
|---|---|
| Fresh install, no tools | Setup offers official dependency installation |
| Existing scrcpy folder | Detect and validate without copying/overwriting user files |
| Existing Android Studio Platform-Tools | Detect path and let user choose it |
| No phone attached | Empty state with cable/setup instructions |
| USB device, debugging disabled | Explain Developer Options / USB Debugging |
| ADB unauthorized | Explain RSA prompt and wait for approval |
| ADB offline | Show reconnection steps and raw state |
| Multiple devices | Require explicit target selection |
| Driver missing | Show possible OEM driver path; never auto-install silently |
| Wireless wrong code | Show pairing failure and allow retry without logging code |
| Wireless pair success, no connect | Discover/request connect endpoint and verify |
| Invalid preset JSON | Reject import without mutating current data |
| Unknown scrcpy flag | Warn and require explicit advanced override |
| Conflicting serial argument | Resolve visibly; do not launch ambiguous target |
| scrcpy process crashes | Capture stderr/exit code, mark failed, keep UI responsive |
| Device unplugged mid-session | Mark session ended/failed and allow reconnect |
| App closes with session running | Follow explicit process-close policy |
| Tool update interrupted | Keep previous verified toolchain usable |

## 18.3 Release gates

Do not publish a release unless:

- All P0 requirements are implemented and tested.
- There are no fake primary controls or misleading success states.
- All packaged executables have verified provenance and notices.
- The first-run USB workflow succeeds on at least two tested OEM devices.
- Command preview and executed argv are byte-for-byte/logically identical after normalization.
- Stop action only affects ScrDeck-owned sessions.
- No pairing code appears in logs or diagnostic exports.
- Installer and portable builds are tested on a clean Windows user profile.
- Accessibility and scaling checks pass.
- README and release notes accurately describe the wrapper relationship.

---

# 19. Packaging, release, and maintenance

## 19.1 Windows distribution

Produce two formats:

1. **Installer `.exe`** using Tauri's supported NSIS target. The installer should handle install directory, shortcuts, uninstallation, and WebView2 prerequisite strategy.
2. **Portable package** as a folder/ZIP containing the app executable and required resources. Portable mode must not depend on an undocumented developer machine path.

Tauri's Windows installer documentation describes NSIS/MSI paths and WebView2 installation modes. Prefer the supported bootstrapper strategy unless project constraints justify another option. Do not skip WebView2 prerequisite checks if the application needs the runtime. [R8]

## 19.2 Architecture rollout

- v0.1: Windows x64.
- Later: Windows ARM64 after official upstream scrcpy and ADB dependencies, Tauri target, and installer pipeline are validated.
- 32-bit Windows is not a priority and should not be promised unless explicitly tested.

## 19.3 Release pipeline

1. Run formatting, lint, frontend tests, Rust tests, parser fixtures, and security checks.
2. Build app for target architecture.
3. Fetch/package only approved upstream tool versions.
4. Generate third-party notices and dependency manifest.
5. Validate installer and portable package on a clean VM.
6. Generate SHA-256 checksums.
7. Sign binaries if an approved certificate is available.
8. Publish GitHub release with version, known limitations, upstream dependency versions, checksums, and source commit.
9. Monitor upstream scrcpy and Platform-Tools releases for compatibility-impacting changes.

## 19.4 Versioning

Use semantic versioning for ScrDeck itself:

- MAJOR: breaking configuration/API changes.
- MINOR: backward-compatible features.
- PATCH: bug fixes/security updates.

Keep ScrDeck version separate from scrcpy and ADB versions. Display all three clearly in About and diagnostic bundles.

## 19.5 Maintenance policy

- Revalidate upstream downloads and checksums at each release.
- Re-run capability parsing tests against each supported scrcpy version.
- Review OEM driver catalogue links quarterly or before a major release.
- Refresh legal/third-party notices when binaries change.
- Maintain a compatibility matrix for current stable and previous stable scrcpy versions where practical.
- Never silently update upstream tools while an active session depends on them.

---

# 20. Risks, constraints, and decisions

| Risk | Impact | Mitigation |
|---|---|---|
| scrcpy flags change between versions | Command builder may generate invalid args | Runtime capability registry plus regression fixtures |
| Multiple ADB binaries/server versions | Device detection or server conflict | Explicit toolchain path, version display, confirmed restart only |
| OEM driver complexity | Novice cannot connect phone | Guided diagnosis and official OEM links; no false certainty |
| Unsigned driver installers | Security risk | Never bundle or silently install unknown drivers |
| Custom argument parsing mistakes | Incorrect invocation or security risk | argv array only, tested Windows argument parser, no shell |
| scrcpy process tree behavior differs | Stop action may leave child process | Windows Job Object / controlled process lifecycle tests |
| Native mirroring window cannot be embedded | UI expectations mismatch | State clearly that v0.1 opens native scrcpy window |
| Bundled binary licensing is misunderstood | Legal/reputation risk | Third-party notices and release-time license review |
| Feature scope expands into full Android management suite | Delayed MVP | Keep v0.1 focused on setup, device selection, mirroring, presets, and diagnostics |
| UI looks like a generic template | Poor differentiation/usability | Enforce design system and anti-pattern checklist |

## 20.1 Key architecture decisions

**ADR-001 — Use scrcpy as the actual mirroring engine.**  
Decision: invoke the official upstream binary.  
Reason: avoids duplicating a mature native implementation and keeps attribution clear.

**ADR-002 — Use Tauri 2 + React + TypeScript + Rust.**  
Decision: local desktop shell with frontend UI and Rust orchestration.  
Reason: native process control, manageable packaging, and a responsive modern UI.

**ADR-003 — Do not embed mirroring video in v0.1.**  
Decision: scrcpy opens its own native window.  
Reason: minimize scope and preserve the upstream rendering path.

**ADR-004 — Prefer the official scrcpy Windows bundle for the default toolchain.**  
Decision: use its included ADB where compatible; allow custom Platform-Tools selection.  
Reason: fewer duplicate binaries and simpler first-run setup.

**ADR-005 — Driver helper, not driver installer.**  
Decision: detect and guide; user installs the official OEM package.  
Reason: reduce privilege and supply-chain risk.

**ADR-006 — Basic/Advanced settings plus transparent custom argv.**  
Decision: progressive disclosure with command preview.  
Reason: approachable defaults without sacrificing power-user flexibility.

---

# 21. Definition of done and acceptance criteria

A release candidate is accepted only when all statements below are true:

1. A clean Windows machine with no existing ADB/scrcpy can install ScrDeck and follow the managed dependency flow.
2. A user can point ScrDeck to an existing scrcpy/ADB installation without modifying their original files.
3. A USB-connected phone with USB Debugging enabled is detected and its authorization state is correctly represented.
4. An unauthorized phone produces a clear instruction to unlock and approve the RSA prompt.
5. Missing or problematic drivers produce a careful diagnostic path and official OEM guidance, not a forced install.
6. A user can select a device explicitly when more than one is connected.
7. Basic Mirroring settings generate a valid argv array for the installed scrcpy version.
8. Advanced custom arguments are visible and passed without shell execution.
9. The command preview is the same invocation that the backend actually executes.
10. ScrDeck launches and tracks a scrcpy process and can stop only its own session.
11. A process failure leaves the application usable and exposes raw technical details.
12. A user can create, edit, duplicate, delete, import, and export presets.
13. Core mirroring works without a ScrDeck account or cloud service once tools are installed.
14. About/README/installer notices clearly state ScrDeck is unofficial and upstream functionality belongs to scrcpy/ADB.
15. The installer, portable build, checksums, and third-party notices are present and validated.

---

# 22. Engineering execution contract

This section is a mandatory instruction set for any coding agent or development team implementing ScrDeck.

## 22.1 Build the real application, not a concept mockup

- Deliver a runnable Tauri desktop application, not a static website pretending to be desktop software.
- Build the UI and backend in small vertical slices that can be tested end-to-end.
- Every visible primary control must perform its advertised action or be clearly marked as planned/disabled with a reason.
- Never use fake device data in production mode. Mock data may exist only in an explicit development/demo mode and must be labelled.
- Do not show success before the backend confirms the operation.
- Do not simulate a connected device because no hardware is available during development.

## 22.2 Implementation order

**Milestone 0 — Repository and build foundation**

- Initialize Tauri 2 + React + TypeScript + Vite.
- Set up code formatting, linting, typed IPC, and test runners.
- Establish color/type/spacing tokens and component foundations.
- Create About/Disclaimer and navigation shell.
- Verify the app builds and launches on Windows before adding features.

**Milestone 1 — Toolchain and environment**

- Implement tool path detection, version check, architecture check, and capability registry.
- Add official dependency download/extraction with validation and rollback.
- Display tool versions and source paths.
- Add tests using captured fixture outputs.

**Milestone 2 — ADB device service**

- Implement server status/initialization, device enumeration, parsing, state machine, refresh, and event/poll updates.
- Implement multiple-device selection.
- Implement setup instructions and authorization handling.
- Test USB states using fixtures and real devices.

**Milestone 3 — Driver assistant and wireless flow**

- Implement careful no-device diagnosis and verified driver-link catalogue.
- Implement wireless pairing/connect only with secure transient pairing-code handling.
- Never auto-install driver packages.

**Milestone 4 — Mirroring and command engine**

- Implement structured settings model, capability mapping, validation, argv generator, preview, custom args parser, and conflicts.
- Launch scrcpy and track child process.
- Implement stop/session lifecycle and error stream.
- Prove preview == executed argv using tests.

**Milestone 5 — Presets and logs**

- Implement versioned preset storage and migrations.
- Implement logs, diagnostics, redaction, export, and error explanations.

**Milestone 6 — Polish and packaging**

- Complete UX states, keyboard navigation, scaling, dark/light theme, and motion.
- Build installer and portable package.
- Add notices, README, release docs, checksums, and clean-machine test.

## 22.3 Rules against low-quality AI-generated UI

- No arbitrary dashboard widgets.
- No placeholder statistics.
- No generic gradients or decorative neon.
- No inconsistent padding/radius/type scales.
- No duplicate navigation patterns.
- No “Lorem ipsum” or fake device examples in the shipped app.
- No invented technical values, supported flags, or Android capabilities.
- No excessive explanatory copy where a direct label and contextual help are clearer.
- All screens must have empty, loading, success, warning, and error states where relevant.
- Use real information architecture, not a collection of disconnected pages.
- Before implementing a screen, identify its user goal, data dependency, primary action, failure state, and keyboard behavior.

## 22.4 Code quality requirements

- Use strict TypeScript.
- Use Rust types and enums for device/session states.
- Keep command execution in a dedicated backend service.
- Avoid giant components and monolithic command handlers.
- Add unit tests to parsers and builders before wiring complex UI.
- Handle errors with typed categories and user-safe messages.
- Keep secrets and pairing codes out of logs.
- Document any deviation from this spec in an ADR or change note.

## 22.5 Definition of a completed implementation task

A task is not complete merely because the code compiles. It is complete when:

- The user-visible behavior works.
- The relevant empty/loading/success/error states exist.
- The feature has appropriate automated tests.
- The feature does not violate process, privacy, security, or licensing requirements.
- The UI follows the ScrDeck design system.
- Documentation is updated if behavior or configuration changed.

---

# Appendix A. Initial preset catalogue

The following profiles are starting ideas. Their settings must be validated against the installed scrcpy version and must not be represented as universal performance guarantees.

| Preset | Intended use | Starting behavior |
|---|---|---|
| Balanced | Daily device use | Moderate max size, up to 60 FPS where supported, standard codec, audio on |
| Low Latency | Responsive control | Reduce unnecessary resolution/encoding overhead; expose audio toggle |
| High Quality | Demonstration and visual inspection | Higher target size/quality, stable codec, device/host capability warning |
| Recording | Screen capture | Recording enabled, explicit output path, overwrite confirmation |
| Presentation | Demonstration on a larger screen | Larger window/fullscreen option, stay-awake, readable scale |
| Battery Conscious | Longer phone session | Lower capture load where supported; avoid promising fixed battery savings |

Built-in presets are templates, not benchmarks. Users can inspect every setting and save modifications as a user preset.

---

# Appendix B. Example configuration schema

The following JSON is illustrative and should be validated against the final Rust/TypeScript schema. The preset format must include a schema version and must not store pairing secrets, executable paths, or shell scripts.

```json
{
  "schemaVersion": 1,
  "id": "balanced-default",
  "name": "Balanced",
  "description": "Everyday mirroring profile.",
  "settings": {
    "maxSize": 1920,
    "maxFps": 60,
    "videoCodec": "h264",
    "audioEnabled": true,
    "fullscreen": false,
    "alwaysOnTop": false,
    "stayAwake": true,
    "turnScreenOff": false,
    "controlEnabled": true,
    "recordingEnabled": false,
    "recordingPath": null
  },
  "customArgs": [],
  "compatibilityNotes": []
}
```

---

# Appendix C. Manual smoke-test checklist

## Fresh setup

- [ ] Install on a clean Windows 11 x64 test machine.
- [ ] Launch without existing ADB or scrcpy.
- [ ] Verify the tool download source and integrity status are displayed.
- [ ] Verify the app does not modify PATH or unrelated SDK directories.
- [ ] Complete USB debugging instructions.
- [ ] Test `unauthorized` and approve RSA prompt.
- [ ] Verify successful device selection.

## Existing tools

- [ ] Point to an existing scrcpy folder.
- [ ] Point to an existing Platform-Tools folder.
- [ ] Confirm version and selected executable paths.
- [ ] Simulate conflicting ADB versions and verify confirmation flow.

## Device and mirroring

- [ ] Connect one device.
- [ ] Connect two devices and select each explicitly.
- [ ] Change every Basic Mode control and verify the generated command.
- [ ] Add a custom argument and inspect parsed argv.
- [ ] Start a real mirroring session.
- [ ] Stop the selected session.
- [ ] Verify unrelated scrcpy/ADB processes remain untouched.
- [ ] Unplug device during mirroring and inspect error/recovery behavior.

## Presets and logs

- [ ] Create/edit/duplicate/delete a preset.
- [ ] Import malformed JSON and confirm no corruption.
- [ ] Export and re-import a valid preset.
- [ ] Trigger a known error and verify explanation plus raw output.
- [ ] Export diagnostics and verify identifiers/secrets are redacted.

## Accessibility and release

- [ ] Navigate all primary flows using keyboard only.
- [ ] Test dark/light/system theme.
- [ ] Test 100%, 125%, 150%, and 200% Windows scaling.
- [ ] Test installer and portable package on clean user profile.
- [ ] Verify third-party notices and disclaimer.
- [ ] Verify checksums for release files.

---

# Appendix D. Official references and design inspiration

**Research snapshot date: 18 November 2026.** Verify every source again when preparing a release because upstream software, binaries, and documentation can change.

### Upstream tools and Android setup

- **[R1] Genymobile scrcpy — official repository, feature overview, prerequisites, usage, and license.** https://github.com/Genymobile/scrcpy
- **[R2] Genymobile scrcpy — official Windows installation and usage documentation.** https://github.com/Genymobile/scrcpy/blob/master/doc/windows.md
- **[R3] Android SDK Platform-Tools release notes.** https://developer.android.com/tools/releases/platform-tools
- **[R4] Android Debug Bridge documentation.** https://developer.android.com/tools/adb
- **[R5] Android Debug Bridge — wireless debugging and pairing-code flow.** https://developer.android.com/tools/adb#wireless
- **[R6] Android Developers — install OEM USB drivers.** https://developer.android.com/studio/run/oem-usb
- **[R7] Microsoft Lifecycle — Windows 10 end of support.** https://learn.microsoft.com/en-us/lifecycle/announcements/windows-10-end-of-support

### Desktop framework and Windows design

- **[R8] Tauri 2 — Windows installer and WebView2 configuration.** https://v2.tauri.app/distribute/windows-installer/
- **[R9] Microsoft — Windows 11 design principles.** https://learn.microsoft.com/en-us/windows/apps/design/design-principles
- **[R10] Microsoft — Windows app design guidelines.** https://learn.microsoft.com/en-us/windows/apps/design/guidelines-overview
- **[R11] Tauri 2 — embedding external binaries / sidecars.** https://v2.tauri.app/develop/sidecar/
- **[R12] Tauri 2 — Shell plugin permissions and process interface.** https://v2.tauri.app/plugin/shell/

### Human-designed product references (interaction inspiration only)

- **LocalSend:** https://localsend.org/ — device discovery and focused transfer flow.
- **KDE Connect:** https://kdeconnect.kde.org/ — paired-device mental model and device trust.
- **Android Studio:** https://developer.android.com/studio — structured technical information.
- **Linear:** https://linear.app/ — restrained hierarchy, density, and spacing.

---

**End of specification.**  
**ScrDeck — Your Android. Your Workspace.**
