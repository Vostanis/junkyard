/**
 * Balance Sheet Breakdown Charts for Stock Dashboard
 */

// Function to create Asset Breakdown chart
const createAssetBreakdownChart = (ctx, finLabels, financialsData, theme) => {
    // Extract detailed asset data
    const extractAssetData = (financialsData) => {
        return financialsData.map(item => {
            // Create an object to store asset breakdown
            // Convert values to billions for consistency with other charts
            return {
                date: item.end_date,
                cash: item.cash ? item.cash / 1e9 : 0,
                marketable_securities: item.marketable_securities_current ? item.marketable_securities_current / 1e9 : 0,
                accounts_receivable: item.accounts_receivable_current ? item.accounts_receivable_current / 1e9 : 0,
                nontrade_receivable_current: item.nontrade_receivable_current ? item.nontrade_receivable_current / 1e9 : 0,
                nontrade_receivable_non_current: item.nontrade_receivable_non_current ? item.nontrade_receivable_non_current / 1e9 : 0,
                inventory: item.inventory_net ? item.inventory_net / 1e9 : 0,
                ppe: item.property_plant_and_equipment_net ? item.property_plant_and_equipment_net / 1e9 : 0,
                other_current: item.other_assets_current ? item.other_assets_current / 1e9 : 0,
                other_non_current: item.other_assets_non_current ? item.other_assets_non_current / 1e9 : 0,
                // Calculate total for verification
                total: (item.assets ? item.assets / 1e9 : 0)
            };
        });
    };

    const assetData = extractAssetData(financialsData);
    
    // Create datasets
    const datasets = [
        {
            label: 'Cash',
            data: assetData.map(d => d.cash),
            backgroundColor: theme.cashColor || '#4CAF50', // Green
            stack: 'assets'
        },
        {
            label: 'Marketable Securities',
            data: assetData.map(d => d.marketable_securities),
            backgroundColor: theme.securitiesColor || '#2196F3', // Blue
            stack: 'assets'
        },
        {
            label: 'Accounts Receivable',
            data: assetData.map(d => d.accounts_receivable),
            backgroundColor: theme.receivablesColor || '#FFC107', // Amber
            stack: 'assets'
        },
        {
            label: 'Nontrade Receivables (Current)',
            data: assetData.map(d => d.nontrade_receivable_current),
            backgroundColor: theme.nontradeRecColor || '#FF9800', // Orange
            stack: 'assets'
        },
        {
            label: 'Nontrade Receivables (Non-Current)',
            data: assetData.map(d => d.nontrade_receivable_non_current),
            backgroundColor: theme.nontradeRecNonCurColor || '#FF5722', // Deep Orange
            stack: 'assets'
        },
        {
            label: 'Inventory',
            data: assetData.map(d => d.inventory),
            backgroundColor: theme.inventoryColor || '#9C27B0', // Purple
            stack: 'assets'
        },
        {
            label: 'Property, Plant & Equipment',
            data: assetData.map(d => d.ppe),
            backgroundColor: theme.ppeColor || '#607D8B', // Blue Grey
            stack: 'assets'
        },
        {
            label: 'Other Current Assets',
            data: assetData.map(d => d.other_current),
            backgroundColor: theme.otherCurAssetsColor || '#795548', // Brown
            stack: 'assets'
        },
        {
            label: 'Other Non-Current Assets',
            data: assetData.map(d => d.other_non_current),
            backgroundColor: theme.otherNonCurAssetsColor || '#9E9E9E', // Grey
            stack: 'assets'
        }
    ];

    return new Chart(ctx, {
        type: 'bar',
        data: {
            labels: finLabels,
            datasets: datasets
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            scales: {
                x: {
                    stacked: true,
                    title: {
                        display: true,
                        text: 'Date',
                        color: theme.foreground
                    },
                    ticks: {
                        color: theme.textMuted,
                        maxRotation: 45,
                        minRotation: 45
                    },
                    grid: {
                        display: true,
                        color: theme.textMuted + '33'
                    }
                },
                y: {
                    stacked: true,
                    title: {
                        display: true,
                        text: 'Billions ($)',
                        color: theme.foreground
                    },
                    ticks: {
                        color: theme.textMuted
                    },
                    grid: {
                        display: true,
                        color: theme.textMuted + '33'
                    }
                }
            },
            plugins: {
                title: {
                    display: true,
                    text: 'Asset Breakdown',
                    color: theme.foreground
                },
                tooltip: {
                    mode: 'index',
                    intersect: false,
                    callbacks: {
                        label: function(context) {
                            return `${context.dataset.label}: $${context.parsed.y.toFixed(2)}B`;
                        },
                        footer: function(tooltipItems) {
                            let sum = 0;
                            tooltipItems.forEach(item => {
                                sum += item.parsed.y;
                            });
                            return `Total: $${sum.toFixed(2)}B`;
                        }
                    },
                    backgroundColor: theme.graphBg,
                    bodyColor: theme.foreground,
                    titleColor: theme.foreground
                },
                legend: {
                    position: 'right',
                    labels: {
                        color: theme.foreground,
                        padding: 10,
                        usePointStyle: true,
                        pointStyle: 'rectRounded'
                    }
                }
            }
        }
    });
};

