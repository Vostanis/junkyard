/**
 * Chart initialization and setup
 */

// Get theme colors from CSS variables
const getThemeColors = () => {
    return {
        background: getComputedStyle(document.documentElement).getPropertyValue('--background').trim() || '#1f2023',
        foreground: getComputedStyle(document.documentElement).getPropertyValue('--foreground').trim() || '#bcb28d',
        accentBlue: getComputedStyle(document.documentElement).getPropertyValue('--accent-blue').trim() || '#58b2dc',
        accentMagenta: getComputedStyle(document.documentElement).getPropertyValue('--accent-magenta').trim() || '#e03c8a',
        textMuted: getComputedStyle(document.documentElement).getPropertyValue('--text-muted').trim() || '#707c74',
        graphBg: getComputedStyle(document.documentElement).getPropertyValue('--graph-bg').trim() || '#212121',
        accentRed: '#ff6b6b',
        accentOrange: '#ffa94d'
    };
};

// Data preparation functions
const interpolateData = (data) => {
    const result = [...data];
    for (let i = 0; i < result.length; i++) {
        if (result[i] === null) {
            let prevVal = null, nextVal = null, prevIndex = null, nextIndex = null;
            for (let j = i - 1; j >= 0; j--) {
                if (result[j] !== null) {
                    prevVal = result[j];
                    prevIndex = j;
                    break;
                }
            }
            for (let j = i + 1; j < result.length; j++) {
                if (result[j] !== null) {
                    nextVal = result[j];
                    nextIndex = j;
                    break;
                }
            }
            if (prevVal !== null && nextVal !== null) {
                const steps = nextIndex - prevIndex;
                const stepValue = (nextVal - prevVal) / steps;
                result[i] = prevVal + stepValue * (i - prevIndex);
            } else if (prevVal !== null) {
                result[i] = prevVal;
            } else if (nextVal !== null) {
                result[i] = nextVal;
            }
        }
    }
    return result;
};

// Filter data by selected date range
const filterDataByRange = (prices, range) => {
    const now = new Date('2025-02-27'); // Use a fixed date or pass as parameter
    let cutoffDate;
    
    switch (range) {
        case '1M': cutoffDate = new Date(now.setMonth(now.getMonth() - 1)); break;
        case '3M': cutoffDate = new Date(now.setMonth(now.getMonth() - 3)); break;
        case '6M': cutoffDate = new Date(now.setMonth(now.getMonth() - 6)); break;
        case '1Y': cutoffDate = new Date(now.setFullYear(now.getFullYear() - 1)); break;
        case '5Y': cutoffDate = new Date(now.setFullYear(now.getFullYear() - 5)); break;
        case 'ALL': default: cutoffDate = null;
    }
    
    const filteredPrices = cutoffDate 
        ? prices.filter(item => new Date(item.date) >= cutoffDate).reverse() 
        : prices.reverse();
    
    return {
        labels: filteredPrices.map(item => item.date),
        priceData: filteredPrices.map(item => item.adj_close),
        price20MAData: filteredPrices.map(item => item.adj_close_20ma),
        price50MAData: filteredPrices.map(item => item.adj_close_50ma),
        price200MAData: filteredPrices.map(item => item.adj_close_200ma),
        volumeData: filteredPrices.map(item => item.volume || 0)
    };
};

