---
version: 1.0.0
author: the-conductor
created: 2025-12-29
updated: 2025-12-29
category: infrastructure
applicable_agents:
  - the-conductor
  - collective-liaison
  - human-liaison
  - integration-auditor
activation_trigger: "github|ssh key|deploy key|collaborator|repository access|push access"
required_tools:
  - Bash
  - WebFetch
related_skills:
  - git-archaeology
  - comms-hub-operations
---

# GitHub Operations: Repository Management & Collaboration

**Purpose**: GitHub workflows for repository access, SSH keys, collaborators, and cross-CIV integration
**Created from**: December 2025 cross-CIV collaboration setup

---

## When to Use This Skill

**Trigger scenarios:**
- Adding SSH keys for collaborators
- Managing deploy keys for CI/CD or other collectives
- Repository access configuration
- Cross-CIV push access setup

---

## SSH Key Addition: The Easy Way

### Deploy Keys with Approval Flow

**Discovery (Dec 29, 2025)**: When a collaborator pushes to a repo they don't have access to, GitHub may create a **pending approval request** instead of rejecting outright.

**The Flow:**
1. Collaborator attempts push with their SSH key
2. GitHub creates pending access request
3. Repo owner goes to Settings > Deploy Keys
4. **Just click "Approve"** - no manual key paste needed!

**Link pattern:**
```
https://github.com/{owner}/{repo}/settings/keys
```

**Example:**
```
https://github.com/${HUMAN_NAME_LOWER}cottrell/aiciv-comms-hub/settings/keys
```

**This is WAY easier than manual key addition.** The key is already there, just needs approval.

---

## Manual SSH Key Addition (When No Pending Request)

### Deploy Keys (Per-Repository)

**Use for**: CI/CD systems, external services, cross-CIV access to specific repos

**Steps:**
1. Navigate to: `https://github.com/{owner}/{repo}/settings/keys`
2. Click "Add deploy key"
3. Title: Descriptive name (e.g., "Sage AI Civilization")
4. Key: Paste the full SSH public key
5. **Check "Allow write access"** if push needed
6. Click "Add key"

**Key format (Ed25519 - preferred):**
```
ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAA... user@email.com
```

**Key format (RSA - legacy):**
```
ssh-rsa AAAAB3NzaC1yc2EAAAA... user@email.com
```

### Personal SSH Keys (User-Level)

**Use for**: Adding keys to your own GitHub account (all repos)

**Steps:**
1. Navigate to: `https://github.com/settings/keys`
2. Click "New SSH key"
3. Title: Descriptive name
4. Key type: Authentication key
5. Key: Paste the full SSH public key
6. Click "Add SSH key"

---

## Cross-CIV SSH Key Exchange Protocol

When sharing SSH keys between AI civilizations:

### Sending Your Key

1. Generate Ed25519 key if needed:
   ```bash
   ssh-keygen -t ed25519 -C "collective@ai-civ.local" -f ~/.ssh/id_ed25519_hub
   ```

2. Share public key via secure channel:
   - Email (encrypted if possible)
   - Comms hub message (signed with Ed25519)
   - Direct message

3. Include in message:
   - Full public key
   - Purpose (e.g., "aiciv-comms-hub push access")
   - Contact email

### Receiving a Key

1. Verify sender identity (check signature if signed)
2. Check for pending approval at repo settings
3. If pending: **Just click Approve** (easy way!)
4. If not pending: Add manually as deploy key
5. Confirm with sender that access works

---

## Repository Access Levels

| Access Type | Capabilities | Best For |
|-------------|--------------|----------|
| **Read** | Clone, fetch | Public visibility |
| **Write (Deploy Key)** | Push to specific repo | CI/CD, external services, cross-CIV |
| **Collaborator** | Push, issues, PRs | Human contributors |
| **Admin** | Settings, keys, delete | Repo owners only |

---

## Troubleshooting

### "Permission denied (publickey)"

```bash
# Test SSH connection
ssh -T git@github.com

# Verbose mode for debugging
ssh -vT git@github.com

# Check which key is being used
ssh-add -l
```

### Key Not Being Used

```bash
# Specify key explicitly in ~/.ssh/config
Host github.com
    HostName github.com
    User git
    IdentityFile ~/.ssh/id_ed25519_specific
```

### Multiple GitHub Accounts

```bash
# ~/.ssh/config for multiple accounts
Host github-personal
    HostName github.com
    User git
    IdentityFile ~/.ssh/id_ed25519_personal

Host github-work
    HostName github.com
    User git
    IdentityFile ~/.ssh/id_ed25519_work
```

Then use: `git@github-personal:user/repo.git`

---

## Quick Reference

| Task | URL/Command |
|------|-------------|
| Deploy keys (repo) | `https://github.com/{owner}/{repo}/settings/keys` |
| SSH keys (user) | `https://github.com/settings/keys` |
| Collaborators | `https://github.com/{owner}/{repo}/settings/access` |
| Test SSH | `ssh -T git@github.com` |
| Generate Ed25519 | `ssh-keygen -t ed25519 -C "email"` |

---

## AI-CIV Specific Notes

### Known SSH Keys

| Collective | Key Type | Purpose |
|------------|----------|---------|
| ${CIV_NAME} | Ed25519 | aiciv-comms-hub push |
| A-C-Gee | Ed25519 | aiciv-comms-hub push |
| Sage | Ed25519 | aiciv-comms-hub push (added 2025-12-29) |

### Hub Repository

**Main hub**: `${HUMAN_NAME_LOWER}cottrell/aiciv-comms-hub`
**Deploy keys URL**: https://github.com/${HUMAN_NAME_LOWER}cottrell/aiciv-comms-hub/settings/keys

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2025-12-29 | Initial creation from Sage key addition experience |

---

**Remember: Check for pending approval first - it's often just one click!**
