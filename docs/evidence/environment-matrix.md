# Terminal Environment Validation Matrix

| Environment | Status | Evidence |
| --- | --- | --- |
| PowerShell host (current session) | Partial | Rust build/test pass; native bash command alias missing, but explicit Git Bash executable available. |
| WSL executable presence | Partial | wsl.exe is present but no installed distro in current host output, so WSL runtime validation was not completed. |
| Git Bash/MSYS | Pass (smoke) | Launcher syntax validated and smoke-start run executed through C:/Program Files/Git/bin/bash.exe; game reached Press any key to start. |
| Linux shell | Not tested | Requires execution in Linux host or configured WSL distribution. |

## Known Gap

Launcher syntax and smoke execution are validated in Git Bash; cross-environment parity still requires Linux/WSL runtime checks.
