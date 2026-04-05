// Tab switching functionality

const USE_MOCK = false;

const API_BASE = USE_MOCK ? '/api/mock' : '/api';

document.addEventListener('DOMContentLoaded', function() {
    console.log('DOM loaded - initializing tabs');

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
        const response = await fetch('/api/live');   // используем ваш новый эндпоинт
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

        // Обновляем содержимое плитки Last block
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

fetchLiveStats();
fetchIndexerStats();

setInterval(fetchLiveStats, 10000);
setInterval(fetchIndexerStats, 10000);