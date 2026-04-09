// Tab switching functionality

const USE_MOCK = false;

const API_BASE = USE_MOCK ? '/api/mock' : '/api';

// Network switching functionality
async function initNetworkSelector() {
    const networkSelect = document.getElementById('network-select');
    if (!networkSelect) return;

    // Get current network
    try {
        const response = await fetch('/api/network/current');
        const data = await response.json();
        networkSelect.value = data.current_network;
        updateNetworkInfo();
    } catch (error) {
        console.error('Error fetching current network:', error);
    }

    // Add change event listener
    networkSelect.addEventListener('change', async (e) => {
        const newNetwork = e.target.value;

        // Show loading state
        const originalText = networkSelect.options[networkSelect.selectedIndex].text;
        networkSelect.disabled = true;

        try {
            const response = await fetch('/api/network/switch', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ network: newNetwork })
            });

            const result = await response.json();

            if (response.ok) {
                console.log('Network switched:', result);
                showNotification(`Switched to ${newNetwork} network`, 'success');

                // Refresh all data
                await refreshAllData();
                await updateNetworkInfo();
            } else {
                showNotification(`Failed to switch network: ${result.message}`, 'error');
                networkSelect.value = networkSelect.getAttribute('data-current') || 'regtest';
            }
        } catch (error) {
            console.error('Error switching network:', error);
            showNotification('Error switching network', 'error');
            networkSelect.value = networkSelect.getAttribute('data-current') || 'regtest';
        } finally {
            networkSelect.disabled = false;
        }
    });

    // Auto-refresh network info every 30 seconds
    setInterval(updateNetworkInfo, 30000);
}

async function updateNetworkInfo() {
    try {
        const response = await fetch('/api/network/info');
        const data = await response.json();

        const indicator = document.getElementById('network-indicator');
        const blockHeightSpan = document.getElementById('network-block-height');

        if (data.blockchain_info && data.blockchain_info.blocks) {
            blockHeightSpan.textContent = `Block ${data.blockchain_info.blocks.toLocaleString()}`;
            indicator.style.backgroundColor = '#4caf50';
        } else {
            blockHeightSpan.textContent = 'Connected';
            indicator.style.backgroundColor = '#f7931a';
        }

        // Store current network in select element
        const networkSelect = document.getElementById('network-select');
        if (networkSelect && networkSelect.value !== data.current_network) {
            networkSelect.value = data.current_network;
        }
        networkSelect.setAttribute('data-current', data.current_network);

    } catch (error) {
        console.error('Error updating network info:', error);
        const indicator = document.getElementById('network-indicator');
        const blockHeightSpan = document.getElementById('network-block-height');
        if (indicator) indicator.style.backgroundColor = '#f44336';
        if (blockHeightSpan) blockHeightSpan.textContent = 'Error connecting';
    }
}

async function refreshAllData() {
    // Refresh all tables and tiles
    await Promise.all([
        updateLiveTiles(),
        fetchIndexerStats(),
        // Add other refresh functions as needed
        window.refreshBlocksTable && window.refreshBlocksTable(),
        window.refreshMempoolTable && window.refreshMempoolTable(),
        window.refreshPeersTable && window.refreshPeersTable(),
    ]);
}

function showNotification(message, type = 'info') {
    // Create notification element
    const notification = document.createElement('div');
    notification.className = `notification notification-${type}`;
    notification.innerHTML = `
        <div style="position: fixed; top: 20px; left: 50%; transform: translateX(-50%); 
                    background: ${type === 'success' ? '#4caf50' : type === 'error' ? '#f44336' : '#2196f3'}; 
                    color: white; padding: 12px 20px; border-radius: 5px; 
                    z-index: 10000; box-shadow: 0 2px 10px rgba(0,0,0,0.3);">
            ${message}
        </div>
    `;
    document.body.appendChild(notification);
    setTimeout(() => notification.remove(), 3000);
}

document.addEventListener('DOMContentLoaded', function() {
    console.log('DOM loaded - initializing tabs');

    // Initialize network selector
    initNetworkSelector();

    // Initialize tabs
    const tabButtons = document.querySelectorAll('.tab-button');
    console.log('Found tab buttons:', tabButtons.length);

    tabButtons.forEach(button => {
        button.addEventListener('click', function(e) {
            console.log('Tab clicked:', this);

            // Get the tab id from data attribute
            const tabId = this.getAttribute('data-tab');
            console.log('Switching to tab:', tabId);

            // Remove active class from all buttons and panes
            document.querySelectorAll('.tab-button').forEach(btn => {
                btn.classList.remove('active');
            });
            document.querySelectorAll('.tab-pane').forEach(pane => {
                pane.classList.remove('active');
            });

            // Add active class to clicked button
            this.classList.add('active');

            // Show corresponding tab pane
            const targetPane = document.getElementById('tab-' + tabId);
            if (targetPane) {
                targetPane.classList.add('active');
                console.log('Activated pane:', targetPane);
            } else {
                console.error('Pane not found for tab:', tabId);
            }

            if (tabId === 'charts' && typeof blockchainChart !== 'undefined' && blockchainChart) {
                // Small delay to ensure the chart container is visible and has dimensions before resizing
                setTimeout(() => {
                    blockchainChart.resize();
                }, 100);
            }

            // Adjust DataTables columns when switching to tables tab
            if ($.fn.dataTable) {
                $.fn.dataTable.tables({ visible: true, api: true }).columns.adjust();
            }

        });
    });

    // Initialize other components
    initLiveTiles();
    initCharts();

    // Start auto-updates
    setInterval(updateLiveTiles, 5000);
    setInterval(updateTimestamp, 1000);
});

