// Mobile navigation drawer toggle
// Handles the sidebar (nav tree) and TOC drawer on narrow viewports.

function initToggles() {
    const navToggle = document.getElementById('nav-toggle');
    const tocToggle = document.getElementById('toc-toggle');
    const sidebar = document.querySelector('.sidebar');
    const tocTree = document.querySelector('.toc-tree');

    if (!navToggle && !tocToggle) return;

    // Backdrop element for closing drawers by clicking outside
    const backdrop = document.createElement('div');
    backdrop.className = 'nav-backdrop';
    backdrop.setAttribute('aria-hidden', 'true');
    document.body.appendChild(backdrop);

    function openPanel(panel, button) {
        panel.classList.add('open');
        button.setAttribute('aria-expanded', 'true');
        backdrop.classList.add('active');
        document.body.classList.add('drawer-open');
        // Move focus into the panel for accessibility
        const firstFocusable = panel.querySelector('a, button, input, [tabindex]');
        firstFocusable?.focus();
    }

    function closeAll() {
        sidebar?.classList.remove('open');
        tocTree?.classList.remove('open');
        navToggle?.setAttribute('aria-expanded', 'false');
        tocToggle?.setAttribute('aria-expanded', 'false');
        backdrop.classList.remove('active');
        document.body.classList.remove('drawer-open');
    }

    navToggle?.addEventListener('click', () => {
        if (sidebar?.classList.contains('open')) {
            closeAll();
        } else {
            closeAll();
            openPanel(sidebar, navToggle);
        }
    });

    tocToggle?.addEventListener('click', () => {
        if (tocTree?.classList.contains('open')) {
            closeAll();
        } else {
            closeAll();
            openPanel(tocTree, tocToggle);
        }
    });

    backdrop.addEventListener('click', closeAll);

    // In-panel close buttons
    sidebar?.querySelector('.drawer-close')?.addEventListener('click', closeAll);
    tocTree?.querySelector('.drawer-close')?.addEventListener('click', closeAll);

    document.addEventListener('keydown', e => {
        if (e.key === 'Escape') closeAll();
    });

    // Close drawers when a nav link is followed (page won't reload on same-page
    // anchors, so close the drawer so the target is visible)
    sidebar?.addEventListener('click', e => {
        if (e.target.closest('a')) closeAll();
    });
    tocTree?.addEventListener('click', e => {
        if (e.target.closest('a')) closeAll();
    });

    // On resize to desktop, clear any open state so the layout resets cleanly
    const desktopMq = window.matchMedia('(min-width: 980px)');
    desktopMq.addEventListener('change', e => {
        if (e.matches) closeAll();
    });

    // On resize below tablet, the TOC moves back to a drawer — reset open state
    const tabletMq = window.matchMedia('(min-width: 500px)');
    tabletMq.addEventListener('change', e => {
        if (!e.matches) closeAll();
    });
}

if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initToggles);
} else {
    initToggles();
}
