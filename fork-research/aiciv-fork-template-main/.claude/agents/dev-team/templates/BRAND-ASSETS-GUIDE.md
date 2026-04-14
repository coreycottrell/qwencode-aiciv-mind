# Brand Assets Guide

**Location:** Google Drive → C Level (CTO) Folder

---

## Colors & Fonts

**Path:**
```
C Level/
└── MA#1.BI-1-221221-Brand Bible/
    └── MA#1.BI-1.1-221118-Brand Identity/
        └── MA#1.BI-1.1-004-211107-Pure Tech Color Codes & Fonts
```

---

## Logos

**Base Path:**
```
C Level/
└── MA#1.BI-1-221221-Brand Bible/
    └── MA#1.BI-1.3-221118-Pure Marketing/
```

**Logo Files:**

| Logo Type | Folder |
|-----------|--------|
| Main Icon (Orange to Blue) | `MA#1.BI-1.3.4-211107-Main Icon - Orange to Blue - PM OR` |
| Full Logo (Orange) | `MA#1.BI-1.3.1-211107-Full Logo - Orange - PM OR` |
| Side by Side Logo (Orange) | `MA#1.BI-1.3.6-211107-Side by Side Logo - Orange - PM` |

---

## Quick Reference (From Code Implementation)

These colors are already in the Pure Brain codebase (`tailwind.config.ts`):

```css
/* Pure Marketing Brand Colors */
--pure-orange: #f1420b;
--pure-dark: #0a0a0f;
--pure-darker: #050508;
--pure-gray: #1a1a24;
--pure-gray-light: #2a2a3a;
--pure-text: #e5e5e7;
--pure-text-dim: #8b8b95;

/* Gradient */
background: linear-gradient(135deg, #f1420b 0%, #ff6b35 100%);
```

---

## Usage Guidelines

1. **Primary Color:** Pure Orange (#f1420b) for CTAs, highlights, brand elements
2. **Backgrounds:** Dark palette for premium feel
3. **Text:** Light colors on dark backgrounds
4. **Gradients:** Orange gradient for buttons, hero elements
5. **Logo:** Use appropriate variant based on context (icon vs full vs side-by-side)

---

**Note to Agents:** Always check the Google Drive source files for the most current brand assets. The code values above should match, but Google Drive is the source of truth.
