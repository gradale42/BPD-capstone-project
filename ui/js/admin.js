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
                const response = await fetch('/api/admin/import-descriptors', { method: 'POST' });
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
                const response = await fetch('/api/admin/mine-blocks?count=100', { method: 'POST' });
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
                const response = await fetch('/api/admin/save-blocks', { method: 'POST' });
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