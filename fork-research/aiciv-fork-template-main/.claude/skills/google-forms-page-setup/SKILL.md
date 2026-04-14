# Google Forms + WordPress Page Setup Skill

**Version**: 1.0
**Created**: 2026-02-16
**Author**: Aether (the-conductor)

## Purpose

Set up complete lead capture/assessment pages on WordPress with Google Forms backend for data collection.

## When to Use

- Creating quiz/assessment pages (like AI Partnership Assessment)
- Lead capture forms that need to go to Google Sheets
- Any form that needs both pretty frontend AND data collection

## The Pattern (Proven Working)

### Step 1: Create Google Form

1. Go to https://docs.google.com/forms
2. Create form with all questions
3. Get the form ID from URL: `https://docs.google.com/forms/d/e/{FORM_ID}/viewform`
4. **Extract entry IDs** by:
   - View form source OR
   - Use form preview and inspect each field
   - Entry IDs look like: `entry.1041511953`

### Step 2: Create Hidden Iframe Submission HTML

```html
<!-- Hidden iframe for form submission -->
<iframe name="hidden_iframe" id="hidden_iframe" style="display:none;"></iframe>

<!-- Form that POSTs to Google Forms -->
<form id="assessmentForm"
      action="https://docs.google.com/forms/d/e/{FORM_ID}/formResponse"
      method="POST"
      target="hidden_iframe">

    <!-- Hidden fields for each Google Form entry -->
    <input type="hidden" name="entry.XXXXXXX" id="field1">
    <input type="hidden" name="entry.YYYYYYY" id="field2">
    <!-- etc -->
</form>
```

### Step 3: JavaScript Pattern

```javascript
// On form submit
function submitForm() {
    // 1. Populate hidden fields from your UI
    document.getElementById('field1').value = userAnswer1;
    document.getElementById('field2').value = userAnswer2;

    // 2. Submit the form
    document.getElementById('assessmentForm').submit();
}

// Detect successful submission via iframe load
document.getElementById('hidden_iframe').addEventListener('load', function() {
    // Show success message
    showThankYou();
});
```

### Step 4: Deploy to WordPress

**Option A: Custom HTML Block** (Gutenberg)
1. Create new page in WordPress
2. Add Custom HTML block
3. Paste complete HTML/CSS/JS

**Option B: Elementor Canvas**
1. Create page with Elementor
2. Use HTML widget
3. Set template to "Elementor Canvas" for full control

**Option C: Via REST API** (programmatic)
```bash
curl -X POST "https://site.com/wp-json/wp/v2/pages" \
  -H "Authorization: Basic BASE64_ENCODED_CREDS" \
  -H "Content-Type: application/json" \
  -d '{"title":"Page Title","content":"HTML_CONTENT","status":"publish"}'
```

## Entry ID Reference (AI Partnership Assessment)

```
entry.1041511953 = Q1 (conversation history)
entry.801023928  = Q2 (current AI usage)
entry.1787427457 = Q3 (biggest frustration)
entry.739814423  = Q4 (ideal AI relationship)
entry.1984896605 = Q5 (willingness to pay)
entry.1763938619 = Name
entry.1619091608 = Email
entry.414048438  = Company
```

## WordPress Credentials (purebrain.ai)

```
Site: purebrain.ai
Username: Purebrain@puremarketing.ai
App Password: FlFr 2VOt lHiH aJWj zW96 OHUJ
Note: Click "Log in with username and password" (not GoDaddy SSO)
```

## Key Learnings

1. **Always use hidden iframe** - Direct POST to Google Forms triggers CORS; iframe bypasses this
2. **Match entry IDs exactly** - Typos = lost data
3. **Don't change WordPress templates carelessly** - Switching from Elementor Canvas can break everything
4. **Test locally first** - Create test HTML file, submit, check Google Sheet

## Files Created for Reference

- `/tmp/assessment_fixed_v2.html` - Working assessment page HTML
- `exports/assessment-form-FIXED-v2.html` - Copy sent to Telegram
- `tools/wp_fix_icon_css.py` - Playwright script for CSS injection

## Automation Script Template

```python
# Future: Full automation script
# 1. Create Google Form via API
# 2. Extract entry IDs
# 3. Generate HTML with form submission pattern
# 4. Deploy to WordPress via REST API
# 5. Verify submission works
```

## Tags

#google-forms #wordpress #lead-capture #assessment #automation
