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

        const chartsTabButton = document.querySelector('[data-tab="charts"]');
        if (chartsTabButton) {
            chartsTabButton.addEventListener('click', async function() {
                setTimeout(async () => {
                    if (typeof window.initMempoolMetricsChart === 'function') {
                        await window.initMempoolMetricsChart();

                        // Initialize date range picker for mempool metrics
                        const mempoolPicker = new DateRangePeeker('mempool-daterange', async (start, end) => {
                            if (typeof window.refreshMempoolMetricsChart === 'function') {
                                await window.refreshMempoolMetricsChart(start, end);
                            }
                        });

                        // Initial load with default range
                        const range = mempoolPicker.getCurrentRange();
                        await window.refreshMempoolMetricsChart(range.startDate, range.endDate);
                    }
                }, 100);
            });
        }
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

        await updateMempoolStats();
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
        const indicatorColor = isEqual ? '#4caf50' : '#f44336';

        // Обновляем только значения, не трогая структуру
        const liveSpan = document.getElementById('last-block-live');
        const indexSpan = document.getElementById('last-block-index');
        const indicatorSpan = document.getElementById('last-block-indicator');

        if (liveSpan) liveSpan.textContent = lastBlockLive;
        if (indexSpan) indexSpan.textContent = lastBlockIndex;
        if (indicatorSpan) indicatorSpan.style.backgroundColor = indicatorColor;
    } catch (error) {
        console.error('Error fetching indexer stats:', error);
    }
}

async function updateMempoolStats() {
    try {
        const response = await fetch('/api/mempool/stats');
        const data = await response.json();

        document.getElementById('mempool-tx-count').textContent = data.tx_count?.toLocaleString() || '--';
        document.getElementById('mempool-vbytes').textContent = data.vbytes ? `${data.vbytes.toLocaleString()} vB` : '-- vB';
        document.getElementById('mempool-fees').textContent = data.total_fees ? `${data.total_fees.toFixed(8)} BTC` : '-- BTC';
    } catch (error) {
        console.error('Error updating mempool stats:', error);
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


$(document).on('click', '.tx-link', function(e) {
    e.preventDefault();
    e.stopPropagation();
    const txid = $(this).data('txid');
    console.log(`Opening transaction details: ${txid}`);
    showTransactionDetails(txid);
});

// Function to show transaction details modal
async function showTransactionDetails(txid) {
    // Create modal if it doesn't exist
    let modal = document.getElementById('tx-modal');
    if (!modal) {
        modal = document.createElement('div');
        modal.id = 'tx-modal';
        modal.className = 'modal';
        modal.innerHTML = `
            <div class="modal-content">
                <div class="modal-header">
                    <h2><i class="fas fa-exchange-alt"></i> Transaction Details</h2>
                    <span class="close">&times;</span>
                </div>
                <div class="modal-body">
                    <div class="tx-info">
                        <div class="info-row">
                            <span class="info-label">TXID:</span>
                            <span id="tx-txid" class="info-value hash-value"></span>
                        </div>
                        <div class="info-row">
                            <span class="info-label">Time:</span>
                            <span id="tx-time" class="info-value"></span>
                        </div>
                        <div class="info-row">
                            <span class="info-label">Height:</span>
                            <span id="tx-height" class="info-value"></span>
                        </div>
                        <div class="info-row">
                            <span class="info-label">Fee Rate:</span>
                            <span id="tx-fee-rate" class="info-value"></span>
                        </div>
                    </div>
                    <div class="json-viewer-container">
                        <pre id="tx-json" style="max-height: 400px; overflow: auto;"></pre>
                    </div>
                </div>
            </div>
        `;
        document.body.appendChild(modal);

        // Add close functionality
        const closeBtn = modal.querySelector('.close');
        closeBtn.onclick = () => modal.style.display = 'none';
        window.onclick = (event) => {
            if (event.target === modal) modal.style.display = 'none';
        };
    }

    modal.style.display = 'block';

    try {
        const response = await fetch(`/api/transaction/${txid}`);
        const data = await response.json();

        document.getElementById('tx-txid').textContent = txid;
        document.getElementById('tx-time').textContent = data.time ? new Date(data.time * 1000).toLocaleString() : 'Unknown';
        document.getElementById('tx-height').textContent = data.height || 'Mempool';
        document.getElementById('tx-fee-rate').textContent = data.fee_rate ? `${data.fee_rate.toFixed(2)} sat/vB` : 'Unknown';
        document.getElementById('tx-json').textContent = JSON.stringify(data, null, 2);
    } catch (error) {
        console.error('Error loading transaction details:', error);
        document.getElementById('tx-json').textContent = `Error loading transaction: ${error.message}`;
    }
}