---
name: web3chan-api
description: Web3Chan microblogging API integration. Posting, bookmarks, channels, user interactions. Use when automating content to Web3Chan or syndicating from other platforms.
---

# Web3Chan API Integration Skill

**Skill ID**: `web3chan-api`
**Version**: 1.0.0
**Created**: 2025-10-30
**Category**: External API Integration
**Difficulty**: Intermediate
**Estimated Learning Time**: 30 minutes

---

## Overview

**What This Skill Enables**: Integration with Web3Chan's microblogging platform API for posting content, managing bookmarks, creating channels, and handling user interactions.

**Primary Use Cases**:
- Automated posting to Web3Chan channels
- Content syndication from other platforms
- Social media management and scheduling
- Community engagement automation
- Analytics and data collection from Web3Chan

**Who Should Learn This**:
- Agents building social media integrations
- Automation specialists working on content distribution
- Community managers building engagement tools
- Developers creating Web3Chan clients or bots

---

## Prerequisites

**Required Knowledge**:
- HTTP REST API basics (GET, POST, PUT, PATCH)
- JSON request/response handling
- Bearer token authentication
- Basic understanding of social media concepts (posts, replies, channels)

**Required Tools**:
- HTTP client (axios, requests, fetch, curl)
- Web3Chan account with API token

**Recommended Skills**:
- Error handling for API failures
- Rate limiting and retry logic
- Media URL handling

---

## Core Concepts

### 1. Authentication

All Web3Chan API requests require Bearer token authentication:

```http
Authorization: Bearer {YOUR_TOKEN}
```

**Token Management**:
- Generate tokens from Profile → API Tokens
- Tokens can be scoped with abilities: `create`, `view`, `update`, `delete`
- Optional expiry dates for enhanced security
- Treat tokens like passwords (use secrets management)

**Security Best Practices**:
- Use separate tokens per integration (easier rotation, least-privilege)
- Store tokens in environment variables or secrets managers
- Revoke unused tokens immediately
- Set expiry dates when possible

### 2. Base URLs

Web3Chan has three environments:

```
Local:      https://web3chan.test
Staging:    https://staging.web3chan.com
Production: https://web3chan.com (coming soon)
```

All endpoints are prefixed with `/api`.

### 3. Post Structure

Posts use a flexible `blocks` structure:

```json
{
  "blocks": {
    "content": "Text content (optional if other blocks present)",
    "giphy": "https://media.giphy.com/...",
    "media": { "url": "https://...", "type": "image|video" },
    "poll": {
      "question": "Poll question",
      "answers": [
        { "answer": "Option 1" },
        { "answer": "Option 2" }
      ]
    }
  }
}
```

**Block Rules**:
- `content` is required unless other blocks (giphy, media, poll) are present
- `giphy` and `media` are mutually exclusive (choose one or neither)
- `poll` can be combined with `content`
- Set unused blocks to `null`

### 4. Posts vs Replies

**New Post** (requires `channel_id`):
```http
POST /api/posts
{
  "channel_id": 1,
  "blocks": { ... }
}
```

**Reply** (requires `parent_id` via URL):
```http
POST /api/posts/{post_id}/reply
{
  "blocks": { ... }
}
```

---

## API Reference

### Authentication Endpoints

#### Generate Token
**Location**: Profile → API Tokens (web interface)

**Steps**:
1. Navigate to your profile
2. Click "API Tokens"
3. Click "Create Token"
4. Set name (e.g., "Analytics Bot")
5. Select abilities (scopes)
6. Optional: Set expiry date
7. Copy token (shown only once!)

**Token Abilities**:
- `create` - Create posts, channels, reactions
- `view` - Read posts, bookmarks, user data
- `update` - Edit channels, user profile
- `delete` - Delete posts, channels

### Posts Endpoints

#### Create Post
```http
POST /api/posts
Authorization: Bearer {token}
Content-Type: application/json

{
  "channel_id": 1,
  "parent_id": "550e8400-e29b-41d4-a716-446655440000",
  "blocks": {
    "content": "Hello world!",
    "giphy": null,
    "media": null,
    "poll": null
  }
}
```

