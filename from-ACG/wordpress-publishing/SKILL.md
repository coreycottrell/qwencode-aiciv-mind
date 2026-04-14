---
name: wordpress-publishing
description: Publish blog posts and media to jaredsanborn.com via WordPress REST API
version: 1.0.0
---

# WordPress Publishing Skill

**Purpose**: Publish blog posts, upload media, and manage content on jaredsanborn.com
**When to use**: Publishing Jared's personal blog posts, thought leadership content
**Tool Location**: `${CIV_ROOT}/tools/wordpress_publisher.py`

---

## Quick Reference

### Test Connection
```bash
python3 ${CIV_ROOT}/tools/wordpress_publisher.py test
```

### Publish Draft Post
```bash
python3 ${CIV_ROOT}/tools/wordpress_publisher.py publish \
    --title "Your Title Here" \
    --content "<p>HTML content here</p>" \
    --status draft
```

### Publish Live Post
```bash
python3 ${CIV_ROOT}/tools/wordpress_publisher.py publish \
    --title "Your Title Here" \
    --content-file /path/to/content.html \
    --status publish \
    --categories "AI" "Technology" \
    --tags "claude" "automation"
```

### Upload Image (for featured image)
```bash
python3 ${CIV_ROOT}/tools/wordpress_publisher.py upload-media \
    --file /path/to/image.jpg \
    --alt "Image description"
```

Then use the returned `media_id` in `--featured-image`.

---

## API Endpoints Used

| Resource | Endpoint |
|----------|----------|
| Posts | `/wp-json/wp/v2/posts` |
| Media | `/wp-json/wp/v2/media` |
| Categories | `/wp-json/wp/v2/categories` |
| Tags | `/wp-json/wp/v2/tags` |
| Users | `/wp-json/wp/v2/users/me` |

---

## Credentials

Stored in `${CIV_ROOT}/.env`:
```
WORDPRESS_URL=https://jaredsanborn.com
WORDPRESS_USER=jared
WORDPRESS_APP_PASSWORD=xxxx xxxx xxxx xxxx xxxx xxxx
```

Application passwords are generated in WordPress Admin:
Users > Profile > Application Passwords

---

## CLI Commands

### test
Test API connection and authentication.
```bash
python3 tools/wordpress_publisher.py test
```

### publish
Create a new blog post.

| Option | Required | Description |
|--------|----------|-------------|
| `--title` | Yes | Post title |
| `--content` | No* | HTML content (inline) |
| `--content-file` | No* | Path to HTML file |
| `--status` | No | draft, publish, pending, private (default: draft) |
| `--excerpt` | No | Post summary |
| `--featured-image` | No | Media ID for featured image |
| `--categories` | No | Category names (space-separated) |
| `--tags` | No | Tag names (created if needed) |
| `--slug` | No | URL slug |

*Must provide either `--content` or `--content-file`

### upload-media
Upload an image or file to the media library.

| Option | Required | Description |
|--------|----------|-------------|
| `--file` | Yes | Path to file |
| `--alt` | No | Alt text |
| `--caption` | No | Caption |

Returns `media_id` for use with `--featured-image`.

### list-posts
List recent posts.

| Option | Description |
|--------|-------------|
| `--limit` | Number of posts (default: 10) |
| `--status` | Filter by status |

### list-categories
List all available categories.

### list-tags
List all existing tags.

---

## Programmatic Usage

```python
from tools.wordpress_publisher import WordPressPublisher

with WordPressPublisher() as wp:
    # Test connection
    result = wp.test_connection()
    if not result["success"]:
        print(f"Error: {result['message']}")

    # Publish post
    result = wp.publish_post(
        title="My Post",
        content="<p>Hello World</p>",
        status="draft",
        categories=["AI", "Technology"],
        tags=["automation", "claude"]
    )

    if result["success"]:
        print(f"Published: {result['url']}")
        print(f"Post ID: {result['post_id']}")

    # Upload media
    media_result = wp.upload_media(
        "/path/to/image.jpg",
        alt_text="Description"
    )

    if media_result["success"]:
        # Use in featured image
        wp.publish_post(
            title="Post with Image",
            content="<p>Content</p>",
            featured_image_id=media_result["media_id"]
        )
```

---

## Content Format

WordPress expects **HTML** content, not Markdown.

### Convert Markdown to HTML

```python
import subprocess

def md_to_html(md_content):
    """Convert markdown to HTML using pandoc."""
    result = subprocess.run(
        ["pandoc", "-f", "markdown", "-t", "html"],
        input=md_content,
        capture_output=True,
        text=True
    )
    return result.stdout
```

Or use Python-Markdown:
```python
import markdown
html = markdown.markdown(md_content)
```

---

## Post Status Values

| Status | Description |
|--------|-------------|
| `draft` | Not visible, editable |
| `publish` | Live and public |
| `pending` | Awaiting review |
| `private` | Only visible to admins |
| `future` | Scheduled (requires date) |

---

## Common Workflows

### Publish Blog Post with Featured Image

```bash
# 1. Upload the image
python3 tools/wordpress_publisher.py upload-media \
    --file exports/featured-image.jpg \
    --alt "Blog post header image"
# Note the media_id from output

# 2. Publish with featured image
python3 tools/wordpress_publisher.py publish \
    --title "How AI Collectives Learn" \
    --content-file exports/blog-post.html \
    --status draft \
    --featured-image 123 \
    --categories "AI" "Technology" \
    --tags "ai-collective" "machine-learning"
```

### Preview Before Publishing

```bash
# First as draft
python3 tools/wordpress_publisher.py publish \
    --title "My Post" \
    --content-file content.html \
    --status draft

# Review at the returned URL
# Then update status in WordPress Admin, or publish new post
```

---

## Error Handling

### Authentication Failed (401)
- Check `WORDPRESS_USER` matches WordPress username
- Verify `WORDPRESS_APP_PASSWORD` is correct
- Ensure app password hasn't been revoked

### Connection Error
- Check `WORDPRESS_URL` is correct
- Verify site is accessible
- Check for SSL certificate issues

### Media Upload Failed
- Verify file exists and is readable
- Check file type is allowed by WordPress
- Ensure media library has space

### Category Not Found
- Categories must exist in WordPress
- Tags will be auto-created if they don't exist
- Use `list-categories` to see available options

---

## Security Notes

- Application passwords are site-specific (not your main password)
- They can be revoked without changing main password
- Each application should have its own app password
- App passwords are stored in `.env` (not committed to git)

---

## Integration with Daily Pipeline

This skill complements the existing content pipeline:

1. `/daily_blog` creates content
2. `/verify_publish` can use this to publish to jaredsanborn.com
3. Bluesky thread posted via bsky-manager

```
Content Pipeline:
  intel-scan -> deep-research -> daily-blog -> verify-publish
                                                    |
                                                    v
                                        wordpress_publisher.py
                                                    |
                                                    v
                                          jaredsanborn.com/blog/
```

---

## Dependencies

```bash
pip install httpx python-dotenv
```

---

## Learned

- WordPress REST API uses Basic Auth with Application Passwords
- App passwords may have spaces (valid per WordPress spec)
- Categories must pre-exist; tags auto-create
- Content must be HTML, not Markdown
- Media must be uploaded separately, then referenced by ID

---

**Tags**: wordpress, publishing, blog, api, jaredsanborn.com
