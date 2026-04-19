# Security Policy

## Scope

AxonOS is designed for safety-critical brain-computer interface applications. Security vulnerabilities in this crate can contribute to patient-safety hazards in downstream systems. We take them seriously.

This policy applies to:

- The `axonos-sdk` crate (this repository).
- The related `axonos-consent` crate at https://github.com/AxonOS-org/axonos-consent.
- Published documentation and examples.

## Reporting

**Do not file public GitHub issues for security reports.**

Email: `axonosorg@gmail.com`
Subject line: `Security: axonos-sdk <one-line summary>`

Please include:

1. The affected crate and version (`cargo pkgid` output).
2. The feature flags you were using.
3. A minimal reproducer or a written description.
4. Your assessment of impact: does this affect confidentiality of intent events, integrity of the consent state machine, availability of the stream, or kernel-application isolation?
5. Whether you have a fix in mind.

You will receive an acknowledgement within 3 business days (Singapore time). If you have not heard back within 5 business days, please email again — inboxes sometimes eat mail.

## What we commit to

- **Acknowledgement** within 3 business days.
- **Triage** within 10 business days: either a reproduction confirming the issue, or a written explanation of why we assess it as not a vulnerability.
- **Coordinated disclosure window** of up to 90 days from acknowledgement, during which we ask you not to publish. For issues affecting downstream medical-device integrations, we may request a longer window (up to 180 days) with the reporter's agreement.
- **Credit** in the published advisory, unless you prefer anonymity.
- **A fix** shipped in a patch release, with the advisory cross-referenced in `CHANGELOG.md`.

## What we cannot commit to

- A bug bounty. AxonOS is a small, self-funded project. We value reports and reporters but cannot pay cash rewards at this stage. Enterprise support customers receive formal SLAs; see `ENTERPRISE.md`.
- Support for unmaintained versions. Security fixes land on the latest minor; previous minors receive fixes at our discretion for 6 months after their last release.

## Threat model (summary)

The AxonOS SDK sits at the application side of a trust boundary. The following properties are intended:

- An application compromise cannot extract raw neural signals (there is no API for them).
- An application compromise cannot forge intent observations to other applications (each subscription is isolated; observations are kernel-attested).
- An application compromise cannot prevent a user consent-withdraw from reaching the hardware interlock (the consent frame is emitted by a separate kernel task, not by the SDK process).

Reports describing violations of these properties are treated as critical.

## Out of scope

- Vulnerabilities in third-party crates we depend on. Please report those upstream; we will coordinate when you do.
- Social-engineering of AxonOS maintainers.
- Issues requiring physical access to the victim's device (this is by design: AxonOS's hardware interlock assumes physical custody is part of the trusted computing base).

---

`axonos.org · medium.com/@AxonOS · axonosorg@gmail.com`
