// initialization of DataTables for blocks, mempool history, and peers
let blocksTable, mempoolTable, peersTable;
let currentMode = 'live';

$(document).ready(function() {
    initTables();
});

function initTables() {
    blocksTable = $('#blocks-table').DataTable({
        processing: true,
        serverSide: true,
        ajax: function(data, callback, settings) {
            data.mode = currentMode;
            $.ajax({
                url: '/api/blocks',
                type: 'GET',
                data: data,
                success: function(response) {
                    callback(response);
                },
                error: function(xhr, error, thrown) {
                    console.error('DataTable error:', error);
                    callback({ draw: data.draw, data: [], recordsTotal: 0, recordsFiltered: 0 });
                }
            });
        },
        columns: [
            {
                data: 'height',
                render: function(data, type, row) {
                    if (type === 'display') {
                        return `<span class="block-link" style="cursor:pointer;color:#f7931a;" data-height="${data}" data-hash="${row.hash}">${data}</span>`;
                    }
                    return data;
                }
            },
            {
                data: 'hash',
                render: function(data, type, row) {
                    if (type === 'display') {
                        const shortHash = data.substring(0, 8) + '...' + data.substring(data.length - 8);
                        return `<span class="block-link" style="cursor:pointer;color:#f7931a;" data-height="${row.height}" data-hash="${data}" title="${data}">${shortHash}</span>`;
                    }
                    return data;
                }
            },
            {
                data: 'time',
                render: function(data) {
                    return new Date(data * 1000).toLocaleString();
                }
            },
            { data: 'tx_count' },
            { data: 'avg_fee_sat' },
            { data: 'avg_feerate' },
            { data: 'total_fees_sat' },
            { data: 'difficulty' }
        ],
        order: [[0, 'desc']],
        pageLength: 25,
        responsive: true,
        dom: 'Bfrtip',
        buttons: ['copy', 'csv', 'excel', 'pdf'],
        drawCallback: function() {
            // Attach click handlers to all block links
            document.querySelectorAll('.block-link').forEach(el => {
                el.removeEventListener('click', handleBlockClick);
                el.addEventListener('click', handleBlockClick);
            });
        }
    });

    $('#mode-live').on('click', function() {
        if (currentMode === 'live') return;
        console.log('Switching to LIVE mode');
        currentMode = 'live';
        $('#mode-live').addClass('active');
        $('#mode-index').removeClass('active');
        blocksTable.ajax.reload();
    });

    $('#mode-index').on('click', function() {
        if (currentMode === 'index') return;
        console.log('Switching to INDEX mode');
        currentMode = 'index';
        $('#mode-index').addClass('active');
        $('#mode-live').removeClass('active');
        blocksTable.ajax.reload();
    });

    // mempool table
    mempoolTable = $('#mempool-table').DataTable({
        processing: true,
        serverSide: true,
        ajax: {
            url: '/api/mempool/transactions',
            type: 'GET',
            data: function(d) {
                // Add sorting information
                if (d.order && d.order.length) {
                    d.order = d.order.map(order => [order.column, order.dir]);
                }
                return d;
            }
        },
        columns: [
            {
                data: 'txid',
                render: function(data, type, row) {
                    if (type === 'display') {
                        const shortTxid = data.substring(0, 8) + '...' + data.substring(data.length - 8);
                        return `<span class="tx-link" style="cursor:pointer;color:#f7931a;" data-txid="${data}" title="${data}">${shortTxid}</span>`;
                    }
                    return data;
                }
            },
            {
                data: 'vsize',
                render: function(data) {
                    return data.toLocaleString();
                }
            },
            {
                data: 'weight',
                render: function(data) {
                    return data.toLocaleString();
                }
            },
            {
                data: 'time',
                render: function(data) {
                    return new Date(data * 1000).toLocaleString();
                }
            },
            { data: 'height' },
            {
                data: 'fee',
                render: function(data) {
                    return data.toFixed(8);
                }
            },
            {
                data: 'fee_rate',
                render: function(data) {
                    return data.toFixed(2);
                }
            },
            { data: 'ancestor_count' },
            { data: 'descendant_count' },
            {
                data: 'bip125_replaceable',
                render: function(data) {
                    return data ? '<span class="rbf-yes" style="color:#ff9800;">✓ RBF</span>' : '<span class="rbf-no" style="color:#888;">—</span>';
                }
            }
        ],
        order: [[6, 'desc']], // Sort by fee rate (column index 6)
        pageLength: 25,
        responsive: true,
        dom: 'Bfrtip',
        buttons: ['copy', 'csv', 'excel', 'pdf'],
        columnDefs: [
            { targets: [0], orderable: true },
            { targets: [1], orderable: true },
            { targets: [2], orderable: true },
            { targets: [3], orderable: true },
            { targets: [4], orderable: true },
            { targets: [5], orderable: true },
            { targets: [6], orderable: true },
            { targets: [7], orderable: true },
            { targets: [8], orderable: true },
            { targets: [9], orderable: true }
        ]
    });

    // peers table
    peersTable = $('#peers-table').DataTable({
        processing: true,
        serverSide: true,
        ajax: {
            url: '/api/peers',
            type: 'GET'
        },
        columns: [
            { data: 'peer_id' },
            {
                data: 'inbound',
                render: function(data) {
                    return data ? 'Inbound' : 'Outbound';
                }
            },
            { data: 'subver' },
            { data: 'version' },
            { data: 'bytes_sent_total' },
            { data: 'bytes_recv_total' },
            { data: 'bytes_sent_delta' },
            { data: 'bytes_recv_delta' },
            { data: 'ping' }
        ],
        order: [[7, 'desc']], // sort by recent activity (bytes received delta)
        pageLength: 25,
        responsive: true,
        rowCallback: function(row, data) {
            // peers with recent activity (bytes sent/received in the last interval) are highlighted
            if (data.bytes_sent_delta > 0 || data.bytes_recv_delta > 0) {
                $(row).addClass('active-peer');
            }
        }
    });

    // refresh tables every 10 seconds
    setInterval(() => {
        blocksTable.ajax.reload(null, false);
        peersTable.ajax.reload(null, false);
        mempoolTable.ajax.reload(null, false);
        updateMempoolStats();
    }, 10000);

    console.log('Tables.js loaded, currentMode =', currentMode);
}

function handleBlockClick(e) {
    e.preventDefault();
    e.stopPropagation();
    const height = this.getAttribute('data-height');
    const hash = this.getAttribute('data-hash');
    console.log(`Opening block details: height=${height}, hash=${hash}`);
    if (typeof showBlockDetails === 'function') {
        showBlockDetails(height, hash);
    } else {
        console.error('showBlockDetails function not found');
    }
    return false;
}