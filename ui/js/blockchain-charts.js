let blockchainChart = null;
let currentBlockchainMetric = 'all';  // 'all', 'tx_count', 'avg_fee_sat', 'avg_feerate'
let currentBlockchainData = { tx_count: [], avg_fee_sat: [], avg_feerate: [] };

async function fetchBlockchainData(fromTimestamp, toTimestamp) {
    const url = `/api/blocks/timeseries?from=${Math.floor(fromTimestamp / 1000)}&to=${Math.floor(toTimestamp / 1000)}`;
    try {
        const response = await fetch(url);
        if (!response.ok) throw new Error(`HTTP ${response.status}`);
        const data = await response.json();
        return data;
    } catch (err) {
        console.error('Failed to fetch blockchain timeseries:', err);
        return [];
    }
}

function initBlockchainCharts() {
    const container = document.getElementById('blockchain-chart');
    if (!container) {
        console.log('blockchain-chart container not found');
        return;
    }

    blockchainChart = echarts.init(container);

    const option = {
        title: {
            text: 'Blockchain Metrics',
            left: 'center'
        },
        tooltip: {
            trigger: 'axis',
            axisPointer: { type: 'shadow' },
            valueFormatter: (value, seriesName) => {
                if (value === undefined || value === null) return 'N/A';
                if (seriesName === 'Avg Fee (sat)') {
                    return value.toFixed(0) + ' sat';
                }
                if (seriesName === 'Fee Rate (sat/vB)') {
                    return value.toFixed(2) + ' sat/vB';
                }
                return value.toLocaleString();
            }
        },
        legend: {
            data: ['Transactions', 'Avg Fee (sat)', 'Fee Rate (sat/vB)'],
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
                name: 'Transaction Count',
                position: 'left',
                alignTicks: true
            },
            {
                type: 'value',
                name: 'Avg Fee (sat)',
                position: 'right',
                alignTicks: true,
                axisLabel: {
                    formatter: (value) => value.toFixed(0)
                }
            },
            {
                type: 'value',
                name: 'Fee Rate (sat/vB)',
                position: 'right',
                offset: 80,
                alignTicks: true,
                axisLabel: {
                    formatter: (value) => value.toFixed(2)
                }
            }
        ],
        series: [
            {
                name: 'Transactions',
                type: 'line',
                smooth: true,
                data: [],
                yAxisIndex: 0,
                lineStyle: { color: '#3b82f6', width: 2 },
                areaStyle: { opacity: 0.1, color: '#3b82f6' }
            },
            {
                name: 'Avg Fee (sat)',
                type: 'line',
                smooth: true,
                data: [],
                yAxisIndex: 1,
                lineStyle: { color: '#10b981', width: 2 },
                areaStyle: { opacity: 0.1, color: '#10b981' }
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
            { type: 'inside', start: 0, end: 100 },
            { type: 'slider', start: 0, end: 100, bottom: 10, height: 20, brushSelect: true, zoomOnMouseWheel: true }
        ]
    };

    blockchainChart.setOption(option);
    window.addEventListener('resize', () => blockchainChart.resize());

    // Setup metric buttons
    document.querySelectorAll('.metric-btn').forEach(btn => {
        btn.addEventListener('click', () => {
            document.querySelectorAll('.metric-btn').forEach(b => b.classList.remove('active'));
            btn.classList.add('active');
            currentBlockchainMetric = btn.dataset.metric;
            updateBlockchainChartVisibility();
        });
    });

    console.log('Blockchain chart initialized');
}

function updateBlockchainChartVisibility() {
    if (!blockchainChart) return;

    const series = blockchainChart.getOption().series;

    if (currentBlockchainMetric === 'all') {
        series.forEach(s => s.show = true);
    } else {
        series.forEach(s => {
            if (s.name === 'Transactions' && currentBlockchainMetric === 'tx_count') s.show = true;
            else if (s.name === 'Avg Fee (sat)' && currentBlockchainMetric === 'avg_fee_sat') s.show = true;
            else if (s.name === 'Fee Rate (sat/vB)' && currentBlockchainMetric === 'avg_feerate') s.show = true;
            else s.show = false;
        });
    }

    blockchainChart.setOption({ series: series });
}

async function refreshBlockchainChart(startDate, endDate) {
    if (!blockchainChart) {
        console.log('Blockchain chart not initialized');
        return;
    }

    try {
        const from = Math.floor(startDate.valueOf() / 1000);
        const to = Math.floor(endDate.valueOf() / 1000);
        const url = `/api/blocks/timeseries?from=${from}&to=${to}`;
        console.log('Fetching blockchain data from:', url);

        const response = await fetch(url);
        if (!response.ok) throw new Error(`HTTP ${response.status}`);

        const data = await response.json();
        console.log('Received blockchain data:', data.length, 'points');

        if (data && data.length > 0) {
            const txCountData = data.map(item => [item.time * 1000, item.tx_count]);
            const avgFeeData = data.map(item => [item.time * 1000, item.avg_fee_sat]);
            const feeRateData = data.map(item => [item.time * 1000, item.avg_feerate]);

            blockchainChart.setOption({
                series: [
                    { data: txCountData, show: currentBlockchainMetric === 'all' || currentBlockchainMetric === 'tx_count' },
                    { data: avgFeeData, show: currentBlockchainMetric === 'all' || currentBlockchainMetric === 'avg_fee_sat' },
                    { data: feeRateData, show: currentBlockchainMetric === 'all' || currentBlockchainMetric === 'avg_feerate' }
                ]
            });

            console.log('Blockchain chart updated with', data.length, 'points');
        } else {
            blockchainChart.setOption({
                series: [
                    { data: [], show: true },
                    { data: [], show: true },
                    { data: [], show: true }
                ]
            });
        }
    } catch (error) {
        console.error('Error fetching blockchain timeseries:', error);
    }
}

// Initialize with date picker
document.addEventListener('DOMContentLoaded', () => {
    if (typeof echarts !== 'undefined') {
        initBlockchainCharts();

        // Initialize date range picker
        const picker = new DateRangePeeker('blockchain-daterange', refreshBlockchainChart);
        const range = picker.getCurrentRange();
        refreshBlockchainChart(range.startDate, range.endDate);
    } else {
        console.warn('ECharts not loaded yet');
    }
});