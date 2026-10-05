---
name: Bug report
about: Create a report to help us improve
title: ''
labels: bug
assignees: ''

---

**BEFORE ANYTHING, PLEASE ENSURE YOU ARE TESTING THE LATEST GIT VERSION.**
Many reported bugs are fixed on `main` before a release is cut. When possible,
reproduce with a debug build (`cargo build`), which logs more information than
the release profile.

**Describe the bug**
A clear and concise description of what the bug is. If it helps, paste the log
from a debug build of `xwww-daemon` here.

**To Reproduce**
Steps to reproduce the behavior. Include the exact `xwww` command line used.

**Expected behavior**
What you expected to happen instead.

**Environment**

- `xwww --version` output:
- Build features used (for example `video-static,scene`, or
  `--no-default-features`):
- Compositor and version (for example Hyprland 0.45, Sway 1.10, niri 25.05):
- Monitor setup (resolution, scale factor, number of outputs):
- Ways the daemon was started (manual, systemd user unit, script):

**Logs**

<details>
<summary>Daemon log</summary>

```
paste the daemon output here
```

</details>

<details>
<summary>Client log</summary>

```
paste the client output here, if relevant
```

</details>

**Screenshots or recordings**
If applicable, add screenshots, GIFs or recordings to help explain the problem.

**Additional context**
Add any other context about the problem here. For rendering problems, attach
the image or video file that triggers it when you can.
