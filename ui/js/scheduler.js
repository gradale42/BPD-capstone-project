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
            url: '/api/v1/scheduler/logs',
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

    $('#scheduler-toggle-btn').click(async function() {
        const btn = $(this);
        const isRunning = btn.find('span').text().includes('Stop');
        const action = isRunning ? 'stop' : 'start';

        const interval = $('#scheduler-interval').val();
        const blocksCount = $('#scheduler-blocks-count').val();

        const url = action === 'start'
            ? `/api/v1/scheduler/start?interval_minutes=${interval}&blocks_count=${blocksCount}`
            : '/api/v1/scheduler/stop';

        btn.prop('disabled', true);
        try {
            const response = await fetch(url, { method: 'POST' });
            const data = await response.json();

            if (data.status === 'success') {
                showMessage(`Scheduler ${action}ed`, 'success');
                updateSchedulerStatus();
            } else {
                showMessage('Error: ' + data.message, 'error');
            }
        } catch (err) {
            showMessage('Network error: ' + err.message, 'error');
        } finally {
            btn.prop('disabled', false);
        }
    });

    // Refresh scheduler status on load
    updateSchedulerStatus();

    // Periodically refresh logs and status
    setInterval(() => {
        updateSchedulerStatus();
        schedulerLogsTable.ajax.reload(null, false);
    }, 5000);
}

async function updateSchedulerStatus() {
    try {
        const response = await fetch('/api/v1/scheduler/status');
        const data = await response.json();
        const isRunning = data.running;

        const buttons = ['#scheduler-toggle-btn', '#admin-scheduler-toggle-btn'];
        const pills = ['#scheduler-status-pill', '#admin-scheduler-status-pill'];

        buttons.forEach(selector => {
            const btn = $(selector);
            if (!btn.length) return;

            const btnText = btn.find('span');
            const btnIcon = btn.find('i');

            if (isRunning) {
                btnText.text('Stop Scheduler');
                btnIcon.attr('class', 'fas fa-stop');
                btn.addClass('btn-danger-mode');
            } else {
                btnText.text('Start Scheduler');
                btnIcon.attr('class', 'fas fa-play');
                btn.removeClass('btn-danger-mode');
            }
        });

        pills.forEach(selector => {
            const pill = $(selector);
            if (!pill.length) return;

            if (isRunning) {
                pill.removeClass('status-stopped').addClass('status-started');
                pill.find('.status-text').text('Started');
            } else {
                pill.removeClass('status-started').addClass('status-stopped');
                pill.find('.status-text').text('Stopped');
            }
        });

    } catch (err) {
        console.error('Error updating status:', err);
    }
}

async function loadLogDetails(logId) {
    if (currentSelectedLogId === logId) return;
    currentSelectedLogId = logId;

    try {
        const response = await fetch(`/api/v1/scheduler/log/${logId}`);
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
        console.log(`${type.toUpperCase()}: ${message}`);
    }
}

$(document).on('click', '#admin-scheduler-toggle-btn', function() {
    $('#scheduler-toggle-btn').click(); //Delegate to main toggle button for consistent behavior
});