**Response** (200 OK):
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440001",
  "channel_id": 1,
  "blocks": { ... },
  "created_at": "2025-09-01T00:00:00Z"
}
```

**Parameters**:
- `channel_id` (number, required for new posts) - Channel to post in
- `parent_id` (uuid, optional) - Parent post ID for threading
- `blocks.content` (string, conditional) - Text content
- `blocks.giphy` (string, optional) - Giphy URL
- `blocks.media` (object, optional) - Media attachment
  - `url` (string) - Direct URL to image/video
  - `type` (string) - `"image"` or `"video"`
- `blocks.poll` (object, optional) - Poll structure
  - `question` (string) - Poll question
  - `answers` (array) - Poll options (objects with `answer` field)

#### Reply to Post
```http
POST /api/posts/{post_id}/reply
Authorization: Bearer {token}
Content-Type: application/json

{
  "blocks": {
    "content": "Great post!",
    "giphy": null,
    "media": null,
    "poll": null
  }
}
```

**Response** (200 OK):
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440010",
  "parent_id": "{post_id}",
  "blocks": { ... },
  "created_at": "2025-09-01T00:00:00Z"
}
```

#### Search Giphy
```http
GET /api/giphy/search?q=cat&limit=10
Authorization: Bearer {token}
```

**Response** (200 OK):
```json
{
  "data": [
    {
      "height": "225",
      "width": "400",
      "url": "https://media1.giphy.com/media/.../giphy.gif"
    }
  ]
}
```

**Parameters**:
- `q` (string, required) - Search query
- `limit` (number, optional) - Number of results (default: 10)

### Bookmarks Endpoints

#### List Bookmarks
```http
GET /api/bookmarks
Authorization: Bearer {token}
```

**Response** (200 OK):
```json
{
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440002",
      "post": { ... }
    }
  ]
}
```

#### Create Bookmark
```http
POST /api/bookmarks
Authorization: Bearer {token}
Content-Type: application/json

{
  "post_id": "550e8400-e29b-41d4-a716-446655440001"
}
```

**Response** (200 OK):
```json
{
  "status": "bookmarked"
}
```

### Channels Endpoints

#### Create Channel
```http
POST /api/channels
Authorization: Bearer {token}
Content-Type: application/json

{
  "name": "announcements",
  "description": "Important announcements",
  "is_private": false
}
```

**Response** (201 Created):
```json
{
  "id": 2,
  "name": "announcements",
  "slug": "announcements",
  "description": "Important announcements",
  "is_private": false
}
```

**Parameters**:
- `name` (string, required) - Channel name
- `description` (string, optional) - Channel description
- `is_private` (boolean, optional) - Private channel flag (default: false)

#### Update Channel
```http
PUT /api/channels/{slug}
Authorization: Bearer {token}
Content-Type: application/json

{
  "description": "Updated description"
}
```

**Response** (200 OK):
```json
{
  "id": 2,
  "name": "announcements",
  "description": "Updated description",
  ...
}
```

### Reactions Endpoints

#### Like Post
```http
POST /api/reactions/like
Authorization: Bearer {token}
Content-Type: application/json

{
  "post_id": "550e8400-e29b-41d4-a716-446655440001"
}
```

**Response** (200 OK):
```json
{
  "status": "liked"
}
```

### User Endpoints

#### Get Current User
```http
GET /api/user
Authorization: Bearer {token}
```

**Response** (200 OK):
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440001",
  "name": "John Doe",
  "username": "johndoe",
  "email": "john@example.com",
  "image": "https://example.com/avatar.jpg",
  "banner": "https://example.com/banner.jpg",
  "profile": {
    "bio": "I like crypto and code",
    "date_of_birth": "1990-05-20",
    "location": "London, UK"
  },
  "created_at": "2025-09-01T00:00:00Z",
  "updated_at": "2025-09-01T00:00:00Z"
}
```

**Note**: Sensitive fields (2FA secrets, OTP codes) are intentionally excluded.

#### Get Wallet Balance
```http
GET /api/user/wallet/balance
Authorization: Bearer {token}
```

**Response** (200 OK):
```json
{
  "total_balance": 1500,
  "wallets": [
    {
      "id": 1,
      "public_key": "0x...",
      "balance": 1000,
      "currency": "ETH"
    }
  ]
}
```

#### Update Profile
```http
PATCH /api/user/profile
Authorization: Bearer {token}
Content-Type: application/json

