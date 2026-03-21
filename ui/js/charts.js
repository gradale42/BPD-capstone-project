// global variables for charts
let feeRateChart, mempoolChart, bandwidthChart;

// init charts on page load
function initCharts() {
    initFeeRateChart();
    initMempoolChart();
    initBandwidthChart();

    // Обновление каждые 30 секунд
    setInterval(updateCharts, 30000);
}

// chart for fee rate
function initFeeRateChart() {
    const ctx = document.getElementById('feeRateChart').getContext('2d');
    feeRateChart = new Chart(ctx, {
        type: 'line',
        data: {
            labels: [],
            datasets: [{
                label: 'avg fee rate (sat/vB)',
                data: [],
                borderColor: '#f7931a',
                backgroundColor: 'rgba(247, 147, 26, 0.1)',
                tension: 0.4,
                fill: true
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            plugins: {
                title: {
                    display: true,
                    text: 'avg fee rate over time (last 24 hours)'
                },
                legend: {
                    display: false
                }
            },
            scales: {
                y: {
                    beginAtZero: true,
                    title: {
                        display: true,
                        text: 'sat/vB'
                    }
                }
            }
        }
    });
}

// mempool chart
function initMempoolChart() {
    const ctx = document.getElementById('mempoolChart').getContext('2d');
    mempoolChart = new Chart(ctx, {
        type: 'line',
        data: {
            labels: [],
            datasets: [
                {
                    label: 'TX Count',
                    data: [],
                    borderColor: '#2563eb',
                    backgroundColor: 'rgba(37, 99, 235, 0.1)',
                    yAxisID: 'y',
                    tension: 0.4
                },
                {
                    label: 'vBytes',
                    data: [],
                    borderColor: '#7c3aed',
                    backgroundColor: 'rgba(124, 58, 237, 0.1)',
                    yAxisID: 'y1',
                    tension: 0.4
                }
            ]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            plugins: {
                title: {
                    display: true,
                    text: 'Mempool Size (TX Count & vBytes)'
                }
            },
            scales: {
                y: {
                    type: 'linear',
                    display: true,
                    position: 'left',
                    title: {
                        display: true,
                        text: 'TX Count'
                    }
                },
                y1: {
                    type: 'linear',
                    display: true,
                    position: 'right',
                    title: {
                        display: true,
                        text: 'vBytes'
                    },
                    grid: {
                        drawOnChartArea: false
                    }
                }
            }
        }
    });
}

// bandwidth chart
function initBandwidthChart() {
    const ctx = document.getElementById('bandwidthChart').getContext('2d');
    bandwidthChart = new Chart(ctx, {
        type: 'line',
        data: {
            labels: [],
            datasets: [
                {
                    label: 'Bytes Sent',
                    data: [],
                    borderColor: '#059669',
                    backgroundColor: 'rgba(5, 150, 105, 0.1)',
                    tension: 0.4
                },
                {
                    label: 'Bytes Received',
                    data: [],
                    borderColor: '#dc2626',
                    backgroundColor: 'rgba(220, 38, 38, 0.1)',
                    tension: 0.4
                }
            ]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            plugins: {
                title: {
                    display: true,
                    text: 'Bandwidth (Bytes Sent/Received)'
                }
            },
            scales: {
                y: {
                    beginAtZero: true,
                    title: {
                        display: true,
                        text: 'Bytes'
                    }
                }
            }
        }
    });
}

// data refresh
async function updateCharts() {
    try {
        // hit API endpoint to get historical data for the last 24 hours
        const response = await fetch('/api/stats/historical?hours=24');
        const data = await response.json();

        // refresh charts with new data
        if (feeRateChart && data.blocks) {
            feeRateChart.data.labels = data.blocks.map(b => new Date(b.time * 1000).toLocaleTimeString());
            feeRateChart.data.datasets[0].data = data.blocks.map(b => b.avg_feerate);
            feeRateChart.update();
        }

        // refresh mempool chart
        if (mempoolChart && data.mempool) {
            mempoolChart.data.labels = data.mempool.map(m => new Date(m.timestamp).toLocaleTimeString());
            mempoolChart.data.datasets[0].data = data.mempool.map(m => m.tx_count);
            mempoolChart.data.datasets[1].data = data.mempool.map(m => m.vbytes);
            mempoolChart.update();
        }

        // refresh bandwidth chart
        if (bandwidthChart && data.peers) {
            bandwidthChart.data.labels = data.peers.map(p => new Date(p.timestamp).toLocaleTimeString());
            bandwidthChart.data.datasets[0].data = data.peers.map(p => p.bytes_sent_delta);
            bandwidthChart.data.datasets[1].data = data.peers.map(p => p.bytes_recv_delta);
            bandwidthChart.update();
        }
    } catch (error) {
        console.error('Error updating charts:', error);
    }

    function initCharts() {
        console.log('Charts initialized');
        // Add your chart initialization code here
    }
}