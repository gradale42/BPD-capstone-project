let schedulerLogsTable;
let currentSelectedLogId = null;

$(document).ready(function() {
    initSchedulerTab();
});

function initSchedulerTab() {
    schedulerLogsTable = $('#scheduler-logs-table').DataTable({
        processing: true,
        serverSide: true,
        ajax: {
            url: '/api/scheduler/logs',
            type: 'GET'
        },
        columns: [
            {
                data: 'start_time',
                render: function(data) {
                    return new Date(data).toLocaleString();
                }
            },
            {
                data: 'end_time',
                render: function(data) {
                    return data ? new Date(data).toLocaleString() : 'Running...';
                }
            },
            { data: 'description' },
            {
                data: 'status',
                render: function(data) {
                    if (data === 'completed') {
                        return '<span class="status-badge" style="background:#28a745;">Completed</span>';
                    } else if (data === 'failed') {
                        return '<span class="status-badge" style="background:#dc3545;">Failed</span>';
                    } else {
                        return '<span class="status-badge" style="background:#ffc107;">Running</span>';
                    }
                }
            }
        ],
        order: [[0, 'desc']],
        pageLength: 25,
        responsive: true,
        rowCallback: function(row, data) {
            $(row).click(function() {
                $('#scheduler-logs-table tbody tr').removeClass('selected');
                $(row).addClass('selected');
                loadLogDetails(data.id);
            });
        }
    });

    // Кнопки управления
    $('#start-scheduler-btn').click(async function() {
        const interval = $('#scheduler-interval').val();
        const blocksCount = $('#scheduler-blocks-count').val();

        try {
            const response = await fetch(`/api/scheduler/start?interval_minutes=${interval}&blocks_count=${blocksCount}`, {
                method: 'POST'
            });
            const data = await response.json();

            if (data.status === 'success') {
                showMessage('Scheduler started', 'success');
                updateSchedulerStatus();
            } else {
                showMessage('Error: ' + data.message, 'error');
            }
        } catch (err) {
            showMessage('Network error: ' + err.message, 'error');
        }
    });

    $('#stop-scheduler-btn').click(async function() {
        try {
            const response = await fetch('/api/scheduler/stop', { method: 'POST' });
            const data = await response.json();

            if (data.status === 'success') {
                showMessage('Scheduler stopped', 'success');
                updateSchedulerStatus();
            } else {
                showMessage('Error: ' + data.message, 'error');
            }
        } catch (err) {
            showMessage('Network error: ' + err.message, 'error');
        }
    });

    // Refresh scheduler status on load
    updateSchedulerStatus();

    // PeRiodically refresh logs and status
    setInterval(() => {
        updateSchedulerStatus();
        schedulerLogsTable.ajax.reload(null, false);
    }, 5000);
}

async function updateSchedulerStatus() {
    try {
        const response = await fetch('/api/scheduler/status');
        const data = await response.json();

        if (data.running) {
            $('#scheduler-status').text('Running').removeClass('stopped').addClass('running');
            $('#start-scheduler-btn').prop('disabled', true);
            $('#stop-scheduler-btn').prop('disabled', false);
        } else {
            $('#scheduler-status').text('Stopped').removeClass('running').addClass('stopped');
            $('#start-scheduler-btn').prop('disabled', false);
            $('#stop-scheduler-btn').prop('disabled', true);
        }
    } catch (err) {
        console.error('Error updating scheduler status:', err);
    }
}

async function loadLogDetails(logId) {
    if (currentSelectedLogId === logId) return;
    currentSelectedLogId = logId;

    try {
        const response = await fetch(`/api/scheduler/log/${logId}`);
        const log = await response.json();

        const resultDetails = $('#log-result-details');

        if (log.result) {
            resultDetails.text(JSON.stringify(log.result, null, 2));
        } else {
            resultDetails.text('No result data available');
        }
    } catch (err) {
        console.error('Error loading log details:', err);
        $('#log-result-details').text('Error loading details: ' + err.message);
    }
}

function showMessage(message, type) {
    const msgDiv = $('#admin-message');
    if (msgDiv.length) {
        msgDiv.text(message);
        msgDiv.removeClass('admin-success admin-error');
        msgDiv.addClass(type === 'success' ? 'admin-success' : 'admin-error');
        setTimeout(() => msgDiv.text(''), 3000);
    } else {
        alert(message);
    }
}