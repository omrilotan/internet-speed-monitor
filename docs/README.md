# Internet Speed Monitor - Documentation Website

This directory contains the documentation website for the Internet Speed Monitor application.

## Files Structure

- `index.html` - Main landing page with features and download links
- `styles.css` - Shared styles for all documentation pages  
- `script.js` - Interactive features and navigation
- `assets/` - Icons and images (moved from root)
- `quick-start/index.html` - Quick start guide for new users
- `user-manual/index.html` - Comprehensive user manual
- `faq/index.html` - Frequently asked questions
- `screenshot.png` - Application screenshot

## Features

- **Responsive Design** - Works on desktop, tablet, and mobile
- **Modern UI** - Clean, professional design following design system principles
- **Interactive Elements** - Smooth scrolling, animations, mobile menu
- **SEO Optimized** - Proper meta tags and structured content
- **Accessibility** - Focus management and semantic HTML
- **Cross-browser Compatible** - Works in all modern browsers

## GitHub Pages Setup

To deploy this documentation to GitHub Pages:

1. Go to your repository settings
2. Navigate to "Pages" section  
3. Select "Deploy from a branch"
4. Choose "main" branch and "/docs" folder
5. Your site will be available at: `https://omrilotan.github.io/internet-speed-monitor/`

## Local Development

To preview locally:

1. Navigate to the docs directory
2. Start a simple HTTP server:
   ```bash
   # Python 3
   python -m http.server 8000
   
   # Python 2
   python -m SimpleHTTPServer 8000
   
   # Node.js (if you have http-server installed)
   npx http-server
   ```
3. Open `http://localhost:8000` in your browser

## Customization

### Colors and Branding
The CSS uses CSS custom properties (variables) for theming. Main colors can be changed in `:root` selector in `styles.css`.

### Content Updates
- Update version numbers in `index.html` and download links
- Add new FAQ items to `faq/index.html`
- Update screenshot by replacing `screenshot.png`

### Adding New Pages
1. Create new HTML file in docs directory
2. Use existing pages as templates for consistent styling
3. Add navigation links to header in each page
4. Update footer links if needed

## Analytics (Optional)

To add Google Analytics or other tracking:

1. Add tracking script to `<head>` section of each page
2. Update download button tracking in `script.js`
3. Consider privacy implications and add privacy policy if needed

## Performance

The website is optimized for performance:
- Minimal dependencies (no external frameworks)
- Optimized images and icons
- CSS and JS minification ready
- Semantic HTML for better SEO

## Browser Support

- Chrome 80+
- Firefox 75+  
- Safari 13+
- Edge 80+

## Contributing

When updating documentation:
1. Test on multiple screen sizes
2. Verify all links work correctly
3. Check spelling and grammar
4. Ensure consistent styling
5. Test accessibility with screen readers
