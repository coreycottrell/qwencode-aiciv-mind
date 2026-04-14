# Design Specification Template

---

## Design: [Feature/Screen Name]

**Designer:** UI/UX Designer
**Date:** [YYYY-MM-DD]
**Version:** 1.0
**Status:** Draft | In Review | Approved | Implemented

---

## 1. Overview

### 1.1 Purpose
[What user problem does this design solve?]

### 1.2 User Story
**As a** [type of user]
**I want** [goal/desire]
**So that** [benefit/value]

### 1.3 Success Criteria
- [ ] [Criterion 1]
- [ ] [Criterion 2]
- [ ] [Criterion 3]

---

## 2. User Research

### 2.1 User Insights
[Key findings from user research that informed this design]

### 2.2 Personas
| Persona | Needs | Pain Points |
|---------|-------|-------------|
| [Persona 1] | [Needs] | [Pain points] |
| [Persona 2] | [Needs] | [Pain points] |

### 2.3 Competitive Analysis
[What competitors do well/poorly in this area]

---

## 3. User Flow

### 3.1 Flow Diagram
```
[Entry Point]
     │
     ▼
[Step 1: Action]
     │
     ├── [Success] ──→ [Next Step]
     │
     └── [Error] ──→ [Error State]
     │
     ▼
[End State]
```

### 3.2 Entry Points
- [How users arrive at this screen]

### 3.3 Exit Points
- [Where users can go from here]

---

## 4. Wireframes

### 4.1 Desktop View
[Attach wireframe or link to Figma]

### 4.2 Mobile View
[Attach wireframe or link to Figma]

### 4.3 Key Interactions
1. [Interaction 1]: [Description]
2. [Interaction 2]: [Description]

---

## 5. Visual Design

### 5.1 Design Files
- Figma: [Link to Figma file]
- Assets: [Link to exported assets]

### 5.2 Brand Compliance
Reference: `BRAND-ASSETS-GUIDE.md`

| Element | Specification |
|---------|---------------|
| Primary Color | `#f1420b` (Pure Orange) |
| Background | `#0a0a0f` (Pure Dark) |
| Font - Headings | [Font name, weight] |
| Font - Body | [Font name, weight] |

### 5.3 Color Usage
| Color | Hex | Usage |
|-------|-----|-------|
| Primary | `#f1420b` | CTAs, highlights |
| Background | `#0a0a0f` | Page background |
| Surface | `#1a1a24` | Cards, inputs |
| Text Primary | `#e5e5e7` | Main text |
| Text Secondary | `#8b8b95` | Supporting text |

---

## 6. Component Specifications

### 6.1 [Component Name]

**Visual:**
[Screenshot or description]

**Specifications:**
| Property | Value |
|----------|-------|
| Width | [px or %] |
| Height | [px or auto] |
| Padding | [top right bottom left] |
| Margin | [top right bottom left] |
| Border Radius | [px] |
| Background | [color] |
| Shadow | [shadow spec] |

**States:**
| State | Changes |
|-------|---------|
| Default | [Base appearance] |
| Hover | [Changes on hover] |
| Active | [Changes when pressed] |
| Disabled | [Changes when disabled] |
| Loading | [Loading indicator] |
| Error | [Error appearance] |

**Code Hint:**
```css
.component-name {
  /* Key styles for developer reference */
}
```

---

## 7. Responsive Behavior

### 7.1 Breakpoints
| Breakpoint | Width | Layout Changes |
|------------|-------|----------------|
| Mobile | < 640px | [Changes] |
| Tablet | 640px - 1024px | [Changes] |
| Desktop | > 1024px | [Changes] |

### 7.2 Mobile-Specific Considerations
- [Touch targets minimum 44x44px]
- [Bottom navigation for primary actions]
- [Simplified layouts]

---

## 8. Interaction Design

### 8.1 Animations
| Element | Animation | Duration | Easing |
|---------|-----------|----------|--------|
| [Element] | [Type] | [ms] | [easing function] |
| Modal | Fade + Scale | 200ms | ease-out |
| Button hover | Scale | 150ms | ease-in-out |

### 8.2 Transitions
[Describe page/view transitions]

### 8.3 Micro-interactions
[Small feedback interactions - button clicks, toggles, etc.]

---

## 9. Accessibility

### 9.1 Requirements
- [ ] WCAG 2.1 AA compliance
- [ ] Keyboard navigation
- [ ] Screen reader support
- [ ] Color contrast (4.5:1 minimum)
- [ ] Focus indicators

### 9.2 ARIA Labels
| Element | ARIA Label |
|---------|------------|
| [Element] | [Label] |

### 9.3 Tab Order
[Describe logical tab order through the interface]

---

## 10. Content

### 10.1 Copy Requirements
| Element | Copy | Character Limit |
|---------|------|-----------------|
| Heading | [Text] | [Limit] |
| CTA Button | [Text] | [Limit] |
| Error Message | [Text] | [Limit] |

### 10.2 Microcopy
[Helpful text, tooltips, empty states, etc.]

### 10.3 Error Messages
| Error Condition | Message |
|-----------------|---------|
| [Condition] | [User-friendly message] |

---

## 11. Assets Required

### 11.1 Icons
| Icon | Usage | Source |
|------|-------|--------|
| [Icon name] | [Where used] | [Lucide/Custom] |

### 11.2 Images
| Image | Dimensions | Format | Usage |
|-------|------------|--------|-------|
| [Image] | [WxH] | [PNG/SVG/WebP] | [Where used] |

### 11.3 Logo Usage
Reference brand assets in:
`C Level → MA#1.BI-1-221221-Brand Bible → MA#1.BI-1.3-221118-Pure Marketing`

---

## 12. Handoff Notes

### 12.1 Developer Notes
[Important implementation details for developers]

### 12.2 Edge Cases
| Case | Design Decision |
|------|-----------------|
| [Edge case] | [How to handle] |

### 12.3 Out of Scope
[What's intentionally NOT included in this design]

---

## 13. Changelog

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | [Date] | Initial design |
