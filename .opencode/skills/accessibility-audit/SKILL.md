---
name: accessibility-audit
description: Audit web accessibility (a11y) per WCAG 2.1/2.2. Use when checking keyboard navigation, screen reader support, color contrast, ARIA, focus management, or inclusive design.
---

# Accessibility Audit Skill

## WCAG 2.1 Levels

| Level | Requirement |
|-------|-------------|
| A | Minimum (must have) |
| AA | Standard (target for most sites) |
| AAA | Enhanced (ideal) |

## Audit Checklist

### 1. Keyboard Navigation

- [ ] All interactive elements focusable via Tab
- [ ] Focus order logical (DOM order)
- [ ] Focus visible (outline/ring)
- [ ] No keyboard traps
- [ ] Skip-to-content link present
- [ ] Custom widgets follow ARIA patterns

### 2. Screen Reader

- [ ] Semantic HTML (headings, landmarks, lists)
- [ ] Alt text for all images
- [ ] ARIA labels for icon buttons
- [ ] Live regions for dynamic content
- [ ] Form labels associated
- [ ] Table headers defined

### 3. Visual

- [ ] Color contrast ≥ 4.5:1 (text) / 3:1 (large text)
- [ ] Don't rely on color alone
- [ ] Text resizable to 200%
- [ ] No flashing content (>3 flashes/sec)
- [ ] Focus indicators visible

### 4. Forms

- [ ] Labels for all inputs
- [ ] Error messages descriptive
- [ ] Required fields indicated
- [ ] Autocomplete attributes set
- [ ] Validation not color-only

### 5. Media

- [ ] Captions for video
- [ ] Transcripts for audio
- [ ] Audio descriptions if needed
- [ ] No auto-playing media

## Common Fixes

### Focus Ring (CSS)

```css
:focus-visible {
  outline: 2px solid #005fcc;
  outline-offset: 2px;
}
```

### Skip Link

```html
<a href="#main" class="skip-link">Skip to main content</a>
```

### ARIA Live Region

```html
<div aria-live="polite" aria-atomic="true" class="sr-only">
  <!-- Dynamic announcements -->
</div>
```

### Reduced Motion

```css
@media (prefers-reduced-motion: reduce) {
  * { animation: none !important; transition: none !important; }
}
```

## Testing Tools

- axe DevTools (browser extension)
- Lighthouse (Accessibility audit)
- NVDA / VoiceOver (screen readers)
- Keyboard-only navigation test
- Color contrast analyzer

## Output Format

```markdown
## Accessibility Audit Report

### WCAG Compliance: Level (A/AA/AAA)

### Critical Issues (Level A failures)
1. [Issue] - [WCAG criterion] - [Fix]

### Improvements (Level AA/AAA)
1. [Issue] - [Recommendation]

### Score: X/100
```
