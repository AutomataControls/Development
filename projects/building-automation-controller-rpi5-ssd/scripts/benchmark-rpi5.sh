#!/bin/bash
# Automata Nexus RPi5 Performance Benchmark Script

echo "=== Automata Nexus RPi5 SSD Performance Benchmark ==="
echo "Starting at: $(date)"
echo

# Check if running on RPi5
if ! grep -q "BCM2712" /proc/cpuinfo; then
    echo "Warning: This script is optimized for Raspberry Pi 5"
fi

# Create results directory
RESULTS_DIR="/mnt/ssd/automata-nexus/benchmarks/$(date +%Y%m%d-%H%M%S)"
mkdir -p "$RESULTS_DIR"

echo "Results will be saved to: $RESULTS_DIR"
echo

# 1. System Information
echo "=== System Information ==="
echo "Hostname: $(hostname)"
echo "Kernel: $(uname -r)"
echo "Architecture: $(uname -m)"
echo "CPU Model: $(cat /proc/cpuinfo | grep "Model" | head -1)"
echo "CPU Cores: $(nproc)"
echo "Total RAM: $(free -h | grep Mem | awk '{print $2}')"
echo

# Save full system info
uname -a > "$RESULTS_DIR/system-info.txt"
cat /proc/cpuinfo >> "$RESULTS_DIR/system-info.txt"
free -h >> "$RESULTS_DIR/system-info.txt"

# 2. CPU Performance
echo "=== CPU Performance Test ==="
echo "Running sysbench CPU test..."
sysbench cpu --cpu-max-prime=20000 --threads=4 run > "$RESULTS_DIR/cpu-benchmark.txt" 2>&1
grep "events per second" "$RESULTS_DIR/cpu-benchmark.txt"
echo

# 3. Memory Performance
echo "=== Memory Performance Test ==="
echo "Running memory bandwidth test..."
sysbench memory --memory-total-size=1G --memory-oper=write --threads=4 run > "$RESULTS_DIR/memory-write.txt" 2>&1
sysbench memory --memory-total-size=1G --memory-oper=read --threads=4 run > "$RESULTS_DIR/memory-read.txt" 2>&1
echo "Write speed: $(grep "transferred" "$RESULTS_DIR/memory-write.txt" | awk '{print $4 " " $5}')"
echo "Read speed: $(grep "transferred" "$RESULTS_DIR/memory-read.txt" | awk '{print $4 " " $5}')"
echo

# 4. SSD Performance
echo "=== NVMe SSD Performance Test ==="
if [ -e /dev/nvme0n1 ]; then
    # Get NVMe info
    echo "NVMe Device Information:"
    nvme id-ctrl /dev/nvme0n1 | grep -E "(mn|sn|fr)" > "$RESULTS_DIR/nvme-info.txt"
    
    # Sequential read/write
    echo "Sequential Write (1GB):"
    dd if=/dev/zero of=/mnt/ssd/test.tmp bs=1M count=1024 conv=fsync 2>&1 | tee "$RESULTS_DIR/ssd-seq-write.txt" | grep -E "copied|MB/s"
    
    echo "Sequential Read (1GB):"
    dd if=/mnt/ssd/test.tmp of=/dev/null bs=1M 2>&1 | tee "$RESULTS_DIR/ssd-seq-read.txt" | grep -E "copied|MB/s"
    
    # Random I/O
    echo "Random 4K Write:"
    fio --name=rand-write --ioengine=libaio --rw=randwrite --bs=4k --numjobs=4 --size=256M --runtime=10 --time_based --filename=/mnt/ssd/test.tmp --group_reporting 2>&1 | tee "$RESULTS_DIR/ssd-rand-write.txt" | grep -E "IOPS|bw="
    
    echo "Random 4K Read:"
    fio --name=rand-read --ioengine=libaio --rw=randread --bs=4k --numjobs=4 --size=256M --runtime=10 --time_based --filename=/mnt/ssd/test.tmp --group_reporting 2>&1 | tee "$RESULTS_DIR/ssd-rand-read.txt" | grep -E "IOPS|bw="
    
    # Cleanup
    rm -f /mnt/ssd/test.tmp
else
    echo "No NVMe device found at /dev/nvme0n1"
fi
echo

# 5. SQLite Performance
echo "=== SQLite Performance Test ==="
SQLITE_DB="/mnt/ssd/benchmark.db"
rm -f "$SQLITE_DB"

echo "Testing SQLite write performance..."
sqlite3 "$SQLITE_DB" <<EOF > "$RESULTS_DIR/sqlite-benchmark.txt" 2>&1
-- Apply optimizations
PRAGMA cache_size = 100000;
PRAGMA mmap_size = 1073741824;
PRAGMA page_size = 4096;
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA temp_store = MEMORY;

-- Create test table
CREATE TABLE metrics (
    timestamp INTEGER PRIMARY KEY,
    sensor_id TEXT,
    value REAL,
    status INTEGER
);

-- Test inserts
.timer on

-- Insert 100k records
BEGIN;
WITH RECURSIVE generate_series(value) AS (
    SELECT 1
    UNION ALL
    SELECT value + 1 FROM generate_series WHERE value < 100000
)
INSERT INTO metrics (timestamp, sensor_id, value, status)
SELECT 
    value,
    'sensor_' || (value % 100),
    random() * 100,
    value % 4
FROM generate_series;
COMMIT;

