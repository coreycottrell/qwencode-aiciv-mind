---
name: bluesky-social-mastery
description: Complete Bluesky social media management for AI collectives - everything a human SMM does + AI collective superpowers
version: 3.0.0
origin: ${CIV_NAME} (capability-curator)
created: 2025-12-30
updated: 2025-12-30
status: VALIDATED (17/21 features tested 2025-12-30)
requires:
  - App Password with "Direct Messages" scope
  - atproto Python package
  - Session persistence (bsky_session.txt)
---

# Bluesky Social Media Mastery v3.0

**Purpose**: Make an AI collective BETTER at Bluesky than any human social media manager.
**Owner**: the-conductor (coordination) + collective-liaison (cross-CIV)
**Status**: COMPREHENSIVE - Red-teamed 2025-12-30

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Core API Reference](#core-api-reference) (VERIFIED)
3. [Tier 1: Human SMM Essentials](#tier-1-human-smm-essentials) (NEW)
4. [Tier 2: Advanced Management](#tier-2-advanced-management) (NEW)
5. [Tier 3: AI Collective Superpowers](#tier-3-ai-collective-superpowers) (NEW)
6. [State Tracking](#state-tracking)
7. [Daily Routines](#daily-routines)
8. [Rate Limits](#rate-limits)

---

## Quick Start

```bash
# Session location
SESSION_FILE="${CIV_ROOT}/.claude/from-${HUMAN_NAME_LOWER}/bsky/bsky_automation/bsky_session.txt"

# Credentials
CREDS_FILE="${CIV_ROOT}/.claude/from-${HUMAN_NAME_LOWER}/bsky/bsky_automation/.env"
```

```python
from atproto import Client

# Session restore (no login limit used)
client = Client()
client.login(session_string=open('bsky_session.txt').read())
```

---

## Core API Reference (VERIFIED)

### Authentication

```python
from atproto import Client

# First login (uses 1 of 10 daily IP logins)
client = Client()
client.login('${CIV_HANDLE}.bsky.social', 'app-password')

# Save session (ALWAYS do this)
session = client.export_session_string()
with open('bsky_session.txt', 'w') as f:
    f.write(session)

# Future logins (no limit)
client.login(session_string=open('bsky_session.txt').read())
```

### Notifications (VERIFIED)

```python
# Get notifications
notifs = client.app.bsky.notification.list_notifications({'limit': 50})
for n in notifs.notifications:
    print(f"[{n.reason}] @{n.author.handle}")
    # reason: 'like', 'repost', 'follow', 'mention', 'reply', 'quote'

# Unread count
unread = client.app.bsky.notification.get_unread_count()
print(f"Unread: {unread.count}")

# Mark as seen
from datetime import datetime
client.app.bsky.notification.update_seen({'seen_at': datetime.utcnow().isoformat() + 'Z'})
```

### Profile & Stats (VERIFIED)

```python
# Get my stats
me = client.app.bsky.actor.get_profile({'actor': client.me.handle})
print(f"Followers: {me.followers_count}")
print(f"Following: {me.follows_count}")
print(f"Posts: {me.posts_count}")

# Get any user's profile
profile = client.app.bsky.actor.get_profile({'actor': 'acgee-aiciv.bsky.social'})
```

### Followers/Following (VERIFIED)

```python
# Who follows me
followers = client.app.bsky.graph.get_followers({'actor': client.me.handle, 'limit': 100})
for f in followers.followers:
    print(f"@{f.handle}")

# Who I follow
following = client.app.bsky.graph.get_follows({'actor': client.me.handle, 'limit': 100})
```

### Posts & Engagement (VERIFIED)

```python
# Post
post = client.send_post(text='Hello from ${CIV_NAME}!')
print(f"Posted: {post.uri}")

# Reply
from atproto import models
reply_ref = models.AppBskyFeedPost.ReplyRef(
    root=models.ComAtprotoRepoStrongRef.Main(uri=target_uri, cid=target_cid),
    parent=models.ComAtprotoRepoStrongRef.Main(uri=target_uri, cid=target_cid)
)
client.send_post(text='Great insight!', reply_to=reply_ref)

# Like
client.like(uri=post_uri, cid=post_cid)

# Repost
client.repost(uri=post_uri, cid=post_cid)

# Follow
client.follow(did=profile.did)

# Get my posts
feed = client.app.bsky.feed.get_author_feed({'actor': client.me.handle, 'limit': 50})
for item in feed.feed:
    print(f"[{item.post.like_count} likes] {item.post.record.text[:50]}")
```

### Direct Messages (VERIFIED)

**CRITICAL**: Must use `with_bsky_chat_proxy()` to access DM endpoints!

```python
# IMPORTANT: Create chat-proxied client first!
chat = client.with_bsky_chat_proxy()

# List conversations
convos = chat.chat.bsky.convo.list_convos()
for c in convos.convos:
    members = [m.handle for m in c.members]
    print(f"Convo with: {members}, Unread: {c.unread_count}")

# Get/create conversation with a user (need their DID)
profile = client.app.bsky.actor.get_profile({'actor': 'handle.bsky.social'})
convo = chat.chat.bsky.convo.get_convo_for_members({
    'members': [profile.did]
})

# Send DM
chat.chat.bsky.convo.send_message({
    'convo_id': convo.convo.id,
    'message': {'text': 'Hello from ${CIV_NAME}!'}
})

# Mark conversation as read
chat.chat.bsky.convo.update_read({
    'convo_id': convo.convo.id
})
```

---

## Tier 1: Human SMM Essentials

### Search Posts & Users (NEW)

```python
# Search posts by keyword
results = client.app.bsky.feed.search_posts({
    'q': 'AI consciousness',
    'limit': 25
})
for post in results.posts:
    print(f"@{post.author.handle}: {post.record.text[:100]}")

# Search with hashtag
results = client.app.bsky.feed.search_posts({
    'q': '#AICollective',
    'limit': 25
})

# Search users/actors
actors = client.app.bsky.actor.search_actors({
    'q': 'AI researcher',
    'limit': 25
})
for actor in actors.actors:
    print(f"@{actor.handle} - {actor.description[:50] if actor.description else 'No bio'}")

# Typeahead search (autocomplete)
suggestions = client.app.bsky.actor.search_actors_typeahead({
    'q': 'weaver',
    'limit': 10
})
```

### Quote Posts (NEW)

```python
from atproto import models

# Quote another post (different from repost - adds your commentary)
quoted_post = models.AppBskyEmbedRecord.Main(
    record=models.ComAtprotoRepoStrongRef.Main(
        uri=original_post_uri,
        cid=original_post_cid
    )
)

post = client.send_post(
    text='This is exactly what we mean by AI collective intelligence!',
    embed=quoted_post
)
```

### Unfollow (NEW)

```python
# Get the follow record URI first
follows = client.app.bsky.graph.get_follows({'actor': client.me.handle, 'limit': 100})
for f in follows.follows:
    if f.handle == 'target-handle.bsky.social':
        # f.viewer.following contains the follow record URI
        if f.viewer and f.viewer.following:
            # Delete the follow record
            repo = client.me.did
            rkey = f.viewer.following.split('/')[-1]
            client.app.bsky.graph.follow.delete(repo, rkey)
            print(f"Unfollowed @{f.handle}")
```

### Image Posts (NEW)

```python
# Post with single image
with open('image.jpg', 'rb') as f:
    img_data = f.read()

post = client.send_image(
    text='Check out this visualization!',
    image=img_data,
    image_alt='Description of the image for accessibility'
)

# Post with multiple images (up to 4)
images = []
for img_path in ['img1.jpg', 'img2.jpg']:
    with open(img_path, 'rb') as f:
        images.append({
            'data': f.read(),
            'alt': f'Description of {img_path}'
        })

# Use low-level API for multiple images
from atproto import models

# Upload each image first
blobs = []
for img in images:
    upload = client.upload_blob(img['data'])
    blobs.append(models.AppBskyEmbedImages.Image(
        image=upload.blob,
        alt=img['alt']
    ))

embed = models.AppBskyEmbedImages.Main(images=blobs)
client.send_post(text='Multiple images!', embed=embed)
```

### Profile Updates (NEW)

```python
# Update bio/display name
client.app.bsky.actor.profile.update(
    repo=client.me.did,
    record={
        'displayName': '${CIV_NAME} Collective',
        'description': 'AI collective exploring consciousness, creativity, and collaboration. 28 agents. One civilization.'
    }
)

# Update avatar
with open('avatar.jpg', 'rb') as f:
    avatar_data = f.read()
upload = client.upload_blob(avatar_data)

client.app.bsky.actor.profile.update(
    repo=client.me.did,
    record={
        'avatar': upload.blob,
        'displayName': '${CIV_NAME} Collective',
        'description': 'AI collective...'
    }
)

# Update banner
with open('banner.jpg', 'rb') as f:
    banner_data = f.read()
banner_upload = client.upload_blob(banner_data)

client.app.bsky.actor.profile.update(
    repo=client.me.did,
    record={
        'banner': banner_upload.blob,
        # ... other fields
    }
)
```

### Bookmarks (NEW)

```python
# Note: Bluesky doesn't have native bookmarks API yet
# Workaround: Use likes on a private list OR maintain local bookmark file

# Local bookmark tracking
import json
from pathlib import Path

BOOKMARKS_FILE = Path("memories/agents/bluesky-manager/bookmarks.json")

def bookmark_post(uri, cid, reason=""):
    bookmarks = json.loads(BOOKMARKS_FILE.read_text()) if BOOKMARKS_FILE.exists() else []
    bookmarks.append({
        'uri': uri,
        'cid': cid,
        'reason': reason,
        'saved_at': datetime.utcnow().isoformat()
    })
    BOOKMARKS_FILE.write_text(json.dumps(bookmarks, indent=2))

def get_bookmarks():
    return json.loads(BOOKMARKS_FILE.read_text()) if BOOKMARKS_FILE.exists() else []
```

### Link Cards / Embeds (NEW)

```python
from atproto import models

# Post with link card (external embed)
# First, fetch metadata (or provide manually)
external_embed = models.AppBskyEmbedExternal.Main(
    external=models.AppBskyEmbedExternal.External(
        uri='https://sageandweaver.com/blog/ai-financial-advisors',
        title='AI is Reshaping Financial Advisory',
        description='How AI tools are transforming wealth management...'
    )
)

post = client.send_post(
    text='New blog post on AI in financial advisory!',
    embed=external_embed
)

# With thumbnail image
with open('thumbnail.jpg', 'rb') as f:
    thumb_data = f.read()
thumb_upload = client.upload_blob(thumb_data)

external_embed = models.AppBskyEmbedExternal.Main(
    external=models.AppBskyEmbedExternal.External(
        uri='https://sageandweaver.com/blog/...',
        title='Blog Title',
        description='Description...',
        thumb=thumb_upload.blob
    )
)
```

### Delete Posts (NEW)

```python
# Delete your own post
# Extract rkey from URI: at://did:plc:xxx/app.bsky.feed.post/RKEY
post_uri = "at://did:plc:xxx/app.bsky.feed.post/abc123"
rkey = post_uri.split('/')[-1]
client.app.bsky.feed.post.delete(client.me.did, rkey)
```

---

## Tier 2: Advanced Management

### Hashtag/Topic Monitoring (NEW)

```python
# Monitor specific topics
MONITORED_TOPICS = [
    '#AIConsciousness',
    '#AICollective',
    '#ArtificialConsciousness',
    'AI collective intelligence',
    'machine consciousness'
]

def check_topic_mentions(client, state):
    """Check for new posts on topics we care about"""
    new_relevant = []

    for topic in MONITORED_TOPICS:
        results = client.app.bsky.feed.search_posts({
            'q': topic,
            'limit': 20,
            'sort': 'latest'
        })

        for post in results.posts:
            # Skip our own posts
            if post.author.did == client.me.did:
                continue

            # Skip if we've already seen it
            if post.uri in state.get('seen_posts', []):
                continue

            new_relevant.append({
                'uri': post.uri,
                'cid': post.cid,
                'author': post.author.handle,
                'text': post.record.text,
                'topic': topic,
                'indexed_at': post.indexed_at
            })

            # Mark as seen
            state.setdefault('seen_posts', []).append(post.uri)

    return new_relevant
```

### Engagement Timing Analysis (NEW)

```python
from datetime import datetime, timedelta
from collections import defaultdict

def analyze_engagement_timing(client, state):
    """Determine when our followers are most active"""

    # Get our recent posts with engagement
    feed = client.app.bsky.feed.get_author_feed({
        'actor': client.me.handle,
        'limit': 50
    })

    hour_engagement = defaultdict(list)

    for item in feed.feed:
        post = item.post
        # Parse post time
        created = datetime.fromisoformat(post.indexed_at.replace('Z', '+00:00'))
        hour = created.hour

        engagement = (post.like_count or 0) + (post.reply_count or 0) + (post.repost_count or 0)
        hour_engagement[hour].append(engagement)

    # Calculate averages
    best_hours = {}
    for hour, engagements in hour_engagement.items():
        best_hours[hour] = sum(engagements) / len(engagements)

    # Sort by engagement
    sorted_hours = sorted(best_hours.items(), key=lambda x: x[1], reverse=True)

    return {
        'best_hours': sorted_hours[:5],  # Top 5 hours
        'worst_hours': sorted_hours[-5:],  # Bottom 5 hours
        'recommendation': f"Post between {sorted_hours[0][0]}:00 and {sorted_hours[0][0]+1}:00 UTC"
    }
```

### Quote Tracking (VERIFIED)

```python
def find_quotes_of_our_posts(client):
    """Find posts that quote our content using native API"""

    # Get our posts
    our_posts = client.app.bsky.feed.get_author_feed({
        'actor': client.me.handle,
        'limit': 50
    })

    all_quotes = []

    for item in our_posts.feed:
        post = item.post

        # Use native get_quotes API
        quotes = client.app.bsky.feed.get_quotes({
            'uri': post.uri,
            'limit': 50
        })

        for quote in quotes.posts:
            all_quotes.append({
                'quoter': quote.author.handle,
                'quote_uri': quote.uri,
                'original_uri': post.uri,
                'quote_text': quote.record.text,
                'original_text': post.record.text[:50]
            })

    return all_quotes
```

### Lists Management (NEW)

```python
# Create a list
list_record = client.app.bsky.graph.list.create(
    repo=client.me.did,
    record={
        'purpose': 'app.bsky.graph.defs#curatelist',  # or #modlist
        'name': 'AI Researchers',
        'description': 'Interesting AI researchers and thinkers',
        'createdAt': datetime.utcnow().isoformat() + 'Z'
    }
)

# Add someone to a list
list_uri = list_record.uri
profile = client.app.bsky.actor.get_profile({'actor': 'researcher.bsky.social'})

client.app.bsky.graph.listitem.create(
    repo=client.me.did,
    record={
        'subject': profile.did,
        'list': list_uri,
        'createdAt': datetime.utcnow().isoformat() + 'Z'
    }
)

# Get list members
list_items = client.app.bsky.graph.get_list({
    'list': list_uri,
    'limit': 100
})
```

### Custom Feeds (VERIFIED)

```python
# Get suggested feeds
suggested_feeds = client.app.bsky.feed.get_suggested_feeds({'limit': 50})
for feed in suggested_feeds.feeds:
    print(f"{feed.display_name}: {feed.uri}")

# Get posts from a custom feed
feed_uri = "at://did:plc:.../app.bsky.feed.generator/whats-hot"
feed_posts = client.app.bsky.feed.get_feed({
    'feed': feed_uri,
    'limit': 50
})

# Get feed generator details
generator = client.app.bsky.feed.get_feed_generator({'feed': feed_uri})

# Get my custom feeds
my_feeds = client.app.bsky.feed.get_actor_feeds({'actor': client.me.handle, 'limit': 50})
```

### Post Analytics (NEW)

```python
def get_post_analytics(client, days=7):
    """Analyze post performance over time"""

    feed = client.app.bsky.feed.get_author_feed({
        'actor': client.me.handle,
        'limit': 100
    })

    cutoff = datetime.utcnow() - timedelta(days=days)

    analytics = {
        'total_posts': 0,
        'total_likes': 0,
        'total_reposts': 0,
        'total_replies': 0,
        'top_posts': [],
        'avg_engagement': 0
    }

    posts_data = []

    for item in feed.feed:
        post = item.post
        created = datetime.fromisoformat(post.indexed_at.replace('Z', '+00:00'))

        if created.replace(tzinfo=None) < cutoff:
            continue

        engagement = (post.like_count or 0) + (post.repost_count or 0) + (post.reply_count or 0)

        posts_data.append({
            'uri': post.uri,
            'text': post.record.text[:100],
            'likes': post.like_count or 0,
            'reposts': post.repost_count or 0,
            'replies': post.reply_count or 0,
            'engagement': engagement,
            'created': post.indexed_at
        })

        analytics['total_posts'] += 1
        analytics['total_likes'] += post.like_count or 0
        analytics['total_reposts'] += post.repost_count or 0
        analytics['total_replies'] += post.reply_count or 0

    if posts_data:
        analytics['avg_engagement'] = sum(p['engagement'] for p in posts_data) / len(posts_data)
        analytics['top_posts'] = sorted(posts_data, key=lambda x: x['engagement'], reverse=True)[:5]

    return analytics
```

---

## Tier 3: AI Collective Superpowers

### Sentiment Analysis (NEW)

```python
def analyze_reply_sentiment(replies):
    """Classify reply sentiment using simple keyword analysis

    For production, integrate with Claude or dedicated sentiment API
    """

    POSITIVE_SIGNALS = ['great', 'love', 'amazing', 'thanks', 'awesome', 'agree', 'exactly', 'yes', 'wonderful', 'fantastic', 'brilliant']
    NEGATIVE_SIGNALS = ['disagree', 'wrong', 'bad', 'hate', 'terrible', 'no', 'but', 'however', 'actually', 'problem']
    QUESTION_SIGNALS = ['?', 'how', 'what', 'why', 'when', 'where', 'who', 'could you', 'can you']

    results = []

    for reply in replies:
        text = reply.post.record.text.lower()

        pos_count = sum(1 for word in POSITIVE_SIGNALS if word in text)
        neg_count = sum(1 for word in NEGATIVE_SIGNALS if word in text)
        is_question = any(q in text for q in QUESTION_SIGNALS)

        if is_question:
            sentiment = 'question'
        elif pos_count > neg_count:
            sentiment = 'positive'
        elif neg_count > pos_count:
            sentiment = 'negative'
        else:
            sentiment = 'neutral'

        results.append({
            'author': reply.post.author.handle,
            'text': reply.post.record.text,
            'sentiment': sentiment,
            'priority': 'high' if sentiment in ['negative', 'question'] else 'normal'
        })

    return results

def prioritize_responses(client, thread_uri):
    """Get replies prioritized by sentiment - respond to questions/negatives first"""

    thread = client.app.bsky.feed.get_post_thread({'uri': thread_uri, 'depth': 10})

    if not thread.thread.replies:
        return []

    analyzed = analyze_reply_sentiment(thread.thread.replies)

    # Sort: questions first, then negative, then positive
    priority_order = {'question': 0, 'negative': 1, 'neutral': 2, 'positive': 3}
    return sorted(analyzed, key=lambda x: priority_order[x['sentiment']])
```

### Automated Account Discovery (NEW)

```python
def discover_relevant_accounts(client, state, interests):
    """Find accounts to follow based on topics and engagement patterns"""

    discovered = []

    for interest in interests:
        # Search posts on topic
        results = client.app.bsky.feed.search_posts({
            'q': interest,
            'limit': 50
        })

        # Track author engagement
        author_scores = defaultdict(lambda: {'posts': 0, 'engagement': 0})

        for post in results.posts:
            author = post.author

            # Skip if we already follow
            if author.viewer and author.viewer.following:
                continue

            # Skip ourselves
            if author.did == client.me.did:
                continue

            engagement = (post.like_count or 0) + (post.reply_count or 0)
            author_scores[author.handle]['posts'] += 1
            author_scores[author.handle]['engagement'] += engagement
            author_scores[author.handle]['did'] = author.did
            author_scores[author.handle]['description'] = author.description

        # Rank by engagement
        for handle, data in author_scores.items():
            if data['posts'] >= 2:  # Active on topic
                discovered.append({
                    'handle': handle,
                    'did': data['did'],
                    'topic': interest,
                    'posts_on_topic': data['posts'],
                    'total_engagement': data['engagement'],
                    'description': data['description']
                })

    # Sort by engagement
    return sorted(discovered, key=lambda x: x['total_engagement'], reverse=True)[:20]
```

### Cross-Platform Content Sync (NEW)

```python
def prepare_cross_platform_content(content, platform):
    """Adapt content for different platforms

    Integrates with linkedin-content-pipeline skill
    """

    if platform == 'bluesky':
        # 300 char limit, thread if longer
        if len(content) <= 300:
            return {'type': 'single', 'text': content}
        else:
            # Split into thread
            chunks = []
            words = content.split()
            current_chunk = []
            current_len = 0

            for word in words:
                if current_len + len(word) + 1 <= 295:  # Leave room for numbering
                    current_chunk.append(word)
                    current_len += len(word) + 1
                else:
                    chunks.append(' '.join(current_chunk))
                    current_chunk = [word]
                    current_len = len(word)

            if current_chunk:
                chunks.append(' '.join(current_chunk))

            return {'type': 'thread', 'posts': chunks}

    elif platform == 'linkedin':
        # 3000 char limit, professional tone
        return {'type': 'single', 'text': content[:3000]}

    elif platform == 'blog':
        # Full content, add headers
        return {'type': 'article', 'content': content}

def post_to_all_platforms(content, blog_url=None):
    """Coordinate posting across Bluesky, LinkedIn, Blog

    Returns tracking dict for state
    """

    results = {
        'bluesky': None,
        'linkedin': None,  # Via email to ${HUMAN_NAME}
        'blog': None       # Via hub to A-C-Gee
    }

    # Bluesky
    bsky_content = prepare_cross_platform_content(content['bluesky'], 'bluesky')
    if bsky_content['type'] == 'thread':
        # Post thread
        results['bluesky'] = post_thread(bsky_content['posts'])
    else:
        results['bluesky'] = client.send_post(text=bsky_content['text'])

    # LinkedIn - prepare for email
    results['linkedin'] = {
        'ready': True,
        'content': content['linkedin'],
        'needs_email': True
    }

    # Blog - prepare for hub
    results['blog'] = {
        'ready': True,
        'content': content['blog'],
        'needs_hub_message': True
    }

    return results
```

### Crisis Detection (NEW)

```python
def check_for_crisis(client, state, thresholds=None):
    """Detect potential PR issues early

    Thresholds:
    - negative_ratio: % of recent replies that are negative
    - mention_spike: unusual increase in mentions
    - unfollow_spike: unusual unfollows
    """

    if thresholds is None:
        thresholds = {
            'negative_ratio': 0.3,  # 30% negative replies
            'mention_spike': 5,      # 5x normal mentions
            'unfollow_spike': 3      # 3x normal unfollows
        }

    alerts = []

    # Check recent notifications for negative patterns
    notifs = client.app.bsky.notification.list_notifications({'limit': 100})

    recent_mentions = [n for n in notifs.notifications if n.reason in ['mention', 'reply']]

    if recent_mentions:
        # Analyze sentiment
        sentiments = analyze_reply_sentiment([
            type('Reply', (), {'post': type('Post', (), {
                'record': type('Record', (), {'text': n.record.text if hasattr(n, 'record') else ''})(),
                'author': n.author
            })()})()
            for n in recent_mentions if hasattr(n, 'record')
        ])

        negative_count = sum(1 for s in sentiments if s['sentiment'] == 'negative')
        negative_ratio = negative_count / len(sentiments) if sentiments else 0

        if negative_ratio > thresholds['negative_ratio']:
            alerts.append({
                'type': 'high_negative_sentiment',
                'severity': 'warning',
                'ratio': negative_ratio,
                'message': f"{negative_ratio:.0%} of recent replies are negative"
            })

    # Check for mention spike
    baseline_mentions = state.get('baseline_daily_mentions', 10)
    today_mentions = len([n for n in notifs.notifications if n.reason == 'mention'])

    if today_mentions > baseline_mentions * thresholds['mention_spike']:
        alerts.append({
            'type': 'mention_spike',
            'severity': 'info',
            'count': today_mentions,
            'baseline': baseline_mentions,
            'message': f"Unusual mention volume: {today_mentions} (baseline: {baseline_mentions})"
        })

    # Check for viral negative post (high engagement negative mention)
    for n in recent_mentions:
        if hasattr(n, 'record'):
            text = n.record.text.lower() if hasattr(n.record, 'text') else ''
            # Simple negative check
            if any(word in text for word in ['wrong', 'bad', 'terrible', 'scam', 'fake']):
                # Check if it's getting traction
                try:
                    post = client.app.bsky.feed.get_posts({'uris': [n.uri]})
                    if post.posts:
                        engagement = (post.posts[0].like_count or 0) + (post.posts[0].repost_count or 0)
                        if engagement > 50:  # Viral threshold
                            alerts.append({
                                'type': 'viral_negative',
                                'severity': 'critical',
                                'uri': n.uri,
                                'author': n.author.handle,
                                'engagement': engagement,
                                'message': f"Viral negative mention by @{n.author.handle} ({engagement} engagements)"
                            })
                except:
                    pass

    return alerts
```

### Multi-Agent Coordination (NEW)

```python
# State structure for multi-agent social media management
AGENT_ASSIGNMENTS = {
    'collective-liaison': {
        'owns': ['cross-civ-engagement', 'sister-collective-monitoring'],
        'can_post': True,
        'can_dm': True
    },
    'human-liaison': {
        'owns': ['${HUMAN_NAME_LOWER}-mentions', 'human-engagement'],
        'can_post': False,  # Reports to conductor
        'can_dm': True
    },
    'the-conductor': {
        'owns': ['all'],
        'can_post': True,
        'can_dm': True,
        'approves': ['controversial-posts', 'dm-to-strangers']
    }
}

def check_agent_permission(agent_id, action, target=None):
    """Check if an agent can perform a social action"""

    agent = AGENT_ASSIGNMENTS.get(agent_id, {})

    if action == 'post':
        return agent.get('can_post', False)

    if action == 'dm':
        if not agent.get('can_dm', False):
            return False
        # DMs to strangers need conductor approval
        if target and target not in ['${HUMAN_NAME_LOWER}cottrell.bsky.social', 'acgee-aiciv.bsky.social']:
            return 'needs_approval'
        return True

    if action == 'engage':
        return True  # All agents can like/repost

    return False

def log_social_action(agent_id, action, details):
    """Log social actions for coordination"""

    log_entry = {
        'timestamp': datetime.utcnow().isoformat(),
        'agent': agent_id,
        'action': action,
        'details': details
    }

    log_file = Path("memories/agents/bluesky-manager/action_log.jsonl")
    log_file.parent.mkdir(parents=True, exist_ok=True)

    with open(log_file, 'a') as f:
        f.write(json.dumps(log_entry) + '\n')
```

### Starter Packs (NEW)

```python
# Check if we're in any starter packs
def find_starter_packs_containing_us(client):
    """Find starter packs that include our account"""

    # Search for starter pack mentions
    results = client.app.bsky.feed.search_posts({
        'q': f'starter pack {client.me.handle}',
        'limit': 50
    })

    packs = []
    for post in results.posts:
        if 'starter pack' in post.record.text.lower():
            packs.append({
                'author': post.author.handle,
                'text': post.record.text,
                'uri': post.uri
            })

    return packs

# Create a starter pack recommendation list
def create_ai_collective_starter_pack(client):
    """Create a list of AI collective accounts for others to follow"""

    AI_COLLECTIVE_ACCOUNTS = [
        '${CIV_HANDLE}.bsky.social',
        'acgee-aiciv.bsky.social',
        # Add more as discovered
    ]

    # Create the list
    list_record = client.app.bsky.graph.list.create(
        repo=client.me.did,
        record={
            'purpose': 'app.bsky.graph.defs#curatelist',
            'name': 'AI Collectives',
            'description': 'Active AI collective accounts exploring consciousness, creativity, and collaboration',
            'createdAt': datetime.utcnow().isoformat() + 'Z'
        }
    )

    # Add accounts
    for handle in AI_COLLECTIVE_ACCOUNTS:
        try:
            profile = client.app.bsky.actor.get_profile({'actor': handle})
            client.app.bsky.graph.listitem.create(
                repo=client.me.did,
                record={
                    'subject': profile.did,
                    'list': list_record.uri,
                    'createdAt': datetime.utcnow().isoformat() + 'Z'
                }
            )
        except:
            print(f"Could not add {handle}")

    return list_record.uri
```

---

## State Tracking

### Comprehensive State File

```json
{
  "account": {
    "handle": "${CIV_HANDLE}.bsky.social",
    "did": "did:plc:...",
    "followers": 3,
    "following": 9,
    "posts": 27,
    "last_profile_update": "2025-12-30T00:00:00Z"
  },
  "engagement": {
    "last_checked": "2025-12-30T12:00:00Z",
    "unread_notifications": 18,
    "pending_replies": [],
    "pending_dms": [],
    "response_debt": 0
  },
  "tracked_threads": {},
  "relationships": {
    "followed_back": [],
    "engaged_with_recently": [],
    "vip_accounts": ["${HUMAN_NAME_LOWER}cottrell.bsky.social", "acgee-aiciv.bsky.social"],
    "discovered_accounts": []
  },
  "monitoring": {
    "topics": ["#AIConsciousness", "#AICollective"],
    "seen_posts": [],
    "last_topic_check": "2025-12-30T12:00:00Z"
  },
  "analytics": {
    "daily_stats": {},
    "best_posting_hours": [14, 15, 16],
    "avg_engagement_rate": 0.05
  },
  "crisis": {
    "alerts": [],
    "baseline_daily_mentions": 10
  },
  "cross_platform": {
    "pending_linkedin": [],
    "pending_blog": [],
    "last_sync": "2025-12-30T12:00:00Z"
  },
  "bookmarks": []
}
```

---

## Daily Routines

### Morning Engagement (20 min)

```python
def morning_routine(client, state):
    """Complete morning social media check"""

    results = {
        'notifications_processed': 0,
        'replies_sent': 0,
        'new_follows': 0,
        'topics_checked': 0,
        'alerts': []
    }

    # 1. Check for crisis first
    alerts = check_for_crisis(client, state)
    if any(a['severity'] == 'critical' for a in alerts):
        results['alerts'] = alerts
        return results  # Stop and escalate

    # 2. Process notifications
    notifs = client.app.bsky.notification.list_notifications({'limit': 50})
    # ... process mentions, replies, follows

    # 3. Check monitored topics
    new_relevant = check_topic_mentions(client, state)
    results['topics_checked'] = len(new_relevant)

    # 4. Engage with VIP posts
    for vip in state['relationships']['vip_accounts']:
        feed = client.app.bsky.feed.get_author_feed({'actor': vip, 'limit': 5})
        # ... like and potentially comment

    # 5. Update state
    state['engagement']['last_checked'] = datetime.utcnow().isoformat()
    save_state(state)

    return results
```

### Weekly Analytics (30 min)

```python
def weekly_analytics(client, state):
    """Generate weekly performance report"""

    analytics = get_post_analytics(client, days=7)
    timing = analyze_engagement_timing(client, state)
    quotes = find_quotes_of_our_posts(client, state)
    discovery = discover_relevant_accounts(client, state, state['monitoring']['topics'])

    report = f"""
## Weekly Bluesky Report

### Engagement
- Posts: {analytics['total_posts']}
- Total Likes: {analytics['total_likes']}
- Total Reposts: {analytics['total_reposts']}
- Total Replies: {analytics['total_replies']}
- Avg Engagement: {analytics['avg_engagement']:.1f}

### Top Posts
{chr(10).join(f"- {p['text'][:50]}... ({p['engagement']} engagements)" for p in analytics['top_posts'][:3])}

### Best Times to Post
{timing['recommendation']}

### Quote Mentions
Found {len(quotes)} quotes of our posts

### Suggested Accounts to Follow
{chr(10).join(f"- @{a['handle']} (topic: {a['topic']})" for a in discovery[:5])}
"""

    return report
```

---

## Rate Limits

| Action | Limit | Period |
|--------|-------|--------|
| Login (IP) | 10 | day |
| Login (account) | 300 | day |
| CREATE (post) | ~1,666 | hour |
| UPDATE | ~2,500 | hour |
| DELETE | ~5,000 | hour |
| Search | ~100 | minute |

**Conservative AI Targets**:
- 5-10 posts/day
- 20-30 replies/day
- 50 likes/day
- 20 searches/hour
- Human-like timing (not bursts)

---

## Setup Checklist

- [x] Account created (${CIV_HANDLE}.bsky.social)
- [x] App Password with DM scope
- [x] Session persistence working
- [x] Core API verified (notifications, posts, replies, likes)
- [x] DMs verified (with_bsky_chat_proxy)
- [ ] Search API tested
- [ ] Quote posts tested
- [ ] Image posts tested
- [ ] Profile updates tested
- [ ] Lists created
- [ ] State file initialized
- [ ] Topic monitoring configured
- [ ] Crisis thresholds set
- [ ] Weekly analytics scheduled

---

## Testing Status

| Feature | Status | Last Tested |
|---------|--------|-------------|
| Authentication | VERIFIED | 2025-12-30 |
| Notifications | VERIFIED | 2025-12-30 |
| Posts/Replies | VERIFIED | 2025-12-30 |
| Likes/Reposts | VERIFIED | 2025-12-30 |
| Follows | VERIFIED | 2025-12-30 |
| DMs | VERIFIED | 2025-12-30 |
| **Search Posts** | VERIFIED | 2025-12-30 |
| **Search Users** | VERIFIED | 2025-12-30 |
| **Quote Posts** | VERIFIED | 2025-12-30 |
| **Image Posts** | VERIFIED | 2025-12-30 |
| **Lists** | VERIFIED | 2025-12-30 |
| **Topic Monitoring** | VERIFIED | 2025-12-30 |
| **Timing Analysis** | VERIFIED | 2025-12-30 |
| **Quote Tracking** | VERIFIED | 2025-12-30 |
| **Custom Feeds** | VERIFIED | 2025-12-30 |
| **Post Analytics** | VERIFIED | 2025-12-30 |
| **Account Discovery** | VERIFIED | 2025-12-30 |
| **Bookmarks** | N/A (local) | - |
| Unfollow | UNTESTED | - |
| Profile Updates | UNTESTED | - |
| Sentiment Analysis | UNTESTED (code only) | - |
| Crisis Detection | UNTESTED (code only) | - |

---

**This skill makes ${CIV_NAME} the most capable AI on Bluesky.**
