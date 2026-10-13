// Progressive enhancement for the website. Everything works without JavaScript; this adds
// scroll reveals, the active section in the nav, the mobile menu, and an easter egg.
document.documentElement.classList.add('js');

document.addEventListener('DOMContentLoaded', () => {
    // Reveal elements as they enter the viewport, staggered within each group.
    const revealables = document.querySelectorAll('.reveal');
    if ('IntersectionObserver' in window) {
        const groups = new Map();
        revealables.forEach((el) => {
            const siblings = groups.get(el.parentElement) || 0;
            el.style.setProperty('--i', String(siblings % 4));
            groups.set(el.parentElement, siblings + 1);
        });

        const revealObserver = new IntersectionObserver((entries) => {
            entries.forEach((entry) => {
                if (entry.isIntersecting) {
                    entry.target.classList.add('is-visible');
                    revealObserver.unobserve(entry.target);
                }
            });
        }, { threshold: 0.12, rootMargin: '0px 0px -40px 0px' });

        revealables.forEach((el) => revealObserver.observe(el));

        // Highlight the nav link of the section in view.
        const navLinks = document.querySelectorAll('.nav-links a[href^="#"]');
        const sections = [...navLinks]
            .map((link) => document.getElementById(link.getAttribute('href').slice(1)))
            .filter(Boolean);
        const sectionObserver = new IntersectionObserver((entries) => {
            entries.forEach((entry) => {
                if (!entry.isIntersecting) return;
                navLinks.forEach((link) => {
                    link.classList.toggle('active', link.getAttribute('href') === `#${entry.target.id}`);
                });
            });
        }, { rootMargin: '-40% 0px -55% 0px' });
        sections.forEach((section) => sectionObserver.observe(section));
    } else {
        revealables.forEach((el) => el.classList.add('is-visible'));
    }

    // Mobile menu
    const nav = document.querySelector('.nav');
    const navLinks = document.querySelector('.nav-links');
    if (nav && navLinks) {
        const button = document.createElement('button');
        button.className = 'mobile-menu-btn';
        button.type = 'button';
        button.setAttribute('aria-label', 'Toggle menu');
        button.setAttribute('aria-expanded', 'false');
        button.textContent = '☰';
        nav.insertBefore(button, navLinks);

        const setOpen = (open) => {
            navLinks.classList.toggle('mobile-open', open);
            button.setAttribute('aria-expanded', String(open));
            button.textContent = open ? '✕' : '☰';
        };

        button.addEventListener('click', () => setOpen(!navLinks.classList.contains('mobile-open')));
        navLinks.addEventListener('click', (event) => {
            if (event.target.closest('a')) setOpen(false);
        });
        document.addEventListener('click', (event) => {
            if (!nav.contains(event.target)) setOpen(false);
        });
        document.addEventListener('keydown', (event) => {
            if (event.key === 'Escape') setOpen(false);
        });
    }

    // Konami code easter egg
    const konami = 'ArrowUp,ArrowUp,ArrowDown,ArrowDown,ArrowLeft,ArrowRight,ArrowLeft,ArrowRight,KeyB,KeyA';
    const sequence = [];
    document.addEventListener('keydown', (event) => {
        sequence.push(event.code);
        const combo = sequence.join(',');
        if (!konami.startsWith(combo)) {
            sequence.length = 0;
            return;
        }
        if (combo !== konami) return;
        sequence.length = 0;
        if (document.body.style.animation) return;
        const style = document.createElement('style');
        style.textContent = '@keyframes rainbow { from { filter: hue-rotate(0deg); } to { filter: hue-rotate(360deg); } }';
        document.head.appendChild(style);
        document.body.style.animation = 'rainbow 2s linear infinite';
        setTimeout(() => {
            document.body.style.animation = '';
        }, 10000);
    });
});
