---
name: image-optimization
description: Optimize images for web: format selection, compression, responsive sizing, lazy loading, and art direction. Use when adding images, improving page speed, or fixing image-related layout shifts.
---

# Image Optimization Skill

## Format Selection

| Format | Use Case | Browser Support |
|--------|----------|-----------------|
| WebP | Photos, general | 97%+ |
| AVIF | Photos (best compression) | 93%+ |
| SVG | Icons, logos, illustrations | 100% |
| PNG | Screenshots, transparency needed | 100% |
| JPEG | Fallback for photos | 100% |

## Compression Guidelines

| Image Type | Quality | Max Size |
|------------|---------|----------|
| Hero image | 80-85% | 200KB |
| Content image | 75-80% | 100KB |
| Thumbnail | 70-75% | 30KB |
| Icon | SVG (lossless) | 5KB |

## Responsive Images

```html
<img
  src="image-800.webp"
  srcset="image-400.webp 400w, image-800.webp 800w, image-1200.webp 1200w"
  sizes="(max-width: 600px) 400px, (max-width: 900px) 800px, 1200px"
  alt="Description"
  loading="lazy"
  decoding="async"
  width="800"
  height="600"
/>
```

## Checklist

- [ ] Use modern formats (WebP/AVIF) with fallback
- [ ] Provide responsive srcset
- [ ] Set explicit width/height (prevent CLS)
- [ ] Use loading="lazy" for below-fold
- [ ] Use loading="eager" for above-fold (LCP)
- [ ] Compress to target quality
- [ ] Use CDN with auto-optimization
- [ ] Consider art direction (picture element)

## Art Direction with picture

```html
<picture>
  <source media="(max-width: 600px)" srcset="mobile.webp" />
  <source media="(min-width: 601px)" srcset="desktop.webp" />
  <img src="fallback.jpg" alt="Description" />
</picture>
```

## Common Mistakes

1. **No width/height** → CLS (layout shift)
2. **Oversized images** → slow load
3. **Wrong format** → bloated files
4. **No lazy loading** → wasted bandwidth
5. **No alt text** → accessibility failure
6. **Base64 inline** → bloats HTML

## Tools

- Squoosh (squoosh.app) - browser-based
- Sharp (Node/Bun) - programmatic
- ImageOptim - macOS
- TinyPNG - online
- Cloudinary/Imgix - CDN with auto-optimization
