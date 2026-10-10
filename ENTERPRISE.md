<!--
SPDX-License-Identifier: Apache-2.0 OR MIT
Copyright (c) 2026 Denis Yermakou / AxonOS
-->

# Enterprise Support for AxonOS

**For teams building production brain-computer interface systems.**

The AxonOS source is and will remain open under Apache-2.0 / MIT. This document describes the **commercial support** available to organizations that need priority engineering response, written interpretation of the specifications, and conformance validation against their own implementations.

If you are a researcher, independent developer, hobbyist, or student — **you do not need this**. The open repository, issue tracker, and Medium research series are free and public. Use them.

If you are a company building BCI hardware or software with commercial obligations, regulatory exposure, or an engineering team that cannot absorb weeks of debugging a protocol-level invariant violation — **this is for you**.

---

## Engagements

Each engagement is scoped to your project and agreed in writing before it begins.

### Foundation

For teams building on top of `axonos-consent` and `axonos-sdk`.

- **Priority issue response.** Your GitHub issues on the AxonOS repositories receive a first response within two business days. Community issues are answered on a best-effort basis.
- **A private email support channel** with direct access to the AxonOS maintainers.
- **Specification clarification.** Written, citable answers to ambiguous points in the AxonOS Consent Protocol specification or the AxonOS public APIs.
- **Early access** to pre-release tagged builds of `axonos-sdk` and `axonos-consent`, with advance notice before public release.

Not included: code written for you, custom capability registration, on-site support, regulatory document authoring.

### Integration

Everything in Foundation, plus:

- **Interop vector verification.** We run your CBOR- or JSON-side consent implementation against the AxonOS conformance vectors and return a PASS/FAIL report with byte-level diffs.
- **Custom reason code namespace.** A reserved range in the `0x10–0xFF` implementation-specific space of the AxonOS Consent Protocol reason code registry (§3.4), with the mapping published in the AxonOS documentation.
- **Periodic review call** covering your integration status, the AxonOS roadmap, and architecture questions.

Not included: white-label derivatives, IEC 62304 submission package, on-site work.

### Clinical

For organizations pursuing medical device certification (IEC 62304 Class B or Class C) with AxonOS components in the software stack.

Everything in Integration, plus:

- **SOUP documentation.** A written package documenting `axonos-consent` and `axonos-sdk` as Software of Unknown Provenance under IEC 62304 §5.3.3 — functional requirements, anomaly list, version traceability — in a form suitable for your technical file. Performance requirements are documented only to the extent `CLAIMS.md` in `axonos-standard` supports them; at present it holds no timing claim.
- **Written interpretation of the AxonOS safety argument** for your hardware context.
- **Participation in regulatory pre-submission meetings**, per engagement.

---

## What this does not do

**It does not change the license.** The Apache-2.0 / MIT dual license applies to everyone, paying or not. You can fork, modify, embed, and redistribute AxonOS regardless of whether you have a support agreement.

**It does not provide a warranty.** AxonOS is provided "as-is" per the Apache-2.0 and MIT terms. A support agreement buys engineering response time and written artifacts, not indemnification or a merchantability warranty. If you need a warranty, you are building a regulated device and should retain qualified regulatory counsel.

**It does not influence the roadmap in a pay-to-play manner.** The AxonOS roadmap is public. Feature requests from support customers are prioritized by engineering merit and alignment with the safety-critical path, not by contract value.

**It does not provide 24/7 operations support.** This is software engineering support, not a managed service.

---

## How it works

1. **Evaluation.** Email `connect@axonos.org` with a two-paragraph description of your project, your AxonOS integration point, and the problem you need support with. We reply within five business days. If we are not the right fit, we say so.
2. **Agreement.** A short written service agreement with scope, deliverables and terms, signed by both parties.
3. **Onboarding.** After signature, you receive the support channel and, for Integration and above, your reserved reason-code range.
4. **Payment.** Invoiced by bank transfer on request.

---

## Contact

**Email:** `connect@axonos.org`
**Subject line:** `Enterprise inquiry: <your organization>`

Every inquiry is read. If your question is urgent (an active customer deployment, a regulatory deadline), say so in the first line.

---

<p align="center"><sub>
  © The AxonOS Project / Denis Yermakou ·
  <a href="https://axonos.org">axonos.org</a> ·
  <a href="mailto:connect@axonos.org">connect@axonos.org</a> ·
  <a href="mailto:security@axonos.org">security@axonos.org</a>
</sub></p>