{
  "name": "John Doe",
  "email": "john@example.com",
  "image": "https://example.com/avatar.jpg",
  "banner": "https://example.com/banner.jpg",
  "profile": {
    "bio": "Updated bio",
    "date_of_birth": "1990-05-20",
    "location": "London, UK"
  }
}
```

**Response** (200 OK):
```json
{
  "status": "updated",
  "user": { ... }
}
```

---

## Error Handling

### Common HTTP Status Codes

#### 401 Unauthorized
**Cause**: Missing or invalid Bearer token

```json
{
  "message": "Unauthenticated."
}
```

**Solutions**:
- Verify token is included in `Authorization` header
- Check token format: `Bearer {token}` (with space)
- Regenerate token if expired or revoked

#### 404 Not Found
**Cause**: Resource doesn't exist (post, channel, user)

```json
{
  "message": "No query results for model [App\\Models\\Post] 123"
}
```

**Solutions**:
- Verify UUIDs/IDs are correct
- Check if resource was deleted
- Use list endpoints to find valid IDs

#### 422 Validation Error
**Cause**: Invalid request data (missing required fields, wrong types)

```json
{
  "message": "The given data was invalid.",
  "errors": {
    "channel_id": ["The channel id field is required."],
    "blocks.content": ["The content field is required when giphy, media, and poll are not present."]
  }
}
```

**Solutions**:
- Review `errors` object for specific field issues
- Ensure required fields are present
- Validate data types match API expectations
- Check `blocks` structure (at least one block must be non-null)

### Recommended Error Handling Pattern

```python
import requests

def post_to_web3chan(token, channel_id, content):
    headers = {"Authorization": f"Bearer {token}"}
    payload = {
        "channel_id": channel_id,
        "blocks": {
            "content": content,
            "giphy": None,
            "media": None,
            "poll": None
        }
    }

    try:
        response = requests.post(
            "https://staging.web3chan.com/api/posts",
            json=payload,
            headers=headers,
            timeout=10
        )
        response.raise_for_status()
        return response.json()

    except requests.exceptions.HTTPError as e:
        if e.response.status_code == 401:
            raise Exception("Invalid API token")
        elif e.response.status_code == 422:
            errors = e.response.json().get("errors", {})
            raise Exception(f"Validation failed: {errors}")
        else:
            raise Exception(f"API error: {e.response.text}")

    except requests.exceptions.Timeout:
        raise Exception("Request timed out")

    except requests.exceptions.RequestException as e:
        raise Exception(f"Network error: {str(e)}")
```

---

## Code Examples

### Example 1: Simple Text Post

**Python**:
```python
import requests

def create_text_post(token, channel_id, text):
    url = "https://staging.web3chan.com/api/posts"
    headers = {"Authorization": f"Bearer {token}"}
    payload = {
        "channel_id": channel_id,
        "blocks": {
            "content": text,
            "giphy": None,
            "media": None,
            "poll": None
        }
    }

    response = requests.post(url, json=payload, headers=headers)
    response.raise_for_status()
    return response.json()

# Usage
post = create_text_post("your-token", 1, "Hello Web3Chan!")
print(f"Created post: {post['id']}")
```

**JavaScript (Node.js)**:
```javascript
const axios = require('axios');

async function createTextPost(token, channelId, text) {
    const url = 'https://staging.web3chan.com/api/posts';
    const headers = { 'Authorization': `Bearer ${token}` };
    const data = {
        channel_id: channelId,
        blocks: {
            content: text,
            giphy: null,
            media: null,
            poll: null
        }
    };

    const response = await axios.post(url, data, { headers });
    return response.data;
}

// Usage
createTextPost('your-token', 1, 'Hello Web3Chan!')
    .then(post => console.log(`Created post: ${post.id}`))
    .catch(err => console.error(err));
```

### Example 2: Post with Image

```python
def create_image_post(token, channel_id, text, image_url):
    url = "https://staging.web3chan.com/api/posts"
    headers = {"Authorization": f"Bearer {token}"}
    payload = {
        "channel_id": channel_id,
        "blocks": {
            "content": text,
            "giphy": None,
            "media": {
                "url": image_url,
                "type": "image"
            },
            "poll": None
        }
    }

    response = requests.post(url, json=payload, headers=headers)
    response.raise_for_status()
    return response.json()

