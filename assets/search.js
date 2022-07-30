// # Javascript Progressive Search for Docket
//
// This file loads a search index that's pre-baked by `docket` at documentation
// compile time. If the search index loads we inject the search box into the
// page and enable the search.

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
        return `<li><a class="search-result" href="${rootPath}/${result.slug}/" >${result.title}</a></li>`;
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

    const searchIndex = await fetch(rootPath + "/search_index.json")
        .then(response => response.json());

    const doSearch = query => {

        // If the search is empty clean up.
        if (query.trim().length == 0) {
            searchResults.innerHTML = "";
            return;
        }

        let terms = query.split(/\W+/)
            .map(term => term.trim().toLowerCase())
            .filter(term => term.length > 0);
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

initialiseSearch(document.body.dataset['root'], '#docket-search')