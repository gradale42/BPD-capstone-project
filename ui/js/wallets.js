// Wallets tab functionality
let addressesDataTable = null;

document.addEventListener('DOMContentLoaded', function() {
    initWalletsTab();
});

async function initWalletsTab() {
    const walletSelect = document.getElementById('wallet-select');
    if (!walletSelect) return;

    // Load wallets when tab is shown
    const tabButtons = document.querySelectorAll('.tab-button');
    tabButtons.forEach(button => {
        if (button.getAttribute('data-tab') === 'wallets') {
            button.addEventListener('click', async () => {
                await loadWalletsList();
            });
        }
    });

    // Setup wallet selector change event
    walletSelect.addEventListener('change', async (e) => {
        const walletName = e.target.value;
        if (walletName) {
            await loadWalletDetails(walletName);
            await loadWalletDescriptors(walletName);
        }
    });

    // Setup refresh button
    const refreshBtn = document.getElementById('refresh-wallets-btn');
    if (refreshBtn) {
        refreshBtn.addEventListener('click', async () => {
            await loadWalletsList();
            const selectedWallet = walletSelect.value;
            if (selectedWallet) {
                await loadWalletDetails(selectedWallet);
                await loadWalletDescriptors(selectedWallet);
            }
        });
    }

    // Initial load
    await loadWalletsList();
}

async function loadWalletsList() {
    const walletSelect = document.getElementById('wallet-select');
    if (!walletSelect) return;

    try {
        walletSelect.innerHTML = '<option value="">Loading wallets...</option>';

        const response = await fetch('/api/v1/wallets');
        const data = await response.json();

        if (data.status === 'success' && data.wallets) {
            walletSelect.innerHTML = '<option value="">Select a wallet</option>';
            data.wallets.forEach(wallet => {
                const option = document.createElement('option');
                option.value = wallet.name;
                option.textContent = `${wallet.name} (${wallet.balance.toFixed(8)} BTC)`;
                walletSelect.appendChild(option);
            });
        } else {
            walletSelect.innerHTML = '<option value="">No wallets found</option>';
        }
    } catch (error) {
        console.error('Error loading wallets:', error);
        walletSelect.innerHTML = '<option value="">Error loading wallets</option>';
    }
}

async function loadWalletDetails(walletName) {
    try {
        const response = await fetch(`/api/v1/wallets/${walletName}`);
        const data = await response.json();

        if (data.status === 'success' && data.wallet) {
            const wallet = data.wallet;

            // Update summary
            document.getElementById('wallet-name-display').textContent = wallet.name;
            document.getElementById('wallet-balance').textContent = `${wallet.balance.toFixed(8)} BTC`;
            document.getElementById('address-count').textContent = wallet.address_count;

            // Update addresses table
            updateAddressesTable(wallet.addresses);
        } else {
            console.error('Error loading wallet details:', data.message);
            document.getElementById('addresses-table-body').innerHTML =
                '<tr><td colspan="4">Error loading addresses</td></tr>';
        }
    } catch (error) {
        console.error('Error loading wallet details:', error);
        document.getElementById('addresses-table-body').innerHTML =
            '<tr><td colspan="4">Error loading addresses</td></tr>';
    }
}

function updateAddressesTable(addresses) {
    const tbody = document.getElementById('addresses-table-body');

    if (!addresses || addresses.length === 0) {
        tbody.innerHTML = '<tr><td colspan="4">No addresses found in this wallet</td></tr>';
        return;
    }

    tbody.innerHTML = '';
    addresses.forEach(addr => {
        const row = tbody.insertRow();
        row.insertCell(0).textContent = addr.address;
        row.insertCell(1).textContent = addr.balance.toFixed(8);
        row.insertCell(2).textContent = addr.label || '—';
        row.insertCell(3).textContent = addr.received.toFixed(8);
    });
}

async function loadWalletDescriptors(walletName) {
    const descriptorsDiv = document.getElementById('descriptors-content');
    if (!descriptorsDiv) return;

    try {
        descriptorsDiv.innerHTML = '<div class="loading">Loading descriptors...</div>';

        const response = await fetch(`/api/v1/wallets/${walletName}/descriptors`);
        const data = await response.json();

        if (data.status === 'success' && data.descriptors) {
            if (data.descriptors.length === 0) {
                descriptorsDiv.innerHTML = '<p>No descriptors found for this wallet</p>';
                return;
            }

            descriptorsDiv.innerHTML = '';
            data.descriptors.forEach(desc => {
                const descDiv = document.createElement('div');
                descDiv.className = 'descriptor-item';

                const activeBadge = desc.active ?
                    '<span class="descriptor-active">✓ Active</span>' :
                    '<span>Inactive</span>';

                descDiv.innerHTML = `
                    <div class="descriptor-string">${escapeHtml(desc.descriptor)}</div>
                    <div class="descriptor-meta">
                        ${activeBadge} | 
                        Timestamp: ${new Date(desc.timestamp * 1000).toLocaleString()}
                        ${desc.range ? ` | Range: [${desc.range[0]}, ${desc.range[1]}]` : ''}
                    </div>
                `;
                descriptorsDiv.appendChild(descDiv);
            });
        } else {
            descriptorsDiv.innerHTML = `<div class="error-message">Error: ${data.message || 'Failed to load descriptors'}</div>`;
        }
    } catch (error) {
        console.error('Error loading descriptors:', error);
        descriptorsDiv.innerHTML = '<div class="error-message">Error loading descriptors</div>';
    }
}

// Helper function to escape HTML
function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}