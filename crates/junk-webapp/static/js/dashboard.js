/**
 * Main dashboard functionality with fixes
 */
document.addEventListener('DOMContentLoaded', () => {
    console.log('Chart.js loaded:', typeof Chart !== 'undefined');
    console.log('Annotation plugin loaded:', typeof Chart.Annotation !== 'undefined');
    console.log('Fuse.js loaded:', typeof Fuse !== 'undefined');

    // Hide search results on initial load
    const searchResults = document.getElementById('searchResults');
    if (searchResults) {
        searchResults.style.display = 'none';
    }

    // Setup prices data
    let pricesData;
    let financialsData;
    
    try {
        // If data was provided from the backend via script tag
        if (typeof prices !== 'undefined' && typeof financials !== 'undefined') {
            pricesData = prices;
            financialsData = financials;
        } else {
            // Fallback data for testing
            pricesData = [
                { date: '2020-01-02', adj_close: 140.20, adj_close_20ma: 138.50, adj_close_50ma: 135.00, adj_close_200ma: 130.00, volume: 350000000 },
                { date: '2020-01-03', adj_close: 145.60, adj_close_20ma: 139.00, adj_close_50ma: 136.00, adj_close_200ma: 131.00, volume: 280000000 },
                { date: '2020-01-06', adj_close: 148.20, adj_close_20ma: 140.00, adj_close_50ma: 137.00, adj_close_200ma: 132.00, volume: 300000000 },
                { date: '2020-01-07', adj_close: 147.80, adj_close_20ma: 141.00, adj_close_50ma: 138.00, adj_close_200ma: 133.00, volume: 250000000 },
                { date: '2025-02-01', adj_close: 280.00, adj_close_20ma: 260.00, adj_close_50ma: 240.00, adj_close_200ma: 200.00, volume: 150000000 }
            ];
            
            financialsData = [
                { end_date: '2024-10-18', accumulated_earnings: null, debt_to_equity: 0, earnings: null, earnings_perc: 0, revenue: null, debt: 110000000000, equity: 58000000000, assets: 168000000000, market_cap: 450000000000, shares_outstanding: 13500000000, float: 10000000000, value_of_shares_bought_back: 500000000, gross_profit: 35000000000, operating_income: 24000000000, eps: 1.58 },
                { end_date: '2024-09-28', accumulated_earnings: -19154000000, debt_to_equity: 1.872326602282704, earnings: null, earnings_perc: 0, revenue: null, debt: 108000000000, equity: 57000000000, assets: 165000000000, market_cap: 420000000000, shares_outstanding: 13600000000, float: 10100000000, value_of_shares_bought_back: 450000000, gross_profit: 34000000000, operating_income: 23000000000, eps: 1.52 },
                { end_date: '2024-07-19', accumulated_earnings: null, debt_to_equity: 0, earnings: null, earnings_perc: 0, revenue: null, debt: 106000000000, equity: 55000000000, assets: 161000000000, market_cap: 410000000000, shares_outstanding: 13700000000, float: 10200000000, value_of_shares_bought_back: 400000000, gross_profit: 33000000000, operating_income: 22500000000, eps: 1.49 },
                { end_date: '2024-06-29', accumulated_earnings: -4726000000, debt_to_equity: 1.5186184565569347, earnings: 21448000000, earnings_perc: 0.25004371801298714, revenue: 85783000000, debt: 104000000000, equity: 52000000000, assets: 156000000000, market_cap: 400000000000, shares_outstanding: 13800000000, float: 10300000000, value_of_shares_bought_back: 350000000, gross_profit: 31500000000, operating_income: 22000000000, eps: 1.46 },
                { end_date: '2024-04-19', accumulated_earnings: null, debt_to_equity: 0, earnings: null, earnings_perc: 0, revenue: null, debt: 102000000000, equity: 50000000000, assets: 152000000000, market_cap: 390000000000, shares_outstanding: 14000000000, float: 10400000000, value_of_shares_bought_back: 300000000, gross_profit: 31000000000, operating_income: 21500000000, eps: 1.42 },
                { end_date: '2024-03-30', accumulated_earnings: 4339000000, debt_to_equity: 1.4096827236703777, earnings: 23636000000, earnings_perc: 0.26044318094167684, revenue: 90753000000, debt: 100000000000, equity: 48000000000, assets: 148000000000, market_cap: 380000000000, shares_outstanding: 14500000000, float: 10500000000, value_of_shares_bought_back: 200000000, gross_profit: 32000000000, operating_income: 22300000000, eps: 1.56 }
            ];
        }
        
        console.log('Data loaded successfully');
    } catch (e) {
        console.error('Error parsing data:', e);
        
        // Set fallback data if parsing failed
        pricesData = [];
        financialsData = [];
    }
    
    // Sort and prepare price data
    pricesData.sort((a, b) => new Date(b.date) - new Date(a.date));
    const fullLabels = pricesData.map(item => item.date).reverse();
    const fullPriceData = pricesData.map(item => item.adj_close).reverse();
    const fullPrice20MAData = pricesData.map(item => item.adj_close_20ma).reverse();
    const fullPrice50MAData = pricesData.map(item => item.adj_close_50ma).reverse();
    const fullPrice200MAData = pricesData.map(item => item.adj_close_200ma).reverse();
    const fullVolumeData = pricesData.map(item => item.volume || 0).reverse();

    // Sort and prepare financials data
    financialsData.sort((a, b) => new Date(b.end_date) - new Date(a.end_date));
    const finLabels = financialsData.map(item => item.end_date).reverse();

    // Extract and process financials data
    const rawFinRevenueData = financialsData.map(item => item.revenue ? item.revenue / 1e9 : null).reverse();
    const rawFinEarningsData = financialsData.map(item => item.earnings ? item.earnings / 1e9 : null).reverse();
    const rawFinAccumulatedEarningsData = financialsData.map(item => item.accumulated_earnings ? item.accumulated_earnings / 1e9 : null).reverse();
    const rawFinDebtToEquityData = financialsData.map(item => item.debt_to_equity || null).reverse();
    const rawFinEarningsPercData = financialsData.map(item => item.earnings_perc * 100 || null).reverse();
    
    // Debt & Equity data
    const rawFinDebtData = financialsData.map(item => item.debt ? item.debt / 1e9 : null).reverse();
    const rawFinEquityData = financialsData.map(item => item.equity ? item.equity / 1e9 : null).reverse();
    const rawFinAssetsData = financialsData.map(item => item.assets ? item.assets / 1e9 : null).reverse();
    
    // Market Mechanics data
    const rawFinMarketCapData = financialsData.map(item => item.market_cap ? item.market_cap / 1e9 : null).reverse();
    const rawFinSharesOutstandingData = financialsData.map(item => item.shares_outstanding ? item.shares_outstanding / 1e6 : null).reverse();
    const rawFinFloatData = financialsData.map(item => item.float ? item.float / 1e6 : null).reverse();
    const rawFinSharesBoughtBackData = financialsData.map(item => item.value_of_shares_bought_back ? item.value_of_shares_bought_back / 1e6 : null).reverse();
    
    // Earnings panel data
    const rawFinGrossProfitData = financialsData.map(item => item.gross_profit ? item.gross_profit / 1e9 : null).reverse();
    const rawFinOperatingIncomeData = financialsData.map(item => item.operating_income ? item.operating_income / 1e9 : null).reverse();
    const rawFinEPSData = financialsData.map(item => item.eps ? item.eps : null).reverse();

    // Interpolate data to fill gaps
    const finRevenueData = interpolateData(rawFinRevenueData);
    const finEarningsData = interpolateData(rawFinEarningsData);
    const finAccumulatedEarningsData = interpolateData(rawFinAccumulatedEarningsData);
    const finDebtToEquityData = interpolateData(rawFinDebtToEquityData);
    const finEarningsPercData = interpolateData(rawFinEarningsPercData);
    
    // Interpolated data for Debt & Equity tab
    const finDebtData = interpolateData(rawFinDebtData);
    const finEquityData = interpolateData(rawFinEquityData);
    const finAssetsData = interpolateData(rawFinAssetsData);
    
    // Interpolated data for Market Mechanics tab
    const finMarketCapData = interpolateData(rawFinMarketCapData);
    const finSharesOutstandingData = interpolateData(rawFinSharesOutstandingData);
    const finFloatData = interpolateData(rawFinFloatData);
    const finSharesBoughtBackData = interpolateData(rawFinSharesBoughtBackData);
    
    // Interpolated data for the earnings right panels
    const finGrossProfitData = interpolateData(rawFinGrossProfitData);
    const finOperatingIncomeData = interpolateData(rawFinOperatingIncomeData);
    const finEPSData = interpolateData(rawFinEPSData);

    // Initial data filtering
    let currentData = filterDataByRange(pricesData, 'ALL');
    
    // Theme colors
    const theme = getThemeColors();

    // Chart element references
    const priceVolumeCtx = document.getElementById('priceVolumeChart')?.getContext('2d');
    const earningsCtx = document.getElementById('earningsChart')?.getContext('2d');
    const operatingProfitCtx = document.getElementById('operatingProfitChart')?.getContext('2d');
    const epsCtx = document.getElementById('epsChart')?.getContext('2d');
    const debtCtx = document.getElementById('debtChart')?.getContext('2d');
    const marketCapCtx = document.getElementById('marketCapChart')?.getContext('2d');
    const sharesOutstandingCtx = document.getElementById('sharesOutstandingChart')?.getContext('2d');
    const floatCtx = document.getElementById('floatChart')?.getContext('2d');
    const sharesBoughtBackCtx = document.getElementById('sharesBoughtBackChart')?.getContext('2d');

    // Initialize charts array
    const charts = [];
    
    // Only create charts if context is available
    if (priceVolumeCtx) {
        // Create Price Volume chart
        const priceVolumeChart = createPriceVolumeChart(priceVolumeCtx, currentData, theme);
        charts.push(priceVolumeChart);
    }
    
    // Create Earnings chart if context exists
    let earningsChart;
    if (earningsCtx) {
        earningsChart = createEarningsChart(
            earningsCtx, 
            finLabels, 
            finRevenueData, 
            finEarningsData, 
            finAccumulatedEarningsData, 
            theme
        );
        charts.push(earningsChart);
    }
    
    // Create Operating Profit chart if context exists
    let operatingProfitChart;
    if (operatingProfitCtx) {
        operatingProfitChart = createOperatingProfitChart(
            operatingProfitCtx, 
            finLabels, 
            finGrossProfitData, 
            finOperatingIncomeData, 
            finEarningsData, 
            theme
        );
    }
    
    // Create EPS chart if context exists
    let epsChart;
    if (epsCtx) {
        epsChart = createEPSChart(epsCtx, finLabels, finEPSData, theme);
    }
    
    // Earnings charts array
    const earningsCharts = [operatingProfitChart, epsChart].filter(chart => chart !== undefined);
    
    // Create Debt & Equity chart if context exists
    let debtChart;
    if (debtCtx) {
        debtChart = createDebtChart(
            debtCtx, 
            finLabels, 
            finDebtData, 
            finEquityData, 
            finAssetsData, 
            theme
        );
        charts.push(debtChart);
    }
    
    // Create Market Mechanics charts if contexts exist
    let marketCapChart, sharesOutstandingChart, floatChart, sharesBoughtBackChart;
    
    if (marketCapCtx) {
        marketCapChart = createMarketCapChart(marketCapCtx, finLabels, finMarketCapData, theme);
    }
    
    if (sharesOutstandingCtx) {
        sharesOutstandingChart = createSharesOutstandingChart(sharesOutstandingCtx, finLabels, finSharesOutstandingData, theme);
    }
    
    if (floatCtx) {
        floatChart = createFloatChart(floatCtx, finLabels, finFloatData, theme);
    }
    
    if (sharesBoughtBackCtx) {
        sharesBoughtBackChart = createSharesBoughtBackChart(sharesBoughtBackCtx, finLabels, finSharesBoughtBackData, theme);
    }
    
    // Market Mechanics charts array - filter out undefined charts
    const marketMechanicsCharts = [marketCapChart, sharesOutstandingChart, floatChart, sharesBoughtBackChart].filter(chart => chart !== undefined);

    // Date Range Handling
    const dateRangeSelector = document.getElementById('dateRange');
    if (dateRangeSelector) {
        dateRangeSelector.addEventListener('change', function() {
            const selectedRange = this.value;
            currentData = filterDataByRange(pricesData, selectedRange);
            
            if (priceVolumeChart) {
                priceVolumeChart.data.labels = currentData.labels;
                priceVolumeChart.data.datasets[0].data = currentData.priceData;
                priceVolumeChart.data.datasets[1].data = currentData.price20MAData;
                priceVolumeChart.data.datasets[2].data = currentData.price50MAData;
                priceVolumeChart.data.datasets[3].data = currentData.price200MAData;
                priceVolumeChart.data.datasets[4].data = currentData.volumeData;
                
                priceVolumeChart.options.scales['y-volume'].max = Math.max(...currentData.volumeData) * 5;
                priceVolumeChart.options.scales['y-price'].suggestedMin = Math.min(...currentData.priceData) * 0.9;
                priceVolumeChart.options.scales['y-price'].suggestedMax = Math.max(...currentData.priceData) * 1.1;
                
                priceVolumeChart.update();
            }
        });
    }

    // Chart Switching Logic
    let activeChartIndex = 0;
    const chartElements = [
        document.getElementById('priceVolumeChart'),
        document.getElementById('earningsContainer'),
        document.getElementById('debtChart'),
        document.getElementById('marketMechanicsContainer')
    ].filter(el => el !== null); // Filter out null elements
    
    const controlsBar = document.querySelector('.controls-bar');
    const prevButton = document.getElementById('prevChart');
    const nextButton = document.getElementById('nextChart');
    const tabs = document.querySelectorAll('.tab');

    function updateChartVisibility() {
        if (chartElements.length === 0) return; // Skip if no chart elements found
        
        chartElements.forEach((element, index) => {
            if (element) {
                element.classList.toggle('active', index === activeChartIndex);
                element.classList.toggle('hidden', index !== activeChartIndex);
            }
        });
        
        tabs.forEach((tab, index) => tab.classList.toggle('active', index === activeChartIndex));
        if (controlsBar) {
            controlsBar.classList.toggle('active', activeChartIndex === 0);
        }
        
        // Update visible charts with a slight delay to ensure proper rendering
        setTimeout(() => {
            if (activeChartIndex === 0) {
                if (charts[activeChartIndex]) charts[activeChartIndex].resize();
            } else if (activeChartIndex === 1) {
                // For Earnings tab, resize all earnings charts
                if (charts[activeChartIndex]) charts[activeChartIndex].resize();
                earningsCharts.forEach(chart => {
                    if (chart && typeof chart.resize === 'function') {
                        chart.resize();
                    }
                });
            } else if (activeChartIndex === 2) {
                if (charts[activeChartIndex]) charts[activeChartIndex].resize();
            } else if (activeChartIndex === 3) {
                // For Market Mechanics, resize all sub-charts
                marketMechanicsCharts.forEach(chart => {
                    if (chart && typeof chart.resize === 'function') {
                        chart.resize();
                    }
                });
            }
        }, 50);
    }

    // Initialize chart visibility if elements exist
    if (chartElements.length > 0) {
        updateChartVisibility();
    }

    // Setup tab navigation
    tabs.forEach(tab => {
        tab.addEventListener('click', function() {
            activeChartIndex = parseInt(this.getAttribute('data-chart-index'));
            updateChartVisibility();
        });
    });

    // Button navigation
    if (nextButton) {
        nextButton.addEventListener('click', function() {
            if (chartElements.length === 0) return;
            activeChartIndex = (activeChartIndex + 1) % chartElements.length;
            updateChartVisibility();
        });
    }

    if (prevButton) {
        prevButton.addEventListener('click', function() {
            if (chartElements.length === 0) return;
            activeChartIndex = (activeChartIndex - 1 + chartElements.length) % chartElements.length;
            updateChartVisibility();
        });
    }

    // Toggle Log Scale
    const toggleLogScaleButton = document.getElementById('toggleLogScale');
    if (toggleLogScaleButton) {
        toggleLogScaleButton.addEventListener('click', function() {
            if (priceVolumeChart) {
                const yAxis = priceVolumeChart.options.scales['y-price'];
                yAxis.type = yAxis.type === 'logarithmic' ? 'linear' : 'logarithmic';
                priceVolumeChart.update();
            }
        });
    }
    
    // Resize handler for all charts
    window.addEventListener('resize', function() {
        charts.forEach(chart => {
            if (chart && typeof chart.resize === 'function') {
                chart.resize();
            }
        });
        
        earningsCharts.forEach(chart => {
            if (chart && typeof chart.resize === 'function') {
                chart.resize();
            }
        });
        
        marketMechanicsCharts.forEach(chart => {
            if (chart && typeof chart.resize === 'function') {
                chart.resize();
            }
        });
    });

    // Call resize on initial load
    window.addEventListener('load', function() {
        setTimeout(() => {
            charts.forEach(chart => {
                if (chart && typeof chart.resize === 'function') {
                    chart.resize();
                }
            });
            
            earningsCharts.forEach(chart => {
                if (chart && typeof chart.resize === 'function') {
                    chart.resize();
                }
            });
            
            marketMechanicsCharts.forEach(chart => {
                if (chart && typeof chart.resize === 'function') {
                    chart.resize();
                }
            });
        }, 100);
    });
});