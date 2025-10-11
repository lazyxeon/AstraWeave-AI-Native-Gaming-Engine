// AstraWeave Benchmark Dashboard - Data Visualization & Analysis
// Reads history.jsonl and renders interactive d3.js charts

const HISTORY_FILE = '../../target/benchmark-data/history.jsonl';
const COLOR_SCHEME = {
    ecs: '#4facfe',
    ai: '#00f2fe',
    physics: '#43e97b',
    terrain: '#fa709a',
    input: '#f093fb',
    default: '#a0a0a0'
};

let benchmarkData = [];
let filteredData = [];
let currentFilters = {
    system: 'all',
    timeRange: 30,
    benchmark: null
};

// Parse JSONL file (one JSON object per line)
async function loadBenchmarkData() {
    try {
        const response = await fetch(HISTORY_FILE);
        if (!response.ok) {
            throw new Error(`Failed to load ${HISTORY_FILE}: ${response.statusText}`);
        }
        
        const text = await response.text();
        const lines = text.split('\n').filter(line => line.trim() !== '');
        
        benchmarkData = lines.map(line => {
            const entry = JSON.parse(line);
            entry.timestamp = new Date(entry.timestamp);
            return entry;
        });
        
        console.log(`Loaded ${benchmarkData.length} benchmark entries`);
        
        // Sort by timestamp (oldest first for charting)
        benchmarkData.sort((a, b) => a.timestamp - b.timestamp);
        
        updateFilters();
        renderDashboard();
        
    } catch (error) {
        console.error('Error loading benchmark data:', error);
        showError(error.message);
    }
}

// Extract unique benchmark names and populate select dropdown
function updateFilters() {
    const benchmarkSelect = document.getElementById('benchmark-select');
    
    // Get unique benchmark names
    const uniqueBenchmarks = [...new Set(benchmarkData.map(d => d.benchmark_name))].sort();
    
    benchmarkSelect.innerHTML = '<option value="all">All Benchmarks</option>';
    
    uniqueBenchmarks.forEach(name => {
        const option = document.createElement('option');
        option.value = name;
        option.textContent = name;
        benchmarkSelect.appendChild(option);
    });
    
    // Set event listeners
    document.getElementById('system-filter').addEventListener('change', onFilterChange);
    document.getElementById('time-range').addEventListener('change', onFilterChange);
    document.getElementById('benchmark-select').addEventListener('change', onFilterChange);
}

// Handle filter changes
function onFilterChange() {
    currentFilters.system = document.getElementById('system-filter').value;
    currentFilters.timeRange = parseInt(document.getElementById('time-range').value);
    currentFilters.benchmark = document.getElementById('benchmark-select').value;
    
    renderDashboard();
}

// Apply filters to dataset
function applyFilters() {
    const now = new Date();
    const cutoffDate = new Date(now.getTime() - currentFilters.timeRange * 24 * 60 * 60 * 1000);
    
    filteredData = benchmarkData.filter(d => {
        // Time range filter
        if (d.timestamp < cutoffDate) return false;
        
        // System filter
        if (currentFilters.system !== 'all') {
            const system = detectSystem(d.crate);
            if (system !== currentFilters.system) return false;
        }
        
        // Specific benchmark filter
        if (currentFilters.benchmark && currentFilters.benchmark !== 'all') {
            if (d.benchmark_name !== currentFilters.benchmark) return false;
        }
        
        return true;
    });
    
    console.log(`Filtered to ${filteredData.length} entries`);
}

// Detect system from crate name
function detectSystem(crate) {
    if (crate.includes('core') || crate.includes('ecs') || crate.includes('stress')) return 'ecs';
    if (crate.includes('ai') || crate.includes('behavior')) return 'ai';
    if (crate.includes('physics')) return 'physics';
    if (crate.includes('terrain')) return 'terrain';
    if (crate.includes('input')) return 'input';
    return 'other';
}

// Get color for system
function getSystemColor(crate) {
    const system = detectSystem(crate);
    return COLOR_SCHEME[system] || COLOR_SCHEME.default;
}

// Render entire dashboard
function renderDashboard() {
    applyFilters();
    renderStatCards();
    renderChart();
    updateGeneratedTime();
}

