# Security Policy

## Supported Versions

The project is currently in alpha. Security fixes are handled on the latest `main` branch.

## Reporting A Vulnerability

Please do not open a public issue for vulnerabilities involving command execution, clipboard access, global hotkeys, model loading, or transcript disclosure.

Use GitHub private vulnerability reporting if it is enabled for the repository. If it is not enabled yet, contact the maintainer privately through the contact method listed on the GitHub profile.

When reporting, include:

- A clear description of the issue
- Steps to reproduce
- Affected commit or release
- Expected impact
- Any relevant logs with secrets removed

## Security Notes

- The app uses local microphone input.
- The app uses clipboard and keyboard simulation for delivery workflows.
- The app launches a PowerShell overlay script during recording and processing.
- The app is intended to process audio locally and does not intentionally upload recordings or transcripts.
- Model files are supplied by users and are not bundled with this repository.