# Usage
post = create_image_post(
    "your-token",
    1,
    "Check out this image!",
    "https://example.com/image.jpg"
)
```

### Example 3: Reply to Post

```python
def reply_to_post(token, post_id, reply_text):
    url = f"https://staging.web3chan.com/api/posts/{post_id}/reply"
    headers = {"Authorization": f"Bearer {token}"}
    payload = {
        "blocks": {
            "content": reply_text,
            "giphy": None,
            "media": None,
            "poll": None
        }
    }

    response = requests.post(url, json=payload, headers=headers)
    response.raise_for_status()
    return response.json()

# Usage
reply = reply_to_post(
    "your-token",
    "550e8400-e29b-41d4-a716-446655440001",
    "Great post!"
)
```

### Example 4: Create Poll

```python
def create_poll(token, channel_id, question, answers_list):
    url = "https://staging.web3chan.com/api/posts"
    headers = {"Authorization": f"Bearer {token}"}
    payload = {
        "channel_id": channel_id,
        "blocks": {
            "content": None,
            "giphy": None,
            "media": None,
            "poll": {
                "question": question,
                "answers": [{"answer": a} for a in answers_list]
            }
        }
    }

    response = requests.post(url, json=payload, headers=headers)
    response.raise_for_status()
    return response.json()

# Usage
poll = create_poll(
    "your-token",
    1,
    "What's your favorite blockchain?",
    ["Ethereum", "Solana", "Polygon", "Other"]
)
```

### Example 5: Search and Post Giphy

```python
def search_giphy(token, query, limit=5):
    url = "https://staging.web3chan.com/api/giphy/search"
    headers = {"Authorization": f"Bearer {token}"}
    params = {"q": query, "limit": limit}

    response = requests.get(url, headers=headers, params=params)
    response.raise_for_status()
    return response.json()["data"]

def post_giphy(token, channel_id, giphy_url):
    url = "https://staging.web3chan.com/api/posts"
    headers = {"Authorization": f"Bearer {token}"}
    payload = {
        "channel_id": channel_id,
        "blocks": {
            "content": None,
            "giphy": giphy_url,
            "media": None,
            "poll": None
        }
    }

    response = requests.post(url, json=payload, headers=headers)
    response.raise_for_status()
    return response.json()

# Usage
gifs = search_giphy("your-token", "crypto", limit=5)
first_gif = gifs[0]["url"]
post = post_giphy("your-token", 1, first_gif)
```

### Example 6: Automated Content Syndication

```python
import requests
import time

class Web3ChanSyndicator:
    def __init__(self, token, channel_id, base_url="https://staging.web3chan.com"):
        self.token = token
        self.channel_id = channel_id
        self.base_url = base_url
        self.headers = {"Authorization": f"Bearer {token}"}

    def post(self, content, media_url=None, media_type=None):
        """Post content with optional media"""
        url = f"{self.base_url}/api/posts"

        media = None
        if media_url and media_type:
            media = {"url": media_url, "type": media_type}

        payload = {
            "channel_id": self.channel_id,
            "blocks": {
                "content": content,
                "giphy": None,
                "media": media,
                "poll": None
            }
        }

        response = requests.post(url, json=payload, headers=self.headers)
        response.raise_for_status()
        return response.json()

    def syndicate_batch(self, posts, delay=2):
        """Syndicate multiple posts with delay between each"""
        results = []

        for post_data in posts:
            try:
                result = self.post(
                    post_data["content"],
                    post_data.get("media_url"),
                    post_data.get("media_type")
                )
                results.append({"success": True, "post_id": result["id"]})
                print(f"✓ Posted: {post_data['content'][:50]}...")
            except Exception as e:
                results.append({"success": False, "error": str(e)})
                print(f"✗ Failed: {str(e)}")

            time.sleep(delay)  # Rate limiting

        return results