// Live Tiles functions
async function initLiveTiles() {
    await updateLiveTiles();
}

async function updateLiveTiles() {
    try {
        const response = await fetch('/api/live');
        const data = await response.json();

        document.getElementById('mempool-count').textContent = data.mempool_count?.toLocaleString() || '--';
        document.getElementById('peer-count').textContent = data.peer_count || '--';
        document.getElementById('hashrate').textContent = data.hashrate ? data.hashrate.toFixed(2) : '--';
    } catch (error) {
        console.error('Error updating live tiles:', error);
    }
}

async function updateIndexerTile() {
    try {
        const response = await fetch('/api/indexer');
        const data = await response.json();

        const lastBlockLive = data.last_block_live !== undefined ? data.last_block_live : '--';
        const lastBlockIndex = data.last_block_index !== undefined ? data.last_block_index : '--';

        const isEqual = (lastBlockLive === lastBlockIndex) && lastBlockLive !== '--';
        const indicatorColor = isEqual ? 'green' : 'red';

        const tileHtml = `
            <div class="tile-icon"><i class="fas fa-cubes"></i></div>
            <div class="tile-content">
                <div class="tile-label">Last block</div>
                <div class="tile-value">
                    <span style="color: #f7931a;">Live: ${lastBlockLive}</span><br>
                    <span style="color: #666;">Index: ${lastBlockIndex}</span>
                    <span style="display: inline-block; width: 12px; height: 12px; border-radius: 50%; background-color: ${indicatorColor}; margin-left: 8px;"></span>
                </div>
            </div>
        `;

        const lastBlockTile = document.querySelector('#last-block-tile');
        if (lastBlockTile) lastBlockTile.innerHTML = tileHtml;
    } catch (error) {
        console.error('Error updating indexer tile:', error);
    }
}

function updateTimestamp() {
    document.getElementById('update-time').textContent = new Date().toLocaleTimeString();
}

// Function for the Live Stats tiles
async function fetchLiveStats() {
    try {
        const response = await fetch('/api/live');
        const data = await response.json();

        document.getElementById('mempool-count').textContent = data.mempool_count?.toLocaleString() || '--';
        document.getElementById('peer-count').textContent = data.peer_count || '--';
        document.getElementById('hashrate').textContent = data.hashrate ? data.hashrate.toFixed(2) : '--';
        document.getElementById('update-time').textContent = new Date().toLocaleTimeString();
    } catch (error) {
        console.error('Error fetching live stats:', error);
        document.getElementById('mempool-count').textContent = 'Error';
        document.getElementById('peer-count').textContent = 'Error';
        document.getElementById('hashrate').textContent = 'Error';
    }
}

// Function for the Last block tile with comparison indicator
async function fetchIndexerStats() {
    try {
        const response = await fetch('/api/indexer');
        const data = await response.json();

        const lastBlockLive = data.last_block_live !== undefined ? data.last_block_live : '--';
        const lastBlockIndex = data.last_block_index !== undefined ? data.last_block_index : '--';
        const isEqual = (lastBlockLive === lastBlockIndex) && lastBlockLive !== '--';
        const indicatorColor = isEqual ? 'green' : 'red';

        // Update the Last block tile content
        const tile = document.getElementById('last-block-tile');
        if (tile) {
            tile.innerHTML = `
                <div class="tile-icon"><i class="fas fa-cubes"></i></div>
                <div class="tile-content">
                    <div class="tile-label">Last block</div>
                    <div class="tile-value">
                        <span style="color: #f7931a;">Live: ${lastBlockLive}</span><br>
                        <span style="color: #666;">Index: ${lastBlockIndex}</span>
                        <span style="display: inline-block; width: 12px; height: 12px; border-radius: 50%; background-color: ${indicatorColor}; margin-left: 8px;"></span>
                    </div>
                </div>
            `;
        }
    } catch (error) {
        console.error('Error fetching indexer stats:', error);
        const tile = document.getElementById('last-block-tile');
        if (tile) {
            tile.innerHTML = `
                <div class="tile-icon"><i class="fas fa-cubes"></i></div>
                <div class="tile-content">
                    <div class="tile-label">Last block</div>
                    <div class="tile-value">Error</div>
                </div>
            `;
        }
    }
}

// Initialize charts
function initCharts() {
    // This function will be overridden by charts.js
    if (typeof window.initAllCharts === 'function') {
        window.initAllCharts();
    }
}

// Call initial fetches
fetchLiveStats();
fetchIndexerStats();

// Set up periodic updates
setInterval(fetchLiveStats, 10000);
setInterval(fetchIndexerStats, 10000);