// Render stat cards (summary metrics)
function renderStatCards() {
    const statsGrid = document.getElementById('stats-grid');
    
    if (filteredData.length === 0) {
        statsGrid.innerHTML = '<div class="loading">No data available for selected filters</div>';
        return;
    }
    
    // Group by benchmark and get latest value
    const benchmarkGroups = {};
    
    filteredData.forEach(d => {
        if (!benchmarkGroups[d.benchmark_name]) {
            benchmarkGroups[d.benchmark_name] = [];
        }
        benchmarkGroups[d.benchmark_name].push(d);
    });
    
    // Calculate stats
    const stats = [];
    
    for (const [name, entries] of Object.entries(benchmarkGroups)) {
        // Sort by timestamp (latest last)
        entries.sort((a, b) => a.timestamp - b.timestamp);
        
        const latest = entries[entries.length - 1];
        const oldest = entries[0];
        
        // Calculate change percentage
        const change = ((latest.value - oldest.value) / oldest.value) * 100;
        
        stats.push({
            name: name,
            value: latest.value,
            unit: latest.unit,
            change: change,
            system: detectSystem(latest.crate),
            color: getSystemColor(latest.crate)
        });
    }
    
    // Sort by value (descending) and take top 8
    stats.sort((a, b) => b.value - a.value);
    const topStats = stats.slice(0, 8);
    
    // Render cards
    statsGrid.innerHTML = '';
    
    topStats.forEach(stat => {
        const card = document.createElement('div');
        card.className = `stat-card ${stat.system}`;
        
        const changeIcon = stat.change > 0 ? '▲' : (stat.change < 0 ? '▼' : '─');
        const changeColor = stat.change > 10 ? '#ff6b6b' : (stat.change < -10 ? '#43e97b' : '#a0a0a0');
        
        card.innerHTML = `
            <div class="stat-label">${stat.name}</div>
            <div class="stat-value">
                ${formatNumber(stat.value)}
                <span class="stat-unit">${stat.unit}</span>
            </div>
            <div class="stat-change" style="color: ${changeColor}">
                ${changeIcon} ${Math.abs(stat.change).toFixed(1)}% (${currentFilters.timeRange}d)
            </div>
        `;
        
        statsGrid.appendChild(card);
    });
}

// Format number with appropriate precision
function formatNumber(value) {
    if (value < 10) return value.toFixed(2);
    if (value < 100) return value.toFixed(1);
    if (value < 1000) return value.toFixed(0);
    if (value < 1000000) return (value / 1000).toFixed(1) + 'K';
    return (value / 1000000).toFixed(1) + 'M';
}