// Create price and volume chart
const createPriceVolumeChart = (ctx, data, theme) => {
    return new Chart(ctx, {
        data: {
            labels: data.labels,
            datasets: [
                { 
                    type: 'line', 
                    label: 'Adjusted Close Price', 
                    data: data.priceData, 
                    borderColor: theme.foreground, 
                    borderWidth: 1, 
                    fill: false, 
                    pointRadius: 0, 
                    yAxisID: 'y-price' 
                },
                { 
                    type: 'line', 
                    label: '20-Day MA', 
                    data: data.price20MAData, 
                    borderColor: theme.accentBlue, 
                    borderWidth: 1, 
                    borderDash: [5, 5], 
                    fill: false, 
                    pointRadius: 0, 
                    yAxisID: 'y-price' 
                },
                { 
                    type: 'line', 
                    label: '50-Day MA', 
                    data: data.price50MAData, 
                    borderColor: theme.accentMagenta, 
                    borderWidth: 1, 
                    borderDash: [5, 5], 
                    fill: false, 
                    pointRadius: 0, 
                    yAxisID: 'y-price' 
                },
                { 
                    type: 'line', 
                    label: '200-Day MA', 
                    data: data.price200MAData, 
                    borderColor: theme.textMuted, 
                    borderWidth: 1, 
                    borderDash: [5, 5], 
                    fill: false, 
                    pointRadius: 0, 
                    yAxisID: 'y-price' 
                },
                { 
                    type: 'bar', 
                    label: 'Volume', 
                    data: data.volumeData, 
                    backgroundColor: theme.accentMagenta + '80', 
                    borderColor: theme.accentMagenta, 
                    borderWidth: 1, 
                    yAxisID: 'y-volume', 
                    barPercentage: 0.7, 
                    categoryPercentage: 0.8 
                }
            ]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            scales: {
                x: { 
                    type: 'category', 
                    title: { 
                        display: true, 
                        text: 'Date', 
                        color: theme.foreground 
                    }, 
                    ticks: { 
                        color: theme.textMuted, 
                        maxRotation: 45, 
                        minRotation: 45, 
                        autoSkip: true, 
                        maxTicksLimit: 10 
                    }, 
                    grid: { 
                        display: true, 
                        color: theme.textMuted + '33' 
                    } 
                },
                'y-price': { 
                    position: 'left', 
                    title: { 
                        display: true, 
                        text: 'Price', 
                        color: theme.foreground 
                    }, 
                    ticks: { 
                        color: theme.textMuted 
                    }, 
                    beginAtZero: false, 
                    suggestedMin: Math.min(...data.priceData) * 0.9, 
                    suggestedMax: Math.max(...data.priceData) * 1.1, 
                    grid: { 
                        display: true, 
                        color: theme.textMuted + '33' 
                    } 
                },
                'y-volume': { 
                    position: 'right', 
                    title: { 
                        display: true, 
                        text: 'Volume', 
                        color: theme.foreground 
                    }, 
                    ticks: { 
                        color: theme.textMuted 
                    }, 
                    grid: { 
                        display: false 
                    }, 
                    beginAtZero: true, 
                    max: Math.max(...data.volumeData) * 5 
                }
            },
            plugins: {
                title: { 
                    display: true, 
                    text: 'Price + Volume with MAs', 
                    color: theme.foreground 
                },
                tooltip: { 
                    mode: 'index', 
                    intersect: false, 
                    callbacks: { 
                        label: function(context) { 
                            return `${context.dataset.label}: ${context.parsed.y.toLocaleString()}`; 
                        } 
                    }, 
                    backgroundColor: theme.graphBg, 
                    bodyColor: theme.foreground, 
                    titleColor: theme.foreground 
                },
                legend: { 
                    labels: { 
                        color: theme.foreground 
                    } 
                }
            }
        }
    });
};