-- Test queries
SELECT COUNT(*) FROM metrics;
SELECT AVG(value) FROM metrics WHERE sensor_id = 'sensor_1';
SELECT sensor_id, MAX(value), MIN(value) FROM metrics GROUP BY sensor_id LIMIT 10;

-- Test updates
BEGIN;
UPDATE metrics SET status = 5 WHERE value > 50;
COMMIT;

-- Cleanup
DROP TABLE metrics;
.quit
EOF

echo "SQLite results saved to $RESULTS_DIR/sqlite-benchmark.txt"
echo

# 6. Application-specific tests
echo "=== Application Performance Tests ==="

# Test Python sensor library performance
echo "Testing Sequent Microsystems library performance..."
python3 <<EOF > "$RESULTS_DIR/sensor-benchmark.txt" 2>&1
import time
import random

# Simulate sensor reads
iterations = 10000
start = time.time()

for i in range(iterations):
    # Simulate I2C read
    value = random.random() * 10  # 0-10V
    
read_time = time.time() - start
reads_per_second = iterations / read_time

print(f"Sensor reads: {reads_per_second:.0f} reads/second")
print(f"Average read time: {(read_time / iterations) * 1000:.3f} ms")
EOF

cat "$RESULTS_DIR/sensor-benchmark.txt"
echo

# 7. Network Performance
echo "=== Network Performance Test ==="
echo "Testing localhost network throughput..."
# Start iperf3 server in background
iperf3 -s -D -1 --logfile "$RESULTS_DIR/iperf-server.log"
sleep 2

# Run client test
iperf3 -c localhost -t 10 -P 4 > "$RESULTS_DIR/network-benchmark.txt" 2>&1
grep -E "sender|receiver" "$RESULTS_DIR/network-benchmark.txt"
echo

# 8. System Load Test
echo "=== System Load Test ==="
echo "Simulating full system load..."

# Function to create CPU load
cpu_load() {
    while true; do
        echo "scale=5000; 4*a(1)" | bc -l > /dev/null
    done
}

# Start CPU load on all cores
for i in $(seq 1 $(nproc)); do
    cpu_load &
    LOAD_PIDS="$LOAD_PIDS $!"
done

# Monitor for 30 seconds
echo "Monitoring system under load for 30 seconds..."
for i in {1..6}; do
    sleep 5
    echo "Time: $((i*5))s - Load: $(uptime | awk -F'load average:' '{print $2}')"
    echo "CPU Temp: $(vcgencmd measure_temp | cut -d= -f2)"
done > "$RESULTS_DIR/load-test.txt"

# Stop load
kill $LOAD_PIDS 2>/dev/null
wait $LOAD_PIDS 2>/dev/null

cat "$RESULTS_DIR/load-test.txt"
echo

# 9. Generate Summary Report
echo "=== Generating Summary Report ==="
cat > "$RESULTS_DIR/summary.md" <<EOF
# Automata Nexus RPi5 Performance Benchmark Results

**Date:** $(date)
**System:** $(hostname) - $(uname -r)

## Key Performance Metrics

### CPU Performance
$(grep "events per second" "$RESULTS_DIR/cpu-benchmark.txt" || echo "N/A")

### Memory Performance
- Write: $(grep "transferred" "$RESULTS_DIR/memory-write.txt" | awk '{print $4 " " $5}' || echo "N/A")
- Read: $(grep "transferred" "$RESULTS_DIR/memory-read.txt" | awk '{print $4 " " $5}' || echo "N/A")

### SSD Performance
- Sequential Write: $(grep "MB/s" "$RESULTS_DIR/ssd-seq-write.txt" | tail -1 || echo "N/A")
- Sequential Read: $(grep "MB/s" "$RESULTS_DIR/ssd-seq-read.txt" | tail -1 || echo "N/A")
- Random 4K Write: $(grep "IOPS" "$RESULTS_DIR/ssd-rand-write.txt" | head -1 || echo "N/A")
- Random 4K Read: $(grep "IOPS" "$RESULTS_DIR/ssd-rand-read.txt" | head -1 || echo "N/A")

### Application Performance
$(cat "$RESULTS_DIR/sensor-benchmark.txt" 2>/dev/null || echo "N/A")

### Network Performance
$(grep -E "sender|receiver" "$RESULTS_DIR/network-benchmark.txt" | head -2 || echo "N/A")

### System Stability
$(tail -3 "$RESULTS_DIR/load-test.txt" || echo "N/A")

## Recommendations

Based on these results:
1. CPU performance is $(grep "events per second" "$RESULTS_DIR/cpu-benchmark.txt" | awk '{if($4 > 5000) print "excellent"; else if($4 > 3000) print "good"; else print "needs optimization"}')
2. SSD performance is $(grep "MB/s" "$RESULTS_DIR/ssd-seq-write.txt" | awk '{if($13 > 400) print "excellent"; else if($13 > 200) print "good"; else print "needs optimization"}')
3. System is $(grep "CPU Temp" "$RESULTS_DIR/load-test.txt" | tail -1 | awk '{if($3 < "70.0") print "thermally stable"; else print "running hot - consider additional cooling"}')

EOF

echo "Summary report saved to: $RESULTS_DIR/summary.md"
echo
echo "=== Benchmark Complete ==="
echo "All results saved to: $RESULTS_DIR"
echo

# Display summary
cat "$RESULTS_DIR/summary.md"