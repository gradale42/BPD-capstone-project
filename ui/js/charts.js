// global variables for charts
let feeRateChart, mempoolChart, bandwidthChart;

// init charts on page load
function initCharts() {
    initFeeRateChart();
    initMempoolChart();
}

// chart for fee rate
function initFeeRateChart() {
    const ctx = document.getElementById('feeRateChart');
    if (!ctx) {
        console.log('feeRateChart canvas not found');
        return;
    }

    feeRateChart = new Chart(ctx.getContext('2d'), {
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

// Live mempool chart (last 24 hours)
function initMempoolChart() {
    const ctx = document.getElementById('mempoolChart');
    if (!ctx) {
        console.log('mempoolChart canvas not found');
        return;
    }

    mempoolChart = new Chart(ctx.getContext('2d'), {
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
                    tension: 0.4,
                    fill: true
                },
                {
                    label: 'vBytes',
                    data: [],
                    borderColor: '#7c3aed',
                    backgroundColor: 'rgba(124, 58, 237, 0.1)',
                    yAxisID: 'y1',
                    tension: 0.4,
                    fill: true
                }
            ]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            plugins: {
                title: {
                    display: true,
                    text: 'Live Mempool Size (Last 24 Hours)'
                },
                tooltip: {
                    callbacks: {
                        label: function(context) {
                            let label = context.dataset.label || '';
                            let value = context.raw;
                            if (context.dataset.label === 'vBytes') {
                                value = (value / 1024 / 1024).toFixed(2);
                                return `${label}: ${value} MB`;
                            }
                            return `${label}: ${value.toLocaleString()}`;
                        }
                    }
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
                    },
                    beginAtZero: true
                },
                y1: {
                    type: 'linear',
                    display: true,
                    position: 'right',
                    title: {
                        display: true,
                        text: 'vBytes (MB)'
                    },
                    grid: {
                        drawOnChartArea: false
                    },
                    beginAtZero: true
                }
            }
        }
    });
}

let mempoolMetricsChart = null;
let currentMempoolMetric = 'all';
let currentMempoolData = { tx_count: [], vbytes: [], avg_feerate: [] };

function initMempoolMetricsChart() {
    const container = document.getElementById('mempool-chart');
    if (!container) {
        console.log('mempool-chart container not found');
        return;
    }

    if (mempoolMetricsChart) {
        mempoolMetricsChart.dispose();
    }

    mempoolMetricsChart = echarts.init(container);

    const option = {
        title: {
            text: 'Mempool Historical Metrics',
            left: 'center'
        },
        tooltip: {
            trigger: 'axis',
            axisPointer: { type: 'shadow' },
            valueFormatter: (value, seriesName) => {
                if (value === undefined || value === null) return 'N/A';
                if (seriesName === 'vBytes (MB)') {
                    return (value / 1024 / 1024).toFixed(2) + ' MB';
                }
                if (seriesName === 'Fee Rate (sat/vB)') {
                    return value.toFixed(2) + ' sat/vB';
                }
                return value.toLocaleString();
            }
        },
        legend: {
            data: ['TX Count', 'vBytes (MB)', 'Fee Rate (sat/vB)'],
            left: 'left',
            orient: 'vertical'
        },
        grid: {
            left: '8%',
            right: '8%',
            bottom: '12%',
            containLabel: true
        },
        toolbox: {
            feature: {
                saveAsImage: {},
                restore: {},
                zoom: {}
            }
        },
        xAxis: {
            type: 'time',
            name: 'Date',
            nameLocation: 'middle',
            nameGap: 30
        },
        yAxis: [
            {
                type: 'value',
                name: 'TX Count',
                position: 'left',
                alignTicks: true
            },
            {
                type: 'value',
                name: 'vBytes (MB)',
                position: 'right',
                alignTicks: true,
                axisLabel: {
                    formatter: (value) => (value / 1024 / 1024).toFixed(0) + 'M'
                }
            },
            {
                type: 'value',
                name: 'Fee Rate (sat/vB)',
                position: 'right',
                offset: 80,
                alignTicks: true,
                axisLabel: {
                    formatter: (value) => value.toFixed(0)
                }
            }
        ],
        series: [
            {
                name: 'TX Count',
                type: 'line',
                smooth: true,
                data: [],
                yAxisIndex: 0,
                lineStyle: { color: '#2563eb', width: 2 },
                areaStyle: { opacity: 0.1, color: '#2563eb' }
            },
            {
                name: 'vBytes (MB)',
                type: 'line',
                smooth: true,
                data: [],
                yAxisIndex: 1,
                lineStyle: { color: '#7c3aed', width: 2 },
                areaStyle: { opacity: 0.1, color: '#7c3aed' }
            },
            {
                name: 'Fee Rate (sat/vB)',
                type: 'line',
                smooth: true,
                data: [],
                yAxisIndex: 2,
                lineStyle: { color: '#f59e0b', width: 2 },
                areaStyle: { opacity: 0.1, color: '#f59e0b' }
            }
        ],
        dataZoom: [
            {
                type: 'inside',
                start: 0,
                end: 100
            },
            {
                type: 'slider',
                start: 0,
                end: 100,
                bottom: 10,
                height: 20,
                brushSelect: true,
                zoomOnMouseWheel: true
            }
        ]
    };

    mempoolMetricsChart.setOption(option);

    // Add resize handler
    window.addEventListener('resize', () => mempoolMetricsChart.resize());

    // Add metric buttons handlers
    document.querySelectorAll('.mempool-metric-btn').forEach(btn => {
        btn.addEventListener('click', () => {
            document.querySelectorAll('.mempool-metric-btn').forEach(b => b.classList.remove('active'));
            btn.classList.add('active');
            currentMempoolMetric = btn.dataset.metric;
            updateMempoolChartVisibility();
        });
    });

    console.log('Mempool historical metrics chart initialized');
}