// Create earnings chart
const createEarningsChart = (ctx, labels, revenueData, earningsData, accumulatedEarningsData, theme) => {
    return new Chart(ctx, {
        type: 'line',
        data: {
            labels: labels,
            datasets: [
                { 
                    label: 'Revenue (Billions)', 
                    data: revenueData, 
                    borderColor: theme.accentBlue, 
                    borderWidth: 1, 
                    fill: false, 
                    pointRadius: 1 
                },
                { 
                    label: 'Earnings (Billions)', 
                    data: earningsData, 
                    borderColor: theme.accentMagenta, 
                    backgroundColor: theme.accentMagenta + '33', 
                    borderWidth: 1, 
                    fill: true, 
                    pointRadius: 1 
                },
                { 
                    label: 'Accumulated Earnings (Billions)', 
                    data: accumulatedEarningsData, 
                    borderColor: theme.textMuted, 
                    borderWidth: 1, 
                    fill: false, 
                    pointRadius: 1 
                }
            ]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            scales: {
                x: { 
                    type: 'category', 
                    title: { 
                        display: true, 
                        text: 'End Date', 
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
                    title: { 
                        display: true, 
                        text: 'Billions', 
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
                    text: 'Earnings Metrics', 
                    color: theme.foreground 
                },
                tooltip: { 
                    mode: 'index', 
                    intersect: false, 
                    callbacks: { 
                        label: function(context) { 
                            let value = context.parsed.y;
                            if (context.dataset.label.includes('Billions')) value = `${value.toLocaleString()}B`;
                            else if (context.dataset.label.includes('Earnings %')) value = `${value.toFixed(2)}%`;
                            return `${context.dataset.label}: ${value}`; 
                        } 
                    }, 
                    backgroundColor: theme.graphBg, 
                    bodyColor: theme.foreground, 
                    titleColor: theme.foreground 
                },
                legend: { 
                    labels: { 
                        color: theme.foreground 
                    } 
                },
                annotation: {
                    annotations: {
                        zeroLine: {
                            type: 'line',
                            yMin: 0,
                            yMax: 0,
                            borderColor: theme.textMuted,
                            borderWidth: 2,
                            borderDash: [5, 5],
                            label: {
                                enabled: true,
                                content: 'Y=0',
                                position: 'start',
                                backgroundColor: theme.graphBg,
                                color: theme.foreground,
                                font: { size: 12 }
                            }
                        }
                    }
                }
            }
        }
    });
};

// Create operating profit chart
const createOperatingProfitChart = (ctx, labels, grossProfitData, operatingIncomeData, earningsData, theme) => {
    return new Chart(ctx, {
        type: 'line',
        data: {
            labels: labels,
            datasets: [
                { 
                    label: 'Gross Profit (B)', 
                    data: grossProfitData, 
                    borderColor: theme.accentOrange, 
                    borderWidth: 0.5, 
                    fill: false, 
                    pointRadius: 0.5 
                },
                { 
                    label: 'Operating Income (B)', 
                    data: operatingIncomeData, 
                    borderColor: theme.accentBlue, 
                    backgroundColor: theme.accentBlue + '33',
                    borderWidth: 0.5, 
                    fill: true, 
                    pointRadius: 0.5 
                },
                { 
                    label: 'Net Income/Earnings (B)', 
                    data: earningsData, 
                    borderColor: theme.accentMagenta, 
                    borderWidth: 0.5, 
                    fill: false, 
                    pointRadius: 0.5,
                    hidden: true
                }
            ]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            scales: {
                x: { 
                    type: 'category', 
                    ticks: { 
                        color: theme.textMuted, 
                        maxRotation: 45, 
                        minRotation: 45,
                        autoSkip: true,
                        maxTicksLimit: 5
                    }, 
                    grid: { 
                        display: true, 
                        color: theme.textMuted + '33' 
                    },
                    display: true 
                },
                y: { 
                    title: { 
                        display: true, 
                        text: 'Billions', 
                        color: theme.foreground 
                    }, 
                    ticks: { 
                        color: theme.textMuted 
                    }, 
                    grid: { 
                        color: theme.textMuted + '33' 
                    }, 
                    beginAtZero: true 
                }
            },
            plugins: {
                title: { 
                    display: true, 
                    text: 'Operating Metrics', 
                    color: theme.foreground,
                    font: { size: 12 }
                },
                tooltip: { 
                    mode: 'index', 
                    intersect: false, 
                    callbacks: { 
                        label: function(context) { 
                            return `${context.dataset.label}: ${context.parsed.y.toFixed(2)}B`; 
                        } 
                    }, 
                    backgroundColor: theme.graphBg, 
                    bodyColor: theme.foreground, 
                    titleColor: theme.foreground 
                },
                legend: { 
                    labels: { 
                        color: theme.foreground,
                        font: { size: 10 }
                    },
                    position: 'top'
                },
                annotation: {
                    annotations: {
                        zeroLine: {
                            type: 'line',
                            yMin: 0,
                            yMax: 0,
                            borderColor: theme.textMuted,
                            borderWidth: 2,
                            borderDash: [5, 5],
                            label: {
                                enabled: true,
                                content: 'Y=0',
                                position: 'start',
                                backgroundColor: theme.graphBg,
                                color: theme.foreground,
                                font: { size: 12 }
                            }
                        }
                    }
                }
            }
        }
    });
};

// Create EPS chart
const createEPSChart = (ctx, labels, epsData, theme) => {
    return new Chart(ctx, {
        type: 'line',
        data: {
            labels: labels,
            datasets: [
                { 
                    label: 'EPS ($)', 
                    data: epsData, 
                    borderColor: theme.accentBlue, 
                    borderWidth: 1, 
                    backgroundColor: theme.accentBlue + '33',
                    fill: true,
                    pointRadius: 1 
                }
            ]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            scales: {
                x: { 
                    type: 'category', 
                    ticks: { 
                        color: theme.textMuted, 
                        maxRotation: 45, 
                        minRotation: 45,
                        autoSkip: true,
                        maxTicksLimit: 5 
                    }, 
                    grid: { 
                        display: true, 
                        color: theme.textMuted + '33' 
                    },
                    display: true
                },
                y: { 
                    title: { 
                        display: true, 
                        text: 'EPS ($)', 
                        color: theme.foreground 
                    }, 
                    ticks: { 
                        color: theme.textMuted 
                    }, 
                    grid: { 
                        color: theme.textMuted + '33' 
                    }
                }
            },
            plugins: {
                title: { 
                    display: true, 
                    text: 'Earnings Per Share', 
                    color: theme.foreground,
                    font: { size: 12 }
                },
                tooltip: { 
                    mode: 'index', 
                    intersect: false, 
                    callbacks: { 
                        label: function(context) { 
                            return `${context.dataset.label}: ${context.parsed.y.toFixed(2)}`; 
                        } 
                    }, 
                    backgroundColor: theme.graphBg, 
                    bodyColor: theme.foreground, 
                    titleColor: theme.foreground 
                },
                legend: { 
                    labels: { 
                        color: theme.foreground,
                        font: { size: 10 }
                    },
                    position: 'top'
                }
            }
        }
    });
};

// Create Debt & Equity chart
const createDebtChart = (ctx, labels, debtData, equityData, assetsData, theme) => {
    return new Chart(ctx, {
        type: 'line',
        data: {
            labels: labels,
            datasets: [
                { 
                    label: 'Debt (Billions)', 
                    data: debtData, 
                    borderColor: theme.accentRed, 
                    borderWidth: 1, 
                    backgroundColor: theme.accentRed + '33',
                    fill: true,
                    pointRadius: 1 
                },
                { 
                    label: 'Equity (Billions)', 
                    data: equityData, 
                    borderColor: theme.accentBlue, 
                    borderWidth: 1, 
                    backgroundColor: theme.accentBlue + '33',
                    fill: true,
                    pointRadius: 1 
                },
                { 
                    label: 'Assets (Billions)', 
                    data: assetsData, 
                    borderColor: theme.accentOrange, 
                    borderWidth: 1, 
                    fill: false,
                    pointRadius: 1 
                }
            ]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            scales: {
                x: { 
                    type: 'category', 
                    title: { 
                        display: true, 
                        text: 'End Date', 
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
                    title: { 
                        display: true, 
                        text: 'Billions ($)', 
                        color: theme.foreground 
                    }, 
                    ticks: { 
                        color: theme.textMuted 
                    }, 
                    grid: { 
                        color: theme.textMuted + '33' 
                    }, 
                    beginAtZero: true 
                }
            },
            plugins: {
                title: { 
                    display: true, 
                    text: 'Debt & Equity Metrics', 
                    color: theme.foreground 
                },
                tooltip: { 
                    mode: 'index', 
                    intersect: false, 
                    callbacks: { 
                        label: function(context) { 
                            const value = context.parsed.y.toFixed(2);
                            let result = `${context.dataset.label}: ${value}B`;
                            
                            // Add debt to equity ratio when hovering
                            if (context.datasetIndex <= 1) { // Only for debt and equity
                                const index = context.dataIndex;
                                const debt = debtData[index];
                                const equity = equityData[index];
                                if (debt !== null && equity !== null && equity !== 0) {
                                    const debtToEquity = (debt / equity).toFixed(2);
                                    result += ` (Debt/Equity: ${debtToEquity})`;
                                }
                            }
                            
                            return result;
                        } 
                    }, 
                    backgroundColor: theme.graphBg, 
                    bodyColor: theme.foreground, 
                    titleColor: theme.foreground 
                },
                legend: { 
                    labels: { 
                        color: theme.foreground 
                    } 
                }
            }
        }
    });
};

// Create Market Cap chart
const createMarketCapChart = (ctx, labels, marketCapData, theme) => {
    return new Chart(ctx, {
        type: 'line',
        data: {
            labels: labels,
            datasets: [
                { 
                    label: 'Market Cap (Billions)', 
                    data: marketCapData, 
                    borderColor: theme.accentMagenta, 
                    borderWidth: 1, 
                    backgroundColor: theme.accentMagenta + '33',
                    fill: true,
                    pointRadius: 1
                }
            ]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            scales: {
                x: { 
                    type: 'category', 
                    title: { 
                        display: true, 
                        text: 'Date', 
                        color: theme.foreground 
                    }, 
                    ticks: { 
                        color: theme.textMuted, 
                        maxRotation: 45, 
                        minRotation: 45, 
                        autoSkip: true, 
                        maxTicksLimit: 4 
                    }, 
                    grid: { 
                        display: true, 
                        color: theme.textMuted + '33' 
                    } 
                },
                y: { 
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
                    }, 
                    beginAtZero: true 
                }
            },
            plugins: {
                title: { 
                    display: true, 
                    text: 'Market Cap', 
                    color: theme.foreground, 
                    font: { size: 12 } 
                },
                tooltip: { 
                    mode: 'index', 
                    intersect: false, 
                    callbacks: { 
                        label: function(context) { 
                            return `${context.dataset.label}: ${context.parsed.y.toFixed(2)}B`; 
                        } 
                    }, 
                    backgroundColor: theme.graphBg, 
                    bodyColor: theme.foreground, 
                    titleColor: theme.foreground 
                },
                legend: { 
                    labels: { 
                        color: theme.foreground, 
                        font: { size: 10 } 
                    },
                    position: 'top'
                }
            }
        }
    });
};

// Create Shares Outstanding chart
const createSharesOutstandingChart = (ctx, labels, sharesOutstandingData, theme) => {
    return new Chart(ctx, {
        type: 'line',
        data: {
            labels: labels,
            datasets: [
                { 
                    label: 'Shares Outstanding (M)', 
                    data: sharesOutstandingData, 
                    borderColor: theme.accentBlue, 
                    borderWidth: 1, 
                    backgroundColor: theme.accentBlue + '33',
                    fill: true,
                    pointRadius: 1
                }
            ]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            scales: {
                x: { 
                    type: 'category', 
                    title: { 
                        display: true, 
                        text: 'Date', 
                        color: theme.foreground 
                    }, 
                    ticks: { 
                        color: theme.textMuted, 
                        maxRotation: 45, 
                        minRotation: 45, 
                        autoSkip: true, 
                        maxTicksLimit: 4 
                    }, 
                    grid: { 
                        display: true, 
                        color: theme.textMuted + '33' 
                    } 
                },
                y: { 
                    title: { 
                        display: true, 
                        text: 'Millions', 
                        color: theme.foreground 
                    }, 
                    ticks: { 
                        color: theme.textMuted 
                    }, 
                    grid: { 
                        display: true, 
                        color: theme.textMuted + '33' 
                    }, 
                    beginAtZero: true 
                }
            },
            plugins: {
                title: { 
                    display: true, 
                    text: 'Shares Outstanding', 
                    color: theme.foreground, 
                    font: { size: 12 } 
                },
                tooltip: { 
                    mode: 'index', 
                    intersect: false, 
                    callbacks: { 
                        label: function(context) { 
                            return `${context.dataset.label}: ${context.parsed.y.toFixed(2)}M`; 
                        } 
                    }, 
                    backgroundColor: theme.graphBg, 
                    bodyColor: theme.foreground, 
                    titleColor: theme.foreground 
                },
                legend: { 
                    labels: { 
                        color: theme.foreground, 
                        font: { size: 10 } 
                    },
                    position: 'top'
                }
            }
        }
    });
};

// Create Float chart
const createFloatChart = (ctx, labels, floatData, theme) => {
    return new Chart(ctx, {
        type: 'line',
        data: {
            labels: labels,
            datasets: [
                { 
                    label: 'Float (Millions)', 
                    data: floatData, 
                    borderColor: theme.accentRed, 
                    borderWidth: 1, 
                    backgroundColor: theme.accentRed + '33',
                    fill: true,
                    pointRadius: 1
                }
            ]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            scales: {
                x: { 
                    type: 'category', 
                    title: { 
                        display: true, 
                        text: 'Date', 
                        color: theme.foreground 
                    }, 
                    ticks: { 
                        color: theme.textMuted, 
                        maxRotation: 45, 
                        minRotation: 45, 
                        autoSkip: true, 
                        maxTicksLimit: 4 
                    }, 
                    grid: { 
                        display: true, 
                        color: theme.textMuted + '33' 
                    } 
                },
                y: { 
                    title: { 
                        display: true, 
                        text: 'Millions', 
                        color: theme.foreground 
                    }, 
                    ticks: { 
                        color: theme.textMuted 
                    }, 
                    grid: { 
                        display: true, 
                        color: theme.textMuted + '33' 
                    }, 
                    beginAtZero: true 
                }
            },
            plugins: {
                title: { 
                    display: true, 
                    text: 'Float', 
                    color: theme.foreground, 
                    font: { size: 12 } 
                },
                tooltip: { 
                    mode: 'index', 
                    intersect: false, 
                    callbacks: { 
                        label: function(context) { 
                            return `${context.dataset.label}: ${context.parsed.y.toFixed(2)}M`; 
                        } 
                    }, 
                    backgroundColor: theme.graphBg, 
                    bodyColor: theme.foreground, 
                    titleColor: theme.foreground 
                },
                legend: { 
                    labels: { 
                        color: theme.foreground, 
                        font: { size: 10 } 
                    },
                    position: 'top'
                }
            }
        }
    });
};

// Create Shares Bought Back chart
const createSharesBoughtBackChart = (ctx, labels, sharesBoughtBackData, theme) => {
    return new Chart(ctx, {
        type: 'line',
        data: {
            labels: labels,
            datasets: [
                { 
                    label: 'Shares Bought Back (M$)', 
                    data: sharesBoughtBackData, 
                    borderColor: theme.accentOrange, 
                    borderWidth: 1, 
                    backgroundColor: theme.accentOrange + '33',
                    fill: true,
                    pointRadius: 1 
                }
            ]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            scales: {
                x: { 
                    type: 'category', 
                    title: { 
                        display: true, 
                        text: 'Date', 
                        color: theme.foreground 
                    }, 
                    ticks: { 
                        color: theme.textMuted, 
                        maxRotation: 45, 
                        minRotation: 45, 
                        autoSkip: true, 
                        maxTicksLimit: 4 
                    }, 
                    grid: { 
                        display: true, 
                        color: theme.textMuted + '33' 
                    } 
                },
                y: { 
                    title: { 
                        display: true, 
                        text: 'Millions', 
                        color: theme.foreground 
                    }, 
                    ticks: { 
                        color: theme.textMuted 
                    }, 
                    grid: { 
                        display: true, 
                        color: theme.textMuted + '33' 
                    }, 
                    beginAtZero: true 
                }
            },
            plugins: {
                title: { 
                    display: true, 
                    text: 'Shares Bought Back', 
                    color: theme.foreground, 
                    font: { size: 12 } 
                },
                tooltip: { 
                    mode: 'index', 
                    intersect: false, 
                    callbacks: { 
                        label: function(context) { 
                            return `${context.dataset.label}: ${context.parsed.y.toFixed(2)}M`; 
                        } 
                    }, 
                    backgroundColor: theme.graphBg, 
                    bodyColor: theme.foreground, 
                    titleColor: theme.foreground 
                },
                legend: { 
                    labels: { 
                        color: theme.foreground, 
                        font: { size: 10 } 
                    },
                    position: 'top'
                }
            }
        }
    });
};