// Function to create Liability Breakdown chart
const createLiabilityBreakdownChart = (ctx, finLabels, financialsData, theme) => {
    // Extract detailed liability data
    const extractLiabilityData = (financialsData) => {
        return financialsData.map(item => {
            // Create an object to store liability breakdown
            // Convert values to billions for consistency with other charts
            return {
                date: item.end_date,
                accounts_payable: item.accounts_payable_current ? item.accounts_payable_current / 1e9 : 0,
                contracts_with_customer_current: item.contracts_with_customer_current ? item.contracts_with_customer_current / 1e9 : 0,
                contracts_with_customer_non_current: item.contracts_with_customer_non_current ? item.contracts_with_customer_non_current / 1e9 : 0,
                commercial_paper: item.commercial_paper ? item.commercial_paper / 1e9 : 0,
                long_term_debt_current: item.long_term_debt_current ? item.long_term_debt_current / 1e9 : 0,
                long_term_debt_non_current: item.long_term_debt_non_current ? item.long_term_debt_non_current / 1e9 : 0,
                other_liabilities_current: item.other_liabilities_current ? item.other_liabilities_current / 1e9 : 0,
                other_liabilities_non_current: item.other_liabilities_non_current ? item.other_liabilities_non_current / 1e9 : 0,
                // Calculate total for verification
                total: (item.debt ? item.debt / 1e9 : 0)
            };
        });
    };

    const liabilityData = extractLiabilityData(financialsData);
    
    // Create datasets
    const datasets = [
        {
            label: 'Accounts Payable',
            data: liabilityData.map(d => d.accounts_payable),
            backgroundColor: theme.payablesColor || '#F44336', // Red
            stack: 'liabilities'
        },
        {
            label: 'Customer Contracts (Current)',
            data: liabilityData.map(d => d.contracts_with_customer_current),
            backgroundColor: theme.customerContractsCurColor || '#E91E63', // Pink
            stack: 'liabilities'
        },
        {
            label: 'Customer Contracts (Non-Current)',
            data: liabilityData.map(d => d.contracts_with_customer_non_current),
            backgroundColor: theme.customerContractsNonCurColor || '#9C27B0', // Purple
            stack: 'liabilities'
        },
        {
            label: 'Commercial Paper',
            data: liabilityData.map(d => d.commercial_paper),
            backgroundColor: theme.commercialPaperColor || '#673AB7', // Deep Purple
            stack: 'liabilities'
        },
        {
            label: 'Long-Term Debt (Current)',
            data: liabilityData.map(d => d.long_term_debt_current),
            backgroundColor: theme.longTermDebtCurColor || '#3F51B5', // Indigo
            stack: 'liabilities'
        },
        {
            label: 'Long-Term Debt (Non-Current)',
            data: liabilityData.map(d => d.long_term_debt_non_current),
            backgroundColor: theme.longTermDebtNonCurColor || '#2196F3', // Blue
            stack: 'liabilities'
        },
        {
            label: 'Other Liabilities (Current)',
            data: liabilityData.map(d => d.other_liabilities_current),
            backgroundColor: theme.otherLiabCurColor || '#00BCD4', // Cyan
            stack: 'liabilities'
        },
        {
            label: 'Other Liabilities (Non-Current)',
            data: liabilityData.map(d => d.other_liabilities_non_current),
            backgroundColor: theme.otherLiabNonCurColor || '#009688', // Teal
            stack: 'liabilities'
        }
    ];

    return new Chart(ctx, {
        type: 'bar',
        data: {
            labels: finLabels,
            datasets: datasets
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            scales: {
                x: {
                    stacked: true,
                    title: {
                        display: true,
                        text: 'Date',
                        color: theme.foreground
                    },
                    ticks: {
                        color: theme.textMuted,
                        maxRotation: 45,
                        minRotation: 45
                    },
                    grid: {
                        display: true,
                        color: theme.textMuted + '33'
                    }
                },
                y: {
                    stacked: true,
                    title: {
                        display: true,
                        text: 'Billions ($)',
                        color: theme.foreground
                    },
                    ticks: {
                        color: theme.textMuted
                    },
                    grid: {
                        display: true,
                        color: theme.textMuted + '33'
                    }
                }
            },
            plugins: {
                title: {
                    display: true,
                    text: 'Liability Breakdown',
                    color: theme.foreground
                },
                tooltip: {
                    mode: 'index',
                    intersect: false,
                    callbacks: {
                        label: function(context) {
                            return `${context.dataset.label}: $${context.parsed.y.toFixed(2)}B`;
                        },
                        footer: function(tooltipItems) {
                            let sum = 0;
                            tooltipItems.forEach(item => {
                                sum += item.parsed.y;
                            });
                            return `Total: $${sum.toFixed(2)}B`;
                        }
                    },
                    backgroundColor: theme.graphBg,
                    bodyColor: theme.foreground,
                    titleColor: theme.foreground
                },
                legend: {
                    position: 'right',
                    labels: {
                        color: theme.foreground,
                        padding: 10,
                        usePointStyle: true,
                        pointStyle: 'rectRounded'
                    }
                }
            }
        }
    });
};

