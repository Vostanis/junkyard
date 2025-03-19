/**
 * Consolidated Tab Navigation Fix
 * 
 * This script should replace the keyboard navigation parts in both
 * dashboard.js and balance-sheet-charts.js to prevent conflicts.
 */

// Run this after the DOM is loaded and both scripts have executed
document.addEventListener('DOMContentLoaded', () => {
    // Wait for everything else to initialize first
    setTimeout(() => {
      console.log('Applying tab navigation fix...');
      
      // Remove existing keyboard event listeners (can't directly remove anonymous functions,
      // so we'll overwrite them with our own consolidated event handler)
      
      // Get all chart containers including Balance Sheet
      const getAllContainers = () => {
        return [
          document.getElementById('priceVolumeChart'),
          document.getElementById('earningsContainer'),
          document.getElementById('debtChart'),
          document.getElementById('marketMechanicsContainer'),
          document.getElementById('balanceSheetContainer')
        ].filter(el => el !== null);
      };
      
      // Consolidated function to update chart visibility
      window.updateAllCharts = (newIndex) => {
        const containers = getAllContainers();
        if (containers.length === 0) return;
        
        // Set global active index
        window.activeChartIndex = newIndex;
        
        // Update visibility classes
        containers.forEach((container, i) => {
          if (container) {
            container.classList.toggle('active', i === newIndex);
            container.classList.toggle('hidden', i !== newIndex);
          }
        });
        
        // Update tab active states
        const tabs = document.querySelectorAll('.tab');
        tabs.forEach((tab, i) => {
          tab.classList.toggle('active', i === newIndex);
        });
        
        // Update controls bar visibility (only visible for Price & Volume tab)
        const controlsBar = document.querySelector('.controls-bar');
        if (controlsBar) {
          controlsBar.classList.toggle('active', newIndex === 0);
        }
        
        // Resize appropriate charts based on active tab
        setTimeout(() => {
          switch(newIndex) {
            case 0: // Price & Volume
              if (window.charts && window.charts[0]) {
                window.charts[0].resize();
              }
              break;
              
            case 1: // Earnings
              if (window.charts && window.charts[1]) {
                window.charts[1].resize();
              }
              if (window.earningsCharts) {
                window.earningsCharts.forEach(chart => {
                  if (chart && typeof chart.resize === 'function') {
                    chart.resize();
                  }
                });
              }
              break;
              
            case 2: // Debt & Equity
              if (window.charts && window.charts[2]) {
                window.charts[2].resize();
              }
              break;
              
            case 3: // Market Mechanics
              if (window.marketMechanicsCharts) {
                window.marketMechanicsCharts.forEach(chart => {
                  if (chart && typeof chart.resize === 'function') {
                    chart.resize();
                  }
                });
              }
              break;
              
            case 4: // Balance Sheet
              if (window.balanceSheetCharts) {
                window.balanceSheetCharts.forEach(chart => {
                  if (chart && typeof chart.resize === 'function') {
                    chart.resize();
                  }
                });
              }
              break;
          }
        }, 50);
      };
      
      // Consolidated keyboard navigation handler
      const handleKeyNavigation = (event) => {
        if (event.key === 'ArrowRight' || event.key === 'ArrowLeft') {
          const containers = getAllContainers();
          if (containers.length === 0) return;
          
          const currentIndex = window.activeChartIndex || 0;
          let newIndex;
          
          if (event.key === 'ArrowRight') {
            newIndex = (currentIndex + 1) % containers.length;
          } else {
            newIndex = (currentIndex - 1 + containers.length) % containers.length;
          }
          
          window.updateAllCharts(newIndex);
        }
      };
      
      // Remove old event listeners by overriding with our consolidated one
      // (We can't directly remove anonymous functions attached by the original code)
      document.removeEventListener('keydown', handleKeyNavigation);
      document.addEventListener('keydown', handleKeyNavigation);
      
      // Update tab click handlers
      const tabs = document.querySelectorAll('.tab');
      tabs.forEach(tab => {
        tab.addEventListener('click', function() {
          const index = parseInt(this.getAttribute('data-chart-index'));
          window.updateAllCharts(index);
        });
      });
      
      // Update navigation buttons
      const prevButton = document.getElementById('prevChart');
      const nextButton = document.getElementById('nextChart');
      
      if (prevButton) {
        prevButton.addEventListener('click', function() {
          const containers = getAllContainers();
          if (containers.length === 0) return;
          
          const currentIndex = window.activeChartIndex || 0;
          const newIndex = (currentIndex - 1 + containers.length) % containers.length;
          
          window.updateAllCharts(newIndex);
        });
      }
      
      if (nextButton) {
        nextButton.addEventListener('click', function() {
          const containers = getAllContainers();
          if (containers.length === 0) return;
          
          const currentIndex = window.activeChartIndex || 0;
          const newIndex = (currentIndex + 1) % containers.length;
          
          window.updateAllCharts(newIndex);
        });
      }
      
      // Store charts in window for global access
      window.charts = window.charts || [];
      window.earningsCharts = window.earningsCharts || [];
      window.marketMechanicsCharts = window.marketMechanicsCharts || [];
      window.balanceSheetCharts = window.balanceSheetCharts || [];
      
      console.log('Tab navigation fix applied.');
    }, 500); // Wait for other scripts to initialize fully
  });