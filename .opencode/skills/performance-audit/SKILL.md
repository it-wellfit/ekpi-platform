---
name: performance-audit
description: Audit and optimize web application performance. Use when reviewing load times, rendering bottlenecks, bundle size, Core Web Vitals, or scalability concerns.
---

# Performance Audit Skill

## When to Use

- Page load time > 3 seconds
- Poor Core Web Vitals (LCP, CLS, INP)
- Large bundle sizes
- Memory leaks or jank
- Scaling to larger datasets

## Audit Checklist

### 1. Core Web Vitals

| Metric | Good | Needs Work | Poor |
|--------|------|------------|------|
| LCP (Largest Contentful Paint) | ≤2.5s | 2.5-4.0s | >4.0s |
| CLS (Cumulative Layout Shift) | ≤0.1 | 0.1-0.25 | >0.25 |
| INP (Interaction to Next Paint) | ≤200ms | 200-500ms | >500ms |

### 2. Bundle Analysis

- [ ] Check bundle size with `bun run build` + analyze
- [ ] Identify largest dependencies
- [ ] Check for duplicate packages
- [ ] Verify tree-shaking works
- [ ] Check for unused code

### 3. Rendering Performance

- [ ] Minimize re-renders (React.memo, useMemo, useCallback)
- [ ] Virtualize long lists
- [ ] Lazy load below-fold content
- [ ] Use CSS containment
- [ ] Avoid layout thrashing

### 4. Network Optimization

- [ ] Compress assets (gzip/brotli)
- [ ] Use CDN for static assets
- [ ] Implement caching headers
- [ ] Prefetch critical resources
- [ ] Use WebP/AVIF for images

### 5. Memory & Runtime

- [ ] Check for memory leaks (detached DOM, closures)
- [ ] Profile with Chrome DevTools
- [ ] Avoid unnecessary object creation
- [ ] Clean up event listeners

## Quick Fixes

1. **Image optimization**: Convert to WebP, use srcset, lazy load
2. **Code splitting**: Dynamic imports for routes
3. **Font optimization**: Preload, font-display: swap
4. **CSS**: Remove unused, inline critical
5. **JS**: Defer non-critical, async third-party

## Tools

- Chrome DevTools Performance tab
- Lighthouse
- WebPageTest
- Bundle analyzer (webpack-bundle-analyzer / vite-bundle-visualizer)

## Output Format

```markdown
## Performance Audit Report

### Summary
- LCP: Xs (Good/Needs Work/Poor)
- CLS: X (Good/Needs Work/Poor)
- INP: Xms (Good/Needs Work/Poor)

### Critical Issues
1. [Issue] - [Impact] - [Fix]

### Recommendations
1. [Priority] [Action] - [Expected improvement]

### Estimated Impact
- Load time: Xs → Ys (-Z%)
- Bundle size: XKB → YKB (-Z%)
```