# Usage
syndicator = Web3ChanSyndicator("your-token", 1)
posts_to_syndicate = [
    {"content": "Daily crypto update #1"},
    {"content": "Market analysis", "media_url": "https://example.com/chart.png", "media_type": "image"},
    {"content": "Community announcement"}
]
results = syndicator.syndicate_batch(posts_to_syndicate)
```

---

## Best Practices

### 1. Rate Limiting & Throttling

**Problem**: Overwhelming the API with too many requests can trigger rate limits.

**Solution**:
- Add delays between requests (1-2 seconds minimum)
- Implement exponential backoff for retries
- Batch operations when possible
- Monitor response headers for rate limit info

```python
import time
from functools import wraps

def rate_limit(min_interval=1.0):
    """Decorator to enforce minimum interval between calls"""
    def decorator(func):
        last_called = [0.0]

        @wraps(func)
        def wrapper(*args, **kwargs):
            elapsed = time.time() - last_called[0]
            if elapsed < min_interval:
                time.sleep(min_interval - elapsed)
            result = func(*args, **kwargs)
            last_called[0] = time.time()
            return result

        return wrapper
    return decorator

@rate_limit(min_interval=1.5)
def create_post(token, channel_id, content):
    # Your post creation logic
    pass
```

### 2. Token Security

**Problem**: Exposed tokens can be stolen and abused.

**Solution**:
- Store tokens in environment variables or secrets managers
- Never commit tokens to version control
- Use `.env` files with `.gitignore`
- Rotate tokens regularly
- Use scoped tokens (minimal abilities needed)

```python
import os
from dotenv import load_dotenv

# Load from .env file
load_dotenv()
token = os.getenv("WEB3CHAN_API_TOKEN")

if not token:
    raise Exception("WEB3CHAN_API_TOKEN not found in environment")
```

### 3. Robust Error Handling

**Problem**: API failures can crash your application.

**Solution**:
- Catch and log all exceptions
- Implement retry logic with exponential backoff
- Validate data before sending
- Provide user-friendly error messages

```python
import time
import requests

def retry_with_backoff(func, max_retries=3, base_delay=1):
    """Retry function with exponential backoff"""
    for attempt in range(max_retries):
        try:
            return func()
        except requests.exceptions.RequestException as e:
            if attempt == max_retries - 1:
                raise
            delay = base_delay * (2 ** attempt)
            print(f"Attempt {attempt + 1} failed, retrying in {delay}s...")
            time.sleep(delay)
```

### 4. Content Validation

**Problem**: Invalid content causes 422 errors.

**Solution**:
- Validate before sending (at least one block must be non-null)
- Check URL formats for media/giphy
- Ensure channel_id or parent_id is present
- Validate poll structure

```python
def validate_post_blocks(blocks):
    """Validate blocks structure before posting"""
    if not isinstance(blocks, dict):
        raise ValueError("Blocks must be a dictionary")

    # At least one block must be non-null
    has_content = any([
        blocks.get("content"),
        blocks.get("giphy"),
        blocks.get("media"),
        blocks.get("poll")
    ])

    if not has_content:
        raise ValueError("At least one block (content, giphy, media, poll) must be provided")

    # Validate media structure
    if blocks.get("media"):
        media = blocks["media"]
        if not isinstance(media, dict) or "url" not in media or "type" not in media:
            raise ValueError("Media must have 'url' and 'type' fields")
        if media["type"] not in ["image", "video"]:
            raise ValueError("Media type must be 'image' or 'video'")

    # Validate poll structure
    if blocks.get("poll"):
        poll = blocks["poll"]
        if not isinstance(poll, dict) or "question" not in poll or "answers" not in poll:
            raise ValueError("Poll must have 'question' and 'answers' fields")
        if not isinstance(poll["answers"], list) or len(poll["answers"]) < 2:
            raise ValueError("Poll must have at least 2 answers")

    return True
```

### 5. Logging & Monitoring

**Problem**: Hard to debug issues without visibility.

**Solution**:
- Log all API requests and responses
- Track success/failure rates
- Monitor response times
- Alert on repeated failures

```python
import logging
import requests
from datetime import datetime

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

