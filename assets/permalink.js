// Heading permalink — click to copy
// Attaches a click handler to every .heading-anchor so the full URL
// (including the fragment) is copied to the clipboard.

const moduleUrl = import.meta.url;
const rootPath = moduleUrl.substring(0, moduleUrl.lastIndexOf('/') + 1);

const checkIcon = `<svg class='anchor-icon' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' aria-hidden='true'><use href='${rootPath}feather-sprite.svg#check'/></svg>`;

function initPermalinks() {
    document.addEventListener('click', e => {
        const anchor = e.target.closest('.heading-anchor');
        if (!anchor) return;
        e.preventDefault();
        const url = anchor.href;
        if (navigator.clipboard) {
            navigator.clipboard.writeText(url).then(() => {
                const prev = anchor.innerHTML;
                anchor.innerHTML = checkIcon;
                anchor.classList.add('copied');
                setTimeout(() => {
                    anchor.innerHTML = prev;
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
