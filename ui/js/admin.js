// Wait for DOM to be ready
document.addEventListener('DOMContentLoaded', function() {
    const importBtn = document.getElementById('import-descriptors-btn');
    const mineBtn = document.getElementById('mine-blocks-btn');

    if (importBtn) {
        importBtn.addEventListener('click', async () => {
            const msgDiv = document.getElementById('admin-message');
            msgDiv.textContent = 'Importing descriptors...';
            msgDiv.className = 'admin-message'; // reset style
            try {
                const response = await fetch('/api/v1/admin/import-descriptors', { method: 'POST' });
                const data = await response.json();
                msgDiv.textContent = data.message;
                msgDiv.className = data.status === 'success' ? 'admin-success' : 'admin-error';
            } catch (err) {
                msgDiv.textContent = 'Network error: ' + err.message;
                msgDiv.className = 'admin-error';
            }
        });
    }

    if (mineBtn) {
        mineBtn.addEventListener('click', async () => {
            const msgDiv = document.getElementById('admin-message');
            msgDiv.textContent = 'Mining 100 blocks...';
            msgDiv.className = 'admin-message';
            try {
                const response = await fetch('/api/v1/admin/mine-blocks?count=100', { method: 'POST' });
                const data = await response.json();
                msgDiv.textContent = data.message;
                msgDiv.className = data.status === 'success' ? 'admin-success' : 'admin-error';
            } catch (err) {
                msgDiv.textContent = 'Network error: ' + err.message;
                msgDiv.className = 'admin-error';
            }
        });
    }

    const saveBlocksBtn = document.getElementById('save-blocks-btn');
    if (saveBlocksBtn) {
        saveBlocksBtn.addEventListener('click', async () => {
            const msgDiv = document.getElementById('admin-message');
            msgDiv.textContent = 'Saving blocks to database...';
            msgDiv.className = 'admin-message';
            try {
                const response = await fetch('/api/v1/admin/save-blocks', { method: 'POST' });
                const data = await response.json();
                msgDiv.textContent = data.message;
                msgDiv.className = data.status === 'success' ? 'admin-success' : 'admin-error';
            } catch (err) {
                msgDiv.textContent = 'Network error: ' + err.message;
                msgDiv.className = 'admin-error';
            }
        });
    }
});

function updateIndexerProgress(percent, label = "Syncing blocks...") {
    const container = $('#indexer-progress-container');
    const fill = $('#indexer-progress-fill');
    const labelEl = $('#progress-label');
    const percentEl = $('#progress-percent');

    container.show();
    fill.css('width', percent + '%');
    percentEl.text(percent + '%');
    labelEl.text(label);

    if (percent >= 100) {
        labelEl.text("Sync Complete!");
        setTimeout(() => {
            container.fadeOut();
            fill.css('width', '0%');
        }, 3000);
    }
}

$('#save-blocks-btn').click(async function() {
    try {
        const response = await fetch('/api/v1/admin/save-blocks', { method: 'POST' });
        const result = await response.json();

        if (result.status === 'success') {

            startProgressPolling();
        }
    } catch (err) {
        console.error('Failed to start indexing:', err);
    }
});

function startProgressPolling() {
    const interval = setInterval(async () => {
        try {
            const res = await fetch('/api/v1/admin/save-progress'); // Ваш эндпоинт со статусом
            const data = await res.json();

            updateIndexerProgress(data.percent, data.message);

            if (data.percent >= 100 || data.status === 'error') {
                clearInterval(interval);
            }
        } catch (e) {
            clearInterval(interval);
        }
    }, 1000);
}
