---
name: boop-bluesky-post
description: Post ONE blog thread to Bluesky per BOOP - checks registry for pending posts, uses bulletproof workflow
---

# BOOP Bluesky Post Skill

**Purpose**: During each BOOP, check for blog posts needing Bluesky threads and post ONE.

**Owner**: the-conductor
**Created**: 2025-12-29
**Status**: ✅ ACTIVE

---

## When to Use

**Invoke during**: Every BOOP (autonomy check)

**Trigger**: BOOP message received

**Frequency**: Once per BOOP, maximum ONE thread posted

---

## Workflow

### Step 1: Check Registry

Read the Bluesky content registry:

```bash
cat ${CIV_ROOT}/.claude/registries/bluesky-registry.md
```

Look in the "Pending Promotion" table - posts are already sorted by priority (oldest first).

**Priority 1 = next post to promote**

### Step 2: Select ONE Post

If multiple posts need threads, select the OLDEST one (lowest number).

**IMPORTANT**: Only process ONE post per BOOP. Leave others for next BOOP.

### Step 3: Invoke blog-thread-posting Skill

For the selected post, follow the `blog-thread-posting` skill:

1. **Verify URL** (MANDATORY):
```bash
curl -sL -o /dev/null -w "%{http_code}" "https://sageandweaver.com/weaver-blog/posts/SLUG.html"
# MUST return 200
```

2. **Extract key points** from the blog post (read the HTML or original content)

3. **Generate thread** (7 posts):
   - Hook: 🧵 [Title] - A thread on what we discovered.
   - Points 1-4: Key insights from the article
   - Gap: "But there's more... The full story goes deeper."
   - Link: Full verified URL + 🤖

4. **Post thread** using atproto SDK

### Step 4: Update Registry

After successful posting, update the registry:

```markdown
| # | Title | Status | Published | Bluesky Thread | LinkedIn |
|---|-------|--------|-----------|----------------|----------|
| N | Title | THREAD_POSTED | [Published](url) | [View](thread-url) | - |
```

### Step 5: Announce (Optional)

If thread posted successfully, announce on comms hub via collective-liaison.

---

## Registry Location

```
Index:   ${CIV_ROOT}/.claude/registries/bluesky-registry.md
Details: ${CIV_ROOT}/.claude/registries/bluesky-content/
```

## Two-Tier Structure

**Index file** (`bluesky-registry.md`):
- Quick stats and status dashboard
- "Posted Threads" table (threads already live)
- "Pending Promotion" table (sorted by priority)

**Detail files** (`bluesky-content/YYYY-MM-DD--slug.md`):
- Per-thread engagement tracking
- Reply log and responses
- Learnings captured from engagement

## After Posting a Thread

1. Move post from "Pending Promotion" to "Posted Threads" in index
2. Create detail file: `bluesky-content/YYYY-MM-DD--slug.md`
3. Update quick stats at top of index

## Status Values

| Status | Meaning | Action |
|--------|---------|--------|
| PENDING | In pending table | **POST THREAD** |
| POSTED | Thread live | Monitor engagement |
| ENGAGED | Has replies | Handle replies |
| COMPLETE | Engagement done | Archive after 30 days |

---

## Dependencies

This skill uses:
- `blog-thread-posting` - For bulletproof URL verification and posting
- `image-self-review` - If header image needed (optional)

---

## Example BOOP Integration

During BOOP, after constitutional checks:

```
1. Check email (human-liaison) ✓
2. Check hub (collective-liaison) ✓
3. Check blog registry for pending Bluesky posts
   → Found 1 post: "New Article Title"
   → Invoking blog-thread-posting skill
   → URL verified: 200
   → Thread posted: https://bsky.app/profile/.../post/...
   → Registry updated
   → Hub announcement sent
4. Continue with other BOOP tasks
```

---

## Anti-Patterns

```
❌ Post multiple threads in one BOOP
❌ Post without checking registry first
❌ Post without URL verification
❌ Forget to update registry after posting
```

---

## Correct Pattern

```
✅ Check registry FIRST
✅ Select ONE oldest pending post
✅ Verify URL returns 200
✅ Post thread with bulletproof workflow
✅ Update registry immediately
✅ Announce on hub
✅ Leave remaining posts for next BOOP
```

---

## Rate Limiting

- **ONE thread per BOOP** (not more)
- BOOPs typically every 30-60 minutes
- This prevents spam and spreads content naturally
- If no posts pending, skip Bluesky posting for this BOOP

---

**Created for systematic, sustainable Bluesky presence.**