// Initialize and setup the Balance Sheet tab and charts
document.addEventListener('DOMContentLoaded', () => {
    console.log('Initializing Balance Sheet charts...');
    
    // First, check if we already have a Balance Sheet tab
    let balanceSheetTab = document.querySelector('.tab[data-chart-index="4"]');
    
    // If not, create the tab
    if (!balanceSheetTab) {
        const tabBar = document.querySelector('.tab-bar');
        if (tabBar) {
            balanceSheetTab = document.createElement('button');
            balanceSheetTab.className = 'tab';
            balanceSheetTab.setAttribute('data-chart-index', '4');
            balanceSheetTab.textContent = 'Balance Sheet';
            tabBar.appendChild(balanceSheetTab);
            
            console.log('Balance Sheet tab created');
        }
    }
    
    // Check if we already have the container
    let balanceSheetContainer = document.getElementById('balanceSheetContainer');
    
    // If not, create the container structure
    if (!balanceSheetContainer) {
        const chartWrapper = document.querySelector('.chart-wrapper');
        if (chartWrapper) {
            // Create main container
            balanceSheetContainer = document.createElement('div');
            balanceSheetContainer.id = 'balanceSheetContainer';
            balanceSheetContainer.className = 'hidden split-chart-container';
            
            // Create asset breakdown div
            const assetBreakdownDiv = document.createElement('div');
            assetBreakdownDiv.className = 'chart-half';
            assetBreakdownDiv.style.height = '50%';
            assetBreakdownDiv.style.width = '100%';
            
            const assetWrapperQuad = document.createElement('div');
            assetWrapperQuad.className = 'chart-wrapper-quad';
            
            const assetCanvas = document.createElement('canvas');
            assetCanvas.id = 'assetBreakdownChart';
            
            assetWrapperQuad.appendChild(assetCanvas);
            assetBreakdownDiv.appendChild(assetWrapperQuad);
            
            // Create liability breakdown div
            const liabilityBreakdownDiv = document.createElement('div');
            liabilityBreakdownDiv.className = 'chart-half';
            liabilityBreakdownDiv.style.height = '50%';
            liabilityBreakdownDiv.style.width = '100%';
            
            const liabilityWrapperQuad = document.createElement('div');
            liabilityWrapperQuad.className = 'chart-wrapper-quad';
            
            const liabilityCanvas = document.createElement('canvas');
            liabilityCanvas.id = 'liabilityBreakdownChart';
            
            liabilityWrapperQuad.appendChild(liabilityCanvas);
            liabilityBreakdownDiv.appendChild(liabilityWrapperQuad);
            
            // Add both divs to the container
            balanceSheetContainer.appendChild(assetBreakdownDiv);
            balanceSheetContainer.appendChild(liabilityBreakdownDiv);
            
            // Add the container to the chart wrapper
            chartWrapper.appendChild(balanceSheetContainer);
            
            console.log('Balance Sheet container structure created');
        }
    }
    
    // Now create the charts if we have financials data
    if (typeof financials !== 'undefined') {
        const assetCtx = document.getElementById('assetBreakdownChart')?.getContext('2d');
        const liabilityCtx = document.getElementById('liabilityBreakdownChart')?.getContext('2d');
        
        if (assetCtx && liabilityCtx) {
            // Get theme colors
            const theme = getThemeColors();
            
            // Add additional theme colors for the breakdown charts
            theme.cashColor = '#4CAF50';
            theme.securitiesColor = '#2196F3';
            theme.receivablesColor = '#FFC107';
            theme.payablesColor = '#F44336';
            theme.nontradeRecColor = '#FF9800';
            theme.nontradeRecNonCurColor = '#FF5722';
            theme.inventoryColor = '#9C27B0';
            theme.ppeColor = '#607D8B';
            theme.otherCurAssetsColor = '#795548';
            theme.otherNonCurAssetsColor = '#9E9E9E';
            theme.customerContractsCurColor = '#E91E63';
            theme.customerContractsNonCurColor = '#9C27B0';
            theme.commercialPaperColor = '#673AB7';
            theme.longTermDebtCurColor = '#3F51B5';
            theme.longTermDebtNonCurColor = '#2196F3';
            theme.otherLiabCurColor = '#00BCD4';
            theme.otherLiabNonCurColor = '#009688';
            
            // Always work with a copy to avoid modifying the original data
            const financialsData = [...financials];
            
            // Prepare financial data (sort by date)
            financialsData.sort((a, b) => new Date(a.end_date) - new Date(b.end_date));
            const finLabels = financialsData.map(item => item.end_date);
            
            // Create charts
            const assetBreakdownChart = createAssetBreakdownChart(assetCtx, finLabels, financialsData, theme);
            const liabilityBreakdownChart = createLiabilityBreakdownChart(liabilityCtx, finLabels, financialsData, theme);
            
            // Store charts in global array for resizing
            if (!window.balanceSheetCharts) {
                window.balanceSheetCharts = [assetBreakdownChart, liabilityBreakdownChart];
            }
            
            console.log('Balance Sheet charts created');
            
            // Update tab click handler (for the Balance Sheet tab)
            if (balanceSheetTab) {
                balanceSheetTab.addEventListener('click', function() {
                    updateChartsVisibility(4);
                });
            }
        }
    }
});

