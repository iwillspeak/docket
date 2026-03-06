// # Javascript Progressive Search for Docket
//
// This file loads a search index that's pre-baked by `docket` at documentation
// compile time. If the search index loads we inject the search box into the
// page and enable the search.
//
// Query terms are stemmed with the Porter1 algorithm before lookup so that
// they match the Snowball-stemmed keys stored in the index (e.g. searching
// for "compilers" matches pages indexed under the stem "compil").

// ─── Porter1 stemmer ────────────────────────────────────────────────────────
// Public-domain implementation of M.F. Porter, "An algorithm for suffix
// stripping", Program 14(3), 1980, pp. 130-137.
const stemWord = (() => {
    const isVowel = (w, i) => {
        const c = w[i];
        if (c === 'a' || c === 'e' || c === 'i' || c === 'o' || c === 'u') return true;
        if (c === 'y' && i > 0) return !isVowel(w, i - 1);
        return false;
    };

    // Number of consonant-vowel sequences (measure) in a stem.
    const m = w => {
        let count = 0, inVowel = false;
        for (let i = 0; i < w.length; i++) {
            if (isVowel(w, i)) { inVowel = true; }
            else if (inVowel) { count++; inVowel = false; }
        }
        return count;
    };

    const hasVowel = w => { for (let i = 0; i < w.length; i++) if (isVowel(w, i)) return true; return false; };
    const endsDoubleC = w => w.length >= 2 && w[w.length-1] === w[w.length-2] && !isVowel(w, w.length-1);
    const endsCVC = w => {
        const l = w.length;
        return l >= 3 && !isVowel(w,l-1) && isVowel(w,l-2) && !isVowel(w,l-3) && !'wxy'.includes(w[l-1]);
    };

    const step1a = w => {
        if (w.endsWith('sses')) return w.slice(0,-2);
        if (w.endsWith('ies'))  return w.slice(0,-2);
        if (w.endsWith('ss'))   return w;
        if (w.endsWith('s'))    return w.slice(0,-1);
        return w;
    };

    const step1b = w => {
        if (w.endsWith('eed')) { const s=w.slice(0,-3); return m(s)>0 ? s+'ee' : w; }
        let mod = false;
        if (w.endsWith('ed'))  { const s=w.slice(0,-2); if (hasVowel(s)) { w=s; mod=true; } }
        else if (w.endsWith('ing')) { const s=w.slice(0,-3); if (hasVowel(s)) { w=s; mod=true; } }
        if (mod) {
            if (w.endsWith('at') || w.endsWith('bl') || w.endsWith('iz')) return w+'e';
            if (endsDoubleC(w) && !w.endsWith('l') && !w.endsWith('s') && !w.endsWith('z')) return w.slice(0,-1);
            if (m(w)===1 && endsCVC(w)) return w+'e';
        }
        return w;
    };

    const step1c = w => (w.endsWith('y') && hasVowel(w.slice(0,-1))) ? w.slice(0,-1)+'i' : w;

    const applyRule = (w, sfx, rep, minM) => {
        if (!w.endsWith(sfx)) return null;
        const s = w.slice(0, -sfx.length);
        return m(s) > minM-1 ? s+rep : null;
    };

    const step2 = w => {
        for (const [s,rep] of [
            ['ational','ate'],['tional','tion'],['enci','ence'],['anci','ance'],
            ['izer','ize'],['abli','able'],['alli','al'],['entli','ent'],['eli','e'],
            ['ousli','ous'],['ization','ize'],['ation','ate'],['ator','ate'],
            ['alism','al'],['iveness','ive'],['fulness','ful'],['ousness','ous'],
            ['aliti','al'],['iviti','ive'],['biliti','ble'],
        ]) { const result=applyRule(w,s,rep,1); if (result!==null) return result; }
        return w;
    };

    const step3 = w => {
        for (const [s,rep] of [
            ['icate','ic'],['ative',''],['alize','al'],['iciti','ic'],['ical','ic'],['ful',''],['ness',''],
        ]) { const result=applyRule(w,s,rep,1); if (result!==null) return result; }
        return w;
    };

    const step4 = w => {
        for (const s of ['al','ance','ence','er','ic','able','ible','ant','ement','ment','ent','ou','ism','ate','iti','ous','ive','ize']) {
            if (w.endsWith(s)) { const stem=w.slice(0,-s.length); if (m(stem)>1) return stem; }
        }
        // 'ion' requires preceding 's' or 't'
        if (w.endsWith('ion')) { const stem=w.slice(0,-3); if (m(stem)>1 && (stem.endsWith('s')||stem.endsWith('t'))) return stem; }
        return w;
    };

    const step5a = w => {
        if (w.endsWith('e')) {
            const s=w.slice(0,-1);
            if (m(s)>1 || (m(s)===1 && !endsCVC(s))) return s;
        }
        return w;
    };

    const step5b = w => (w.endsWith('ll') && m(w.slice(0,-1))>1) ? w.slice(0,-1) : w;

    return w => {
        if (w.length < 3) return w;
        return step5b(step5a(step4(step3(step2(step1c(step1b(step1a(w))))))));
    };
})();
// ─────────────────────────────────────────────────────────────────────────────

const initialiseSearch = async (rootPath, targetSelector) => {
    const searchForm = document.querySelector(targetSelector);
    if (searchForm == null) {
        return;
    }

    searchForm.innerHTML = `<h2>Search</h2>
    <form target="none">
        <input id="query" autofocus="" placeholder="enter your search" type="search">
        <button action="submit">Search</button>
    </form>
    <div id="docket-search-results"></div>`;
    const searchBox = searchForm.querySelector('#query');
    const searchResults = searchForm.querySelector('#docket-search-results');

    const searchEntryForResult = result => {
        return `<li><a class="search-result" href="${rootPath}${result.slug}/" >${result.title}</a></li>`;
    }

    const displayResults = results => {
        if (results.length == 0) {
            searchResults.innerHTML = "<h3>No results</h3>";
        } else {
            searchResults.innerHTML =
                `<h3>${results.length} results</h3>
                <ul class="search-results">
                    ${results.map(searchEntryForResult).join('')}
                </ul>`;
        }
    }

    const searchIndex = await fetch(`${rootPath}search_index.json`)
        .then(response => response.json());

    const doSearch = query => {

        // If the search is empty clean up.
        if (query.trim().length == 0) {
            searchResults.innerHTML = "";
            return;
        }

        // Split, lowercase, filter short tokens, then stem to match the index.
        let terms = query.split(/\W+/)
            .map(term => term.trim().toLowerCase())
            .filter(term => term.length >= 3)
            .map(term => stemWord(term));
        let found = []

        searchIndex.forEach(page => {
            let score = 0;
            terms.forEach(term => {
                let termScore = page.terms[term];
                if (termScore !== undefined) {
                    score += termScore;
                }
            });
            if (score > 0) {
                found.push({
                    score: score,
                    page: page,
                });
            }
        });

        // Order them by the score, descending.
        found.sort((a, b) => b.score - a.score);

        displayResults(found.map(f => f.page));
    }

    let timer = null;
    searchBox.addEventListener('keyup', event => {
        if (timer !== null) {
            clearTimeout(timer);
        }
        timer = setTimeout(() => {
            timer = null;
            doSearch(searchBox.value);
        }, 500);
    });

    searchForm.addEventListener('submit', event => {
        event.preventDefault();
        if (timer !== null) {
            clearTimeout(timer);
            timer = null;
        }
        doSearch(searchBox.value);
    });
}

const uri = import.meta.url;
initialiseSearch(uri.substring(0, uri.lastIndexOf('/') + 1), '#docket-search')