function updateMempoolChartVisibility() {
    if (!mempoolMetricsChart) return;

    const series = mempoolMetricsChart.getOption().series;

    if (currentMempoolMetric === 'all') {
        // Show all series
        series.forEach(s => s.show = true);
    } else {
        // Show only selected metric
        series.forEach(s => {
            if (s.name === 'TX Count' && currentMempoolMetric === 'tx_count') s.show = true;
            else if (s.name === 'vBytes (MB)' && currentMempoolMetric === 'vbytes') s.show = true;
            else if (s.name === 'Fee Rate (sat/vB)' && currentMempoolMetric === 'avg_feerate') s.show = true;
            else s.show = false;
        });
    }

    mempoolMetricsChart.setOption({ series: series });
}

async function refreshMempoolMetricsChart(startDate, endDate) {
    if (!mempoolMetricsChart) {
        console.log('Chart not initialized, skipping refresh');
        return;
    }

    try {
        const from = Math.floor(startDate.valueOf() / 1000);
        const to = Math.floor(endDate.valueOf() / 1000);
        const url = `/api/v1/mempool/timeseries?from=${from}&to=${to}`;
        console.log('Fetching historical mempool data from:', url);

        const response = await fetch(url);
        if (!response.ok) {
            throw new Error(`HTTP ${response.status}`);
        }

        const data = await response.json();
        console.log('Received historical mempool data:', data.length, 'points');

        if (data && data.length > 0) {
            const txCountData = data.map(item => [item.time * 1000, item.tx_count]);
            const vbytesData = data.map(item => [item.time * 1000, item.vbytes]);
            const feeRateData = data.map(item => [item.time * 1000, item.avg_feerate || 0]);

            mempoolMetricsChart.setOption({
                series: [
                    { data: txCountData, show: currentMempoolMetric === 'all' || currentMempoolMetric === 'tx_count' },
                    { data: vbytesData, show: currentMempoolMetric === 'all' || currentMempoolMetric === 'vbytes' },
                    { data: feeRateData, show: currentMempoolMetric === 'all' || currentMempoolMetric === 'avg_feerate' }
                ]
            });

            console.log('Chart updated with', data.length, 'points');
        } else {
            mempoolMetricsChart.setOption({
                series: [
                    { data: [], show: true },
                    { data: [], show: true },
                    { data: [], show: true }
                ]
            });
        }
    } catch (error) {
        console.error('Error fetching historical mempool timeseries:', error);
    }
}

// Export functions for global access
window.mempoolMetricsChart = mempoolMetricsChart;
window.initMempoolMetricsChart = initMempoolMetricsChart;
window.refreshMempoolMetricsChart = refreshMempoolMetricsChart;

// Log that charts.js is loaded
console.log('charts.js loaded');