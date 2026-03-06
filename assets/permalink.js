// Heading permalink — click to copy
// Attaches a click handler to every .heading-anchor so the full URL
// (including the fragment) is copied to the clipboard.

function initPermalinks() {
    document.addEventListener('click', e => {
        const anchor = e.target.closest('.heading-anchor');
        if (!anchor) return;
        e.preventDefault();
        const url = anchor.href;
        if (navigator.clipboard) {
            navigator.clipboard.writeText(url).then(() => {
                const prev = anchor.textContent;
                anchor.textContent = '✓';
                anchor.classList.add('copied');
                setTimeout(() => {
                    anchor.textContent = prev;
                    anchor.classList.remove('copied');
                }, 1500);
            }).catch(() => {
                window.location.href = url;
            });
        } else {
            window.location.href = url;
        }
    });
}

if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initPermalinks);
} else {
    initPermalinks();
}
