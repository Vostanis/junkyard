// Enhanced search functionality with backend data
document.addEventListener('DOMContentLoaded', () => {
    const searchInput = document.getElementById('searchInput');
    const searchResults = document.getElementById('searchResults');
    const resultsList = document.getElementById('resultsList');
    
    if (!searchInput || !resultsList) return;
    
    // Initialize Fuse.js for fuzzy search with improved settings
    const fuseOptions = {
        includeScore: true,
        includeMatches: true,
        threshold: 0.4,       // Balanced threshold for good matches
        keys: ['symbol', 'title', 'industry'],
        location: 0,
        distance: 150,        // Increased distance for better fuzzy matching
        minMatchCharLength: 1,
        shouldSort: true,
        findAllMatches: true,
        limit: 30             // Show more results
    };
    
    const fuse = new Fuse(stockData, fuseOptions);
    let currentIndex = -1;
    let selectedItem = null;
    let typingTimer;       // Timer identifier for debouncing
    
    // Set up event listeners
    searchInput.addEventListener('input', debounceSearch);
    searchInput.addEventListener('keydown', handleKeyNavigation);
    
    // Add keyboard shortcut for search focus like on home page
    document.addEventListener('keydown', function(e) {
        // Check if the pressed key is '/' and no input field is currently focused
        if (e.key === '/' && document.activeElement.tagName !== 'INPUT') {
            e.preventDefault(); // Prevent the '/' character from being typed
            searchInput.focus(); // Focus on the search input
        }
    });
    
    // Hide search results when clicking outside
    document.addEventListener('click', function(e) {
        if (!searchInput.contains(e.target) && !searchResults.contains(e.target)) {
            searchResults.classList.remove('visible');
            currentIndex = -1;
            selectedItem = null;
        }
    });
    
    // Debounce search to prevent too many searches while typing
    function debounceSearch() {
        clearTimeout(typingTimer);
        typingTimer = setTimeout(performSearch, 150); // Wait 150ms before searching
    }
    
    function performSearch() {
        const query = searchInput.value.trim();
        
        if (!query) {
            searchResults.classList.remove('visible');
            resultsList.innerHTML = '';
            currentIndex = -1;
            selectedItem = null;
            return;
        }
        
        const results = fuse.search(query);
        
        if (results.length === 0) {
            resultsList.innerHTML = '<li>No matches found</li>';
        } else {
            resultsList.innerHTML = '<li class="label">Search Results</li>';
            
            // Create category grouping
            const groupedResults = {
                symbols: [],
                companies: [],
                industries: []
            };
            
            // Group results by match type
            results.slice(0, 30).forEach(result => {
                if (result.matches.some(match => match.key === 'symbol' && match.indices[0][0] === 0)) {
                    groupedResults.symbols.push(result);
                } else if (result.matches.some(match => match.key === 'title')) {
                    groupedResults.companies.push(result);
                } else if (result.matches.some(match => match.key === 'industry')) {
                    groupedResults.industries.push(result);
                } else {
                    groupedResults.companies.push(result); // Default group
                }
            });
            
            // First display exact symbol matches
            if (groupedResults.symbols.length > 0) {
                addResultsToList(groupedResults.symbols, 'Top Matches');
            }
            
            // Then display company name matches
            if (groupedResults.companies.length > 0) {
                addResultsToList(groupedResults.companies, 'Companies');
            }
            
            // Finally display industry matches
            if (groupedResults.industries.length > 0) {
                addResultsToList(groupedResults.industries, 'Industries');
            }
        }
        
        searchResults.classList.add('visible');
        currentIndex = -1;
    }
    
    function addResultsToList(results, categoryLabel) {
        if (resultsList.children.length > 1) { // If we already have content, add a separator
            const separator = document.createElement('li');
            separator.className = 'label';
            separator.textContent = categoryLabel;
            resultsList.appendChild(separator);
        }
        
        results.forEach((result) => {
            const el = document.createElement('li');
            
            // Highlight matching parts if available
            let displaySymbol = result.item.symbol;
            let displayTitle = result.item.title;
            let displayIndustry = result.item.industry || '';
            
            if (result.matches) {
                // Apply highlighting to matched parts
                result.matches.forEach(match => {
                    if (match.key === 'symbol') {
                        displaySymbol = highlightMatches(result.item.symbol, match.indices);
                    } else if (match.key === 'title') {
                        displayTitle = highlightMatches(result.item.title, match.indices);
                    } else if (match.key === 'industry' && result.item.industry) {
                        displayIndustry = highlightMatches(result.item.industry, match.indices);
                    }
                });
            }
            
            el.innerHTML = `
                <strong>[${displaySymbol}]</strong> ${displayTitle} <weak>${displayIndustry}</weak>
            `;
            
            el.dataset.symbol = result.item.symbol;
            
            el.addEventListener('click', () => {
                handleStockSelection(result.item.symbol);
            });
            
            resultsList.appendChild(el);
        });
    }
    
    function highlightMatches(text, indices) {
        if (!indices || indices.length === 0) return text;
        
        const chars = text.split('');
        // Create a map of positions to highlight
        const positions = new Set();
        indices.forEach(([start, end]) => {
            for (let i = start; i <= end; i++) {
                positions.add(i);
            }
        });
        
        // Apply highlighting
        let result = '';
        chars.forEach((char, index) => {
            if (positions.has(index)) {
                result += `<span style="color: var(--accent-blue, #58b2dc); font-weight: bold;">${char}</span>`;
            } else {
                result += char;
            }
        });
        
        return result;
    }
    
    function handleKeyNavigation(e) {
        const items = Array.from(resultsList.querySelectorAll("li:not(.label)"));
        
        if (items.length === 0) return;
        
        if (e.key === "ArrowDown") {
            e.preventDefault();
            currentIndex++;
            if (currentIndex >= items.length) currentIndex = 0;
            updateHighlight(items);
            selectedItem = items[currentIndex];
        } else if (e.key === "ArrowUp") {
            e.preventDefault();
            currentIndex--;
            if (currentIndex < 0) currentIndex = items.length - 1;
            updateHighlight(items);
            selectedItem = items[currentIndex];
        } else if (e.key === "Enter") {
            if (selectedItem) {
                e.preventDefault();
                const symbol = selectedItem.dataset.symbol;
                handleStockSelection(symbol);
            } else if (searchInput.value.trim()) {
                // If no item is selected but we have a search term, treat it as a direct symbol search
                e.preventDefault();
                handleStockSelection(searchInput.value.trim().toUpperCase());
            }
        } else if (e.key === "Escape") {
            searchResults.classList.remove('visible');
            currentIndex = -1;
            selectedItem = null;
        }
    }
    
    function updateHighlight(items) {
        items.forEach((item, index) => {
            item.classList.toggle('highlighted', index === currentIndex);
            if (index === currentIndex) {
                item.scrollIntoView({ block: "nearest" });
            }
        });
    }
    
    function handleStockSelection(symbol) {
        searchInput.value = symbol;
        searchResults.classList.remove('visible');
        console.log(`Selected stock: ${symbol}`);
        
        // Redirect to the stock page
        window.location.href = `/asset/${symbol}`;
    }
    
    // If the user arrives with a symbol already in the search box, make it functional
    if (searchInput.value.trim()) {
        performSearch();
    }
});
