// Tab switching functionality
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

        document.getElementById('last-block').textContent = data.last_block || '--';
        document.getElementById('mempool-count').textContent = data.mempool_count?.toLocaleString() || '--';
        document.getElementById('peer-count').textContent = data.peer_count || '--';
        document.getElementById('hashrate').textContent = data.hashrate ? data.hashrate.toFixed(2) : '--';
    } catch (error) {
        console.error('Error updating live tiles:', error);
    }
}

function updateTimestamp() {
    document.getElementById('update-time').textContent = new Date().toLocaleTimeString();
}