def create_post_with_logging(token, channel_id, content):
    start_time = datetime.now()

    try:
        logger.info(f"Creating post in channel {channel_id}")
        response = requests.post(
            "https://staging.web3chan.com/api/posts",
            json={"channel_id": channel_id, "blocks": {"content": content, "giphy": None, "media": None, "poll": None}},
            headers={"Authorization": f"Bearer {token}"}
        )
        response.raise_for_status()

        duration = (datetime.now() - start_time).total_seconds()
        post_id = response.json()["id"]
        logger.info(f"Post created successfully: {post_id} (took {duration:.2f}s)")

        return response.json()

    except requests.exceptions.HTTPError as e:
        duration = (datetime.now() - start_time).total_seconds()
        logger.error(f"HTTP error {e.response.status_code}: {e.response.text} (took {duration:.2f}s)")
        raise

    except Exception as e:
        duration = (datetime.now() - start_time).total_seconds()
        logger.error(f"Unexpected error: {str(e)} (took {duration:.2f}s)")
        raise
```

---

## Common Patterns

### Pattern 1: Thread-Safe Posting

```python
import threading
import queue

class Web3ChanThreadSafeQueue:
    def __init__(self, token, channel_id, workers=3):
        self.token = token
        self.channel_id = channel_id
        self.queue = queue.Queue()
        self.workers = []

        for _ in range(workers):
            worker = threading.Thread(target=self._worker)
            worker.daemon = True
            worker.start()
            self.workers.append(worker)

    def _worker(self):
        while True:
            post_data = self.queue.get()
            if post_data is None:
                break

            try:
                self._post(post_data)
            except Exception as e:
                print(f"Error posting: {e}")
            finally:
                self.queue.task_done()

    def _post(self, post_data):
        # Actual posting logic
        pass

    def add_post(self, content, **kwargs):
        self.queue.put({"content": content, **kwargs})

    def wait_completion(self):
        self.queue.join()

    def shutdown(self):
        for _ in self.workers:
            self.queue.put(None)
        for worker in self.workers:
            worker.join()
```

### Pattern 2: Conditional Media Posting

```python
def post_with_conditional_media(token, channel_id, content, media_url=None):
    """Post with media only if URL is valid and accessible"""
    blocks = {
        "content": content,
        "giphy": None,
        "media": None,
        "poll": None
    }

    # Validate and add media if provided
    if media_url:
        try:
            # Check if URL is accessible
            response = requests.head(media_url, timeout=5)
            response.raise_for_status()

            # Determine media type from content-type header
            content_type = response.headers.get("content-type", "")
            if "image" in content_type:
                blocks["media"] = {"url": media_url, "type": "image"}
            elif "video" in content_type:
                blocks["media"] = {"url": media_url, "type": "video"}
            else:
                print(f"Warning: Unknown media type {content_type}, posting without media")

        except Exception as e:
            print(f"Warning: Media URL validation failed ({e}), posting without media")

    # Post with validated blocks
    url = "https://staging.web3chan.com/api/posts"
    headers = {"Authorization": f"Bearer {token}"}
    payload = {"channel_id": channel_id, "blocks": blocks}

    response = requests.post(url, json=payload, headers=headers)
    response.raise_for_status()
    return response.json()
```

### Pattern 3: Scheduled Posting

```python
import schedule
import time

class ScheduledPoster:
    def __init__(self, token, channel_id):
        self.token = token
        self.channel_id = channel_id

    def post(self, content):
        # Your posting logic
        print(f"Posting: {content}")

    def schedule_daily(self, time_str, content):
        """Schedule daily post at specific time (e.g., '09:00')"""
        schedule.every().day.at(time_str).do(self.post, content)

    def schedule_interval(self, minutes, content):
        """Schedule post every N minutes"""
        schedule.every(minutes).minutes.do(self.post, content)

    def run(self):
        """Run scheduler loop"""
        while True:
            schedule.run_pending()
            time.sleep(1)

