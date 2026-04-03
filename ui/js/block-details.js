// Block details modal functionality
$(document).ready(function() {
    // Get modal elements
    const modal = document.getElementById('block-modal');
    const closeBtn = document.getElementsByClassName('close')[0];

    // Close modal when clicking X
    if (closeBtn) {
        closeBtn.onclick = function() {
            modal.style.display = "none";
        }
    }

    // Close modal when clicking outside
    window.onclick = function(event) {
        if (event.target == modal) {
            modal.style.display = "none";
        }
    }

    // Close on Escape key
    document.addEventListener('keydown', function(event) {
        if (event.key === "Escape" && modal.style.display === "block") {
            modal.style.display = "none";
        }
    });
});

// Function to show block details
window.showBlockDetails = async function(height, hash) {
    const modal = document.getElementById('block-modal');
    const blockHeightSpan = document.getElementById('block-height');
    const blockHashSpan = document.getElementById('block-hash');
    const blockTimeSpan = document.getElementById('block-time');
    const blockTxCountSpan = document.getElementById('block-tx-count');
    const jsonViewer = document.getElementById('block-json-viewer');

    if (!modal) {
        console.error('Modal element not found');
        return;
    }

    // Show modal with loading state
    modal.style.display = "block";
    blockHeightSpan.textContent = height;
    blockHashSpan.textContent = hash;
    blockTimeSpan.textContent = 'Loading...';
    blockTxCountSpan.textContent = 'Loading...';
    jsonViewer.innerHTML = '<div class="loading">Loading block data...</div>';

    try {
        const response = await fetch(`/api/block/${hash}`);
        const result = await response.json();

        console.log('Block data received:', result);

        if (result.status === 'success') {
            const blockData = result.data || result.block;

            if (blockData) {
                if (blockData.time) {
                    blockTimeSpan.textContent = new Date(blockData.time * 1000).toLocaleString();
                } else {
                    blockTimeSpan.textContent = 'Unknown';
                }

                if (blockData.tx_count !== undefined) {
                    blockTxCountSpan.textContent = blockData.tx_count;
                } else if (blockData.transactions) {
                    blockTxCountSpan.textContent = blockData.transactions.length;
                } else {
                    blockTxCountSpan.textContent = 'Unknown';
                }

                jsonViewer.innerHTML = '';

                if (typeof $(jsonViewer).jsonViewer === 'function') {
                    $(jsonViewer).jsonViewer(blockData, {
                        collapsed: true,
                        withQuotes: false
                    });
                } else {
                    jsonViewer.innerHTML = `<pre>${JSON.stringify(blockData, null, 2)}</pre>`;
                }
            } else {
                jsonViewer.innerHTML = '<div class="error-message">No block data received</div>';
            }
        } else {
            jsonViewer.innerHTML = `<div class="error-message">Error: ${result.message || 'Failed to load block data'}</div>`;
        }
    } catch (error) {
        console.error('Error loading block details:', error);
        jsonViewer.innerHTML = `<div class="error-message">Error loading block data: ${error.message}</div>`;
    }
};