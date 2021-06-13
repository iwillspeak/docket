// Javascript Progressive Search for Docket
(function (window, document) {
    fetch("./search_index.json")
        .then(response => response.json())
        .then(searchIndex => {

            let searchForm = document.querySelector("#docket-search");
            if (searchForm == null) {
                return;
            }

            searchForm.innerHTML = '<h2>Search</h2><form target="none"><input id="query" autofocus="" placeholder="enter your search" type="search"><button action="submit">Search</button></form><div id="docket-search-results"></div>';
            searchBox = searchForm.querySelector('#query');
            searchResults = searchForm.querySelector('#docket-search-results');

            let searchEntryForResult = function (result) {
                return `<li><a class="search-result" href="${result.slug}/" >${result.title}</a></li>`;
            }

            let displayResults = function (results) {
                if (results.length == 0) {
                    searchResults.innerHTML = "<h3>No results</h3>";
                } else {
                    searchResults.innerHTML =
                        `<h3>${results.length} files match:</h3><ul class="search-results">`
                        + results.map(searchEntryForResult).join('') + '</ul>';
                }
            }

            let doSearch = function (query) {

                // If the search is empty clean up.
                if (query.trim().length == 0) {
                    searchResults.innerHTML = "";
                    return;
                }

                let terms = query.split(/[^\w]/)
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

                // Order them by the score.
                found.sort(f => f.score);

                displayResults(found.map(f => f.page));
            }

            searchForm.addEventListener('submit', function (event) {
                event.preventDefault();
                doSearch(searchBox.value);
            });

            let timer = null;
            searchBox.addEventListener('keyup', function (event) {
                if (timer == null) {
                    timer = setTimeout(function () {
                        timer = null;
                        doSearch(searchBox.value);
                    }, 500);
                }
            })
        });
})(window, document)