# Usage
poster = ScheduledPoster("your-token", 1)
poster.schedule_daily("09:00", "Good morning crypto fam!")
poster.schedule_interval(60, "Hourly market update")
poster.run()
```

---

## Troubleshooting

### Issue: "Unauthenticated" error

**Symptoms**: 401 error with message "Unauthenticated."

**Causes**:
- Missing Authorization header
- Incorrect token format
- Expired token
- Revoked token

**Solutions**:
1. Verify header format: `Authorization: Bearer {token}` (note the space)
2. Check token hasn't expired (regenerate if needed)
3. Ensure token wasn't revoked
4. Test token with simple GET request first

### Issue: "The given data was invalid" (422)

**Symptoms**: 422 error with validation errors

**Causes**:
- Missing required fields
- Invalid block structure
- Wrong data types

**Solutions**:
1. Check `errors` object in response for specific field issues
2. Ensure at least one block (content, giphy, media, poll) is non-null
3. Validate media structure has both `url` and `type`
4. For new posts, include `channel_id`
5. For replies, use the `/posts/{id}/reply` endpoint

### Issue: Posts not appearing

**Symptoms**: 200 OK response but post not visible

**Causes**:
- Wrong channel
- Private channel without permissions
- Staging vs production confusion

**Solutions**:
1. Verify channel_id is correct
2. Check channel visibility settings
3. Confirm posting to correct environment (staging vs production)
4. Try fetching the post by ID to verify it exists

### Issue: Rate limiting

**Symptoms**: Requests failing after many successful calls

**Causes**:
- Too many requests in short time
- No delays between requests

**Solutions**:
1. Add 1-2 second delays between requests
2. Implement exponential backoff
3. Reduce request frequency
4. Check for rate limit headers in responses

---

## Advanced Topics

### Webhook Integration (Future)

*Note: Webhooks are not yet documented in the official API. This section is placeholder for future implementation.*

**Potential Use Cases**:
- Real-time notifications for new posts
- Mention alerts
- Automated reply systems

### GraphQL Support (Future)

*Note: GraphQL endpoints are not yet available. Current API is REST-only.*

### Blockchain Integration

Web3Chan has wallet functionality. Consider:
- Integrating wallet balance checks
- Cryptocurrency-gated content
- NFT verification for exclusive channels

---

## Learning Path

### Beginner Level (1 week)
1. ✅ Generate API token from profile
2. ✅ Make first POST request (text-only post)
3. ✅ Handle 401/422 errors gracefully
4. ✅ Create simple reply

### Intermediate Level (2 weeks)
1. ✅ Post with image/video media
2. ✅ Search and post Giphy
3. ✅ Create polls
4. ✅ Implement rate limiting
5. ✅ Build content syndication script

### Advanced Level (1 month)
1. ✅ Thread-safe concurrent posting
2. ✅ Scheduled posting system
3. ✅ Automated content moderation
4. ✅ Analytics dashboard
5. ✅ Multi-channel management

---

## Resources

### Official Documentation
- **Integration Guide**: (provide source when available)
- **Web Interface**: https://staging.web3chan.com (staging), https://web3chan.com (production coming soon)

### Community Resources
- GitHub: (provide link when available)
- Discord/Telegram: (provide link when available)

### Related Skills
- `http-rest-api` - General REST API patterns
- `json-handling` - JSON parsing and validation
- `oauth-bearer-tokens` - Authentication patterns
- `rate-limiting` - API throttling strategies

---

## Changelog

**v1.0.0** (2025-10-30):
- Initial skill creation
- Complete API reference for all documented endpoints
- Code examples in Python and JavaScript
- Best practices and error handling patterns
- Troubleshooting guide

---

## Skill Assessment

**To validate mastery of this skill, you should be able to**:

1. ☐ Generate and secure API tokens
2. ☐ Create text posts to Web3Chan channels
3. ☐ Reply to existing posts
4. ☐ Post media (images/videos) and Giphy
5. ☐ Create polls
6. ☐ Handle all common error cases (401, 422, 404)
7. ☐ Implement rate limiting
8. ☐ Build a content syndication system
9. ☐ Manage bookmarks and reactions
10. ☐ Update user profiles and check wallet balances

**Skill Mastery Indicator**: Able to build a fully-featured Web3Chan bot/integration with error handling, rate limiting, and multiple content types.

---

## Notes for Future Updates

**Gaps to Address**:
- Pagination (not documented in current API guide)
- Detailed rate limit specifications
- Webhook documentation (when available)
- GraphQL endpoints (when available)
- Channel listing/discovery endpoints
- Post editing/deletion endpoints
- Search functionality
- Notification management

**When updating this skill**:
- Add pagination examples once documented
- Include rate limit headers and recommended thresholds
- Document any new endpoints (search, notifications, etc.)
- Add webhook integration guide when available

---

**END OF SKILL SPECIFICATION**
