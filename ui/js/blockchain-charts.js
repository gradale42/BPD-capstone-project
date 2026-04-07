let blockchainChart = null;
let currentMetric = 'tx_count';
let currentData = [];

async function fetchBlockchainData(fromTimestamp, toTimestamp) {
    const statusSpan = document.getElementById('blockchain-status-text');
    const pointsSpan = document.getElementById('blockchain-data-points');

    statusSpan.innerText = 'Loading...';
    pointsSpan.innerText = '⏳ fetching';

    const url = `/api/blocks/timeseries?from=${Math.floor(fromTimestamp / 1000)}&to=${Math.floor(toTimestamp / 1000)}`;
    try {
        const response = await fetch(url);
        if (!response.ok) throw new Error(`HTTP ${response.status}`);
        const data = await response.json();
        currentData = data;
        statusSpan.innerText = 'Connected';
        pointsSpan.innerText = `📊 ${data.length} points`;
        return data;
    } catch (err) {
        console.error('Failed to fetch blockchain timeseries:', err);
        statusSpan.innerText = 'Error';
        pointsSpan.innerText = '⚠️ failed';
        return [];
    }
}

function renderBlockchainChart() {
    if (!blockchainChart || !currentData.length) return;

    const chartData = currentData.map(item => [item.time * 1000, item[currentMetric]]);
    const metricTitles = {
        tx_count: 'Transaction count per block',
        avg_fee_sat: 'Average fee (satoshis)',
        avg_feerate: 'Fee rate (sat/vByte)'
    };
    const colors = {
        tx_count: '#3b82f6',
        avg_fee_sat: '#10b981',
        avg_feerate: '#f59e0b'
    };

    blockchainChart.setOption({
        //backgroundColor: '#1e1e2f',
        tooltip: {
            trigger: 'axis',
            valueFormatter: (value) => value?.toLocaleString() ?? 'N/A'
        },
        xAxis: { type: 'time', name: 'Date' },
        yAxis: { type: 'value', name: metricTitles[currentMetric] },
        series: [{
            data: chartData,
            type: 'line',
            smooth: true,
            lineStyle: { width: 2, color: colors[currentMetric] },
            areaStyle: { opacity: 0.1 }
        }],
        dataZoom: [{ type: 'inside' }, { type: 'slider' }]
    });
}

async function refreshBlockchainChart(startDate, endDate) {
    await fetchBlockchainData(startDate.valueOf(), endDate.valueOf());
    renderBlockchainChart();
}

function initBlockchainCharts() {
    blockchainChart = echarts.init(document.getElementById('blockchain-chart'));
    window.addEventListener('resize', () => blockchainChart.resize());

    // Date range picker
    const picker = new DateRangePeeker('blockchain-daterange', refreshBlockchainChart);

    // Metric buttons
    document.querySelectorAll('.metric-btn').forEach(btn => {
        btn.addEventListener('click', () => {
            document.querySelectorAll('.metric-btn').forEach(b => b.classList.remove('active'));
            btn.classList.add('active');
            currentMetric = btn.dataset.metric;
            renderBlockchainChart();
        });
    });

    // Initial load
    const range = picker.getCurrentRange();
    refreshBlockchainChart(range.startDate, range.endDate);
}

// Wait for DOM and ECharts to be ready
document.addEventListener('DOMContentLoaded', () => {
    if (typeof echarts !== 'undefined') {
        initBlockchainCharts();
    } else {
        console.warn('ECharts not loaded yet');
    }
});