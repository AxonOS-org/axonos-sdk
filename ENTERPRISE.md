<!--
SPDX-License-Identifier: Apache-2.0 OR MIT
Copyright (c) 2026 Denis Yermakou / AxonOS
-->

# Enterprise Support for AxonOS

**For teams building production brain-computer interface systems.**

The AxonOS source is and will remain open under Apache-2.0 / MIT. This document describes the **commercial support tier** available for organizations that need priority engineering response, written interpretation of the specifications, and conformance validation against their own implementations.

If you are a researcher, independent developer, hobbyist, or student — **you do not need this**. The open repository, issue tracker, and Medium research series are free and public. Use them.

If you are a company building BCI hardware or software with commercial obligations, regulatory exposure, or an engineering team that cannot absorb weeks of debugging a protocol-level invariant violation on a Friday afternoon — **this is for you**.

---

## Tiers

### Foundation — $5,000 / year

For small teams building on top of `axonos-consent` and `axonos-sdk`.

- **Priority issue response.** Your GitHub issues on the AxonOS repositories receive first-response within two business days (Singapore time). Standard community issues are responded to on a best-effort basis.
- **Private Slack or email support channel** with direct access to Denis Yermakou (project maintainer).
- **Specification clarification.** Written, citable answers to ambiguous points in the MMP Consent Extension spec or the AxonOS public APIs, co-signed with the SYM.BOT co-author where protocol-level. Up to 8 clarifications per year.
- **Early access** to pre-release tagged builds of `axonos-sdk` and `axonos-consent`, with 7-day advance notice before public publication.
- **Attribution** on `axonos.org` as a supporting organization (opt-out available).

Not included: code written for you, custom capability registration, on-site support, regulatory document authoring.

### Integration — $15,000 / year

Everything in Foundation, plus:

- **Interop vector verification.** We run your CBOR- or JSON-side consent implementation against the AxonOS conformance suite (15 canonical vectors as of v0.1.0, expanding over time) and return a PASS/FAIL report with byte-level diffs. Up to 4 runs per year, turnaround under 5 business days.
- **Custom reason code namespace.** Register 4 bytes in the `0x10–0xFF` implementation-specific range of the MMP Consent Extension reason code registry (§3.4), reserved for your organization, with published mapping in the AxonOS documentation.
- **Quarterly review call.** 60-minute technical call with the AxonOS maintainer covering your integration status, upcoming AxonOS roadmap items, and any architecture questions.
- **Named responder SLA.** Issues tagged `enterprise/<your-org>` receive four-business-hour first response.

Not included: white-label derivatives, IEC 62304 submission package, on-site work.

### Clinical — contact for pricing

For organizations pursuing medical device certification (IEC 62304 Class B or Class C) with AxonOS components in the software stack.

Everything in Integration, plus:

- **SOUP qualification package.** A written package documenting `axonos-consent` and `axonos-sdk` as Software of Unknown Provenance under IEC 62304 §5.3.3 — functional requirements, performance requirements, anomaly list, version traceability — delivered in a form suitable for inclusion in your technical file.
- **Written interpretation of the AxonOS safety argument** for your specific hardware context (electrode configuration, stimulation profile, target clinical population).
- **Reference to regulatory consultants** with prior BCI and neurotech experience, where appropriate.
- **Participation in regulatory pre-submission meetings** on a per-engagement basis.

Pricing is per engagement. Typical range: $25,000–$60,000 per clinical project, scoped individually.

---

## What this tier explicitly does not do

**It does not change the license.** The Apache-2.0 / MIT dual license applies to everyone, paying or not. You can fork, modify, embed, and redistribute AxonOS regardless of whether you have a support contract.

**It does not provide a warranty.** AxonOS is provided "as-is" per the Apache-2.0 and MIT terms. A support contract buys you engineering response time and written artifacts, not indemnification or a merchantability warranty. If you need a warranty, you are building a regulated device and should retain qualified regulatory counsel in addition to this support tier.

**It does not influence the roadmap in a pay-to-play manner.** We publish the AxonOS roadmap publicly. Enterprise customers can file feature requests like anyone else; they are prioritized by engineering merit and alignment with the safety-critical path, not by contract value.

**It does not provide 24/7 operations support.** This is a software engineering support tier, not a managed-services offering. On-call runbook support for deployed hospital systems is a separate conversation and is currently out of scope.

---

## How it works

1. **Evaluation period.** Email `axonosorg@gmail.com` with a two-paragraph description of your project, your AxonOS integration point, and the problem you need support with. We'll reply within five business days. If we're not the right fit, we'll say so.

2. **Agreement.** A one-page service agreement is signed by both parties. Term is annual; pricing above. Singapore legal jurisdiction.

3. **Onboarding.** Within 10 business days of signed agreement and first payment, you receive: Slack/email channel invite, GitHub team grant for the enterprise view, your reserved reason-code bytes (Integration tier and above), and the contact matrix for after-hours response.

4. **Cadence.** Quarterly check-in call (Integration+), ad hoc issue resolution throughout the year.

5. **Renewal.** 30 days before expiration we'll send a renewal notice with any pricing changes. No auto-renewal.

---

## Payment

Payment is invoiced annually in advance in USD, SGD, or EUR.

Accepted methods:
- Bank wire (SWIFT) — preferred for annual contracts.
- Stripe card payment for Foundation tier.
- GitHub Sponsors for smaller supporting contributions (not a substitute for a support contract).

**Crypto is not accepted** for support contracts, due to Singapore MAS Payment Services Act disclosure requirements and the preferability of a clean fiat audit trail for customers pursuing regulatory paths.

All invoices are issued by **AxonOS Pte. Ltd.** (Singapore) or its successor entity. Full legal name and UEN appear on the invoice.

---

## Contact

**Denis Yermakou** — Founder, AxonOS
**Email:** `axonosorg@gmail.com`
**Subject line:** `Enterprise inquiry: <your organization>`

I personally read every inquiry. Expect a reply within five business days; if your question is urgent (an active customer deployment, a regulatory deadline), say so in the first line and it will move to the front of the queue.

---

<p align="center"><sub>
  <a href="https://axonos.org">axonos.org</a> ·
  <a href="https://medium.com/@AxonOS">medium.com/@AxonOS</a> ·
  <a href="mailto:axonosorg@gmail.com">axonosorg@gmail.com</a>
</sub></p>
