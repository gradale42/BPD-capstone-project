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

    // JSON view tabs switching (for 4 tabs)
    const tabBtns = document.querySelectorAll('.json-tab-btn');
    tabBtns.forEach(btn => {
        btn.addEventListener('click', function() {
            const view = this.getAttribute('data-view');

            // Remove active class from all buttons
            tabBtns.forEach(b => b.classList.remove('active'));
            // Add active class to clicked button
            this.classList.add('active');

            // Hide all panes
            document.querySelectorAll('.json-viewer-pane').forEach(pane => {
                pane.classList.remove('active');
            });

            // Show selected pane
            if (view === 'tree') {
                document.getElementById('block-json-tree').classList.add('active');
            } else if (view === 'text') {
                document.getElementById('block-json-text').classList.add('active');
            }
        });
    });

    // Make modal resizable (jQuery UI)
    const modalContent = $('.modal-content');
    modalContent.resizable({
        handles: 'se',
        minHeight: 250,
        minWidth: 300
    });
});

window.showBlockDetails = async function(height, hash) {
    const modal = document.getElementById('block-modal');
    const blockHeightSpan = document.getElementById('block-height');
    const blockHashSpan = document.getElementById('block-hash');
    const blockTimeSpan = document.getElementById('block-time');
    const blockTxCountSpan = document.getElementById('block-tx-count');

    const treeContainer = document.getElementById('block-json-tree');
    const textContainer = document.getElementById('block-json-text');

    if (!modal) return;

    modal.style.display = "block";
    blockHeightSpan.textContent = height;
    blockHashSpan.textContent = hash;
    blockTimeSpan.textContent = 'Loading...';
    blockTxCountSpan.textContent = 'Loading...';

    const loadingHtml = '<div class="loading">Loading block data...</div>';
    treeContainer.innerHTML = loadingHtml;
    textContainer.innerHTML = loadingHtml;

    try {
        const response = await fetch(`/api/block/${hash}`);
        const result = await response.json();
        if (result.status !== 'success') throw new Error(result.message);
        const blockData = result.data || result.block;
        if (!blockData) throw new Error('No block data');

        blockTimeSpan.textContent = blockData.time ? new Date(blockData.time * 1000).toLocaleString() : 'Unknown';
        blockTxCountSpan.textContent = blockData.tx_count ?? blockData.transactions?.length ?? 'Unknown';

        // 1. Tree Component @andypf/json-viewer
        treeContainer.innerHTML = '';
        if (customElements.get('andypf-json-viewer')) {
            try {
                const viewer = document.createElement('andypf-json-viewer');
                viewer.setAttribute('data', JSON.stringify(blockData));
                viewer.setAttribute('expanded', '2'); // Expand to level 2 by default
                viewer.setAttribute('show-toolbar', 'true'); // Show toolbar for expand/collapse
                treeContainer.appendChild(viewer);
            } catch (err) {
                console.error('Tree Component error:', err);
                treeContainer.innerHTML = `<pre>${JSON.stringify(blockData, null, 2)}</pre>`;
            }
        } else {
            console.warn('Tree Component not loaded');
            treeContainer.innerHTML = `<pre>${JSON.stringify(blockData, null, 2)}</pre>`;
        }

        // 2. Text view
        textContainer.innerHTML = `<pre>${JSON.stringify(blockData, null, 2)}</pre>`;

    } catch (err) {
        const errMsg = `<div class="error-message">${err.message}</div>`;
        treeContainer.innerHTML = errMsg;
        textContainer.innerHTML = errMsg;
    }
};