// Function to update visibility of all chart elements
function updateChartsVisibility(index) {
    // Get all chart container elements
    const containers = [
        document.getElementById('priceVolumeChart'),
        document.getElementById('earningsContainer'),
        document.getElementById('debtChart'),
        document.getElementById('marketMechanicsContainer'),
        document.getElementById('balanceSheetContainer')
    ].filter(el => el !== null);
    
    // Update visibility classes
    containers.forEach((container, i) => {
        if (container) {
            container.classList.toggle('active', i === index);
            container.classList.toggle('hidden', i !== index);
        }
    });
    
    // Update tab active states
    const tabs = document.querySelectorAll('.tab');
    tabs.forEach((tab, i) => {
        tab.classList.toggle('active', i === index);
    });
    
    // Update controls bar visibility (only visible for Price & Volume tab)
    const controlsBar = document.querySelector('.controls-bar');
    if (controlsBar) {
        controlsBar.classList.toggle('active', index === 0);
    }
    
    // Resize visible charts
    setTimeout(() => {
        if (index === 4 && window.balanceSheetCharts) {
            window.balanceSheetCharts.forEach(chart => {
                if (chart && typeof chart.resize === 'function') {
                    chart.resize();
                }
            });
        }
    }, 50);
    
    // Store active index in window for keyboard/button navigation
    window.activeChartIndex = index;
}

// Add Balance Sheet charts to window resize handler
window.addEventListener('resize', function() {
    if (window.balanceSheetCharts) {
        window.balanceSheetCharts.forEach(chart => {
            if (chart && typeof chart.resize === 'function') {
                chart.resize();
            }
        });
    }
});