// Render d3.js line chart
function renderChart() {
    const chartDiv = document.getElementById('chart');
    chartDiv.innerHTML = ''; // Clear previous chart
    
    if (filteredData.length === 0) {
        chartDiv.innerHTML = '<div class="loading">No data to display</div>';
        return;
    }
    
    // Group data by benchmark name
    const benchmarkGroups = d3.group(filteredData, d => d.benchmark_name);
    
    // Prepare data for charting
    const series = [];
    
    benchmarkGroups.forEach((values, name) => {
        // Sort by timestamp
        values.sort((a, b) => a.timestamp - b.timestamp);
        
        series.push({
            name: name,
            values: values,
            color: getSystemColor(values[0].crate)
        });
    });
    
    // If specific benchmark selected, show only that one
    let chartSeries = series;
    if (currentFilters.benchmark && currentFilters.benchmark !== 'all') {
        chartSeries = series.filter(s => s.name === currentFilters.benchmark);
    }
    
    // Update chart title
    const chartTitle = document.getElementById('chart-title');
    if (currentFilters.benchmark && currentFilters.benchmark !== 'all') {
        chartTitle.textContent = currentFilters.benchmark;
    } else {
        chartTitle.textContent = `All Benchmarks (${chartSeries.length} series)`;
    }
    
    // Chart dimensions
    const margin = { top: 20, right: 120, bottom: 60, left: 80 };
    const width = chartDiv.clientWidth - margin.left - margin.right;
    const height = 500 - margin.top - margin.bottom;
    
    // Create SVG
    const svg = d3.select(chartDiv)
        .append('svg')
        .attr('width', width + margin.left + margin.right)
        .attr('height', height + margin.top + margin.bottom)
        .append('g')
        .attr('transform', `translate(${margin.left},${margin.top})`);
    
    // Scales
    const allValues = chartSeries.flatMap(s => s.values);
    
    const xScale = d3.scaleTime()
        .domain(d3.extent(allValues, d => d.timestamp))
        .range([0, width]);
    
    const yScale = d3.scaleLinear()
        .domain([0, d3.max(allValues, d => d.value) * 1.1])
        .range([height, 0]);
    
    // Grid lines
    svg.append('g')
        .attr('class', 'grid')
        .attr('transform', `translate(0,${height})`)
        .call(d3.axisBottom(xScale).tickSize(-height).tickFormat(''));
    
    svg.append('g')
        .attr('class', 'grid')
        .call(d3.axisLeft(yScale).tickSize(-width).tickFormat(''));
    
    // Axes
    svg.append('g')
        .attr('class', 'axis')
        .attr('transform', `translate(0,${height})`)
        .call(d3.axisBottom(xScale).ticks(6).tickFormat(d3.timeFormat('%m/%d')));
    
    svg.append('g')
        .attr('class', 'axis')
        .call(d3.axisLeft(yScale).ticks(8).tickFormat(d => formatNumber(d)));
    
    // Y-axis label
    svg.append('text')
        .attr('transform', 'rotate(-90)')
        .attr('y', -60)
        .attr('x', -height / 2)
        .attr('text-anchor', 'middle')
        .attr('fill', '#a0a0a0')
        .style('font-size', '14px')
        .text('Time (ns)');
    
    // Line generator
    const line = d3.line()
        .x(d => xScale(d.timestamp))
        .y(d => yScale(d.value))
        .curve(d3.curveMonotoneX);
    
    // Draw lines
    chartSeries.forEach(s => {
        svg.append('path')
            .datum(s.values)
            .attr('class', 'line')
            .attr('d', line)
            .attr('stroke', s.color)
            .attr('stroke-width', 2.5);
    });
    
    // Tooltip
    const tooltip = d3.select('.tooltip');
    
    // Add invisible overlay for mouse tracking
    svg.append('rect')
        .attr('width', width)
        .attr('height', height)
        .attr('fill', 'none')
        .attr('pointer-events', 'all')
        .on('mousemove', function(event) {
            const [mx, my] = d3.pointer(event);
            const x0 = xScale.invert(mx);
            
            // Find closest point
            let closestPoint = null;
            let closestDistance = Infinity;
            
            allValues.forEach(d => {
                const distance = Math.abs(d.timestamp - x0);
                if (distance < closestDistance) {
                    closestDistance = distance;
                    closestPoint = d;
                }
            });
            
            if (closestPoint) {
                tooltip
                    .style('left', (event.pageX + 15) + 'px')
                    .style('top', (event.pageY - 28) + 'px')
                    .classed('visible', true)
                    .html(`
                        <strong>${closestPoint.benchmark_name}</strong><br/>
                        Value: ${formatNumber(closestPoint.value)} ${closestPoint.unit}<br/>
                        Date: ${d3.timeFormat('%Y-%m-%d %H:%M')(closestPoint.timestamp)}<br/>
                        Branch: ${closestPoint.git_branch}<br/>
                        SHA: ${closestPoint.git_sha}
                    `);
            }
        })
        .on('mouseleave', function() {
            tooltip.classed('visible', false);
        });
    
    // Render legend
    renderLegend(chartSeries);
}

// Render legend for chart series
function renderLegend(series) {
    const legendDiv = document.getElementById('legend');
    legendDiv.innerHTML = '';
    
    // Show max 10 items in legend
    const legendItems = series.slice(0, 10);
    
    legendItems.forEach(s => {
        const item = document.createElement('div');
        item.className = 'legend-item';
        
        const color = document.createElement('div');
        color.className = 'legend-color';
        color.style.backgroundColor = s.color;
        
        const label = document.createElement('span');
        label.textContent = s.name;
        
        item.appendChild(color);
        item.appendChild(label);
        
        legendDiv.appendChild(item);
    });
}

// Update generated timestamp
function updateGeneratedTime() {
    const timeSpan = document.getElementById('generated-time');
    timeSpan.textContent = new Date().toLocaleString();
}

// Show error message
function showError(message) {
    const statsGrid = document.getElementById('stats-grid');
    statsGrid.innerHTML = `
        <div class="error">
            <h2>Error Loading Dashboard</h2>
            <p>${message}</p>
            <p>Please run benchmarks and export JSONL data:</p>
            <pre>cargo bench
.\\scripts\\export_benchmark_jsonl.ps1</pre>
        </div>
    `;
}

// Initialize dashboard on page load
document.addEventListener('DOMContentLoaded', () => {
    console.log('Loading AstraWeave Benchmark Dashboard...');
    loadBenchmarkData();
});
