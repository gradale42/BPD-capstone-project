// initialization of DataTables for blocks, mempool history, and peers
let blocksTable, mempoolTable, peersTable;

$(document).ready(function() {
    initTables();
});

function initTables() {

    // blocks table
    blocksTable = $('#blocks-table').DataTable({
        processing: true,
        serverSide: true,
        ajax: {
            url: '/api/blocks',
            type: 'GET',
            data: function(d) {
                return d;
            }
        },
        columns: [
            { data: 'height' },
            {
                data: 'hash',
                render: function(data) {
                    return data.substring(0, 8) + '...' + data.substring(data.length - 8);
                }
            },
            {
                data: 'time',
                render: function(data) {
                    return new Date(data * 1000).toLocaleString();
                }
            },
            { data: 'tx_count' },
            { data: 'avg_fee' },
            { data: 'avg_feerate' },
            { data: 'total_fees' },
            { data: 'difficulty' }
        ],
        order: [[0, 'desc']],
        pageLength: 25,
        responsive: true,
        dom: 'Bfrtip',
        buttons: ['copy', 'csv', 'excel', 'pdf']
    });

    // mempool history table
    mempoolTable = $('#mempool-table').DataTable({
        processing: true,
        serverSide: true,
        ajax: {
            url: '/api/mempool',
            type: 'GET'
        },
        columns: [
            {
                data: 'timestamp',
                render: function(data) {
                    return new Date(data).toLocaleString();
                }
            },
            { data: 'tx_count' },
            { data: 'vbytes' },
            { data: 'total_fees' },
            { data: 'min_relay_feerate' }
        ],
        order: [[0, 'desc']],
        pageLength: 25,
        responsive: true
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
    }, 10000);

    console.log('Tables.js loaded');
}