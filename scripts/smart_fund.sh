#!/bin/bash
# smart_fund.sh - Wallet initialization and coin distribution
#
# What this script does:
# 1. Creates miner_wallet and student_wallet if they don't exist
# 2. Imports all descriptors from student_wallet.json into student_wallet
# 3. Generates 101 blocks to make coinbase rewards spendable
# 4. Generates all student addresses (legacy, p2sh-segwit, bech32, bech32m) from descriptors
# 5. Distributes 10 BTC across all student addresses (each gets ~0.011 BTC)
# 6. Generates blocks to confirm transactions
# 7. Creates a flag file (.initialized) to skip re-initialization on subsequent runs
#
# Run this script AFTER docker-compose up, BEFORE starting the Rust dashboard

set -e

# Configuration
RPC_USER="alice"
RPC_PASSWORD="password"
CONTAINER="bitcoind-regtest"
WALLET_MINER="miner_wallet"
WALLET_STUDENT="student_wallet"
DESCRIPTORS_FILE="./datadir/student_wallet.json"
FLAG_FILE="./datadir/.initialized"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }
log_debug() { echo -e "${BLUE}[DEBUG]${NC} $1"; }

# Global miner address
MINER_ADDR=""

rpc() {
    docker exec "$CONTAINER" bitcoin-cli -regtest -rpcuser="$RPC_USER" -rpcpassword="$RPC_PASSWORD" "$@"
}

rpc_wallet() {
    docker exec "$CONTAINER" bitcoin-cli -regtest -rpcuser="$RPC_USER" -rpcpassword="$RPC_PASSWORD" -rpcwallet="$1" "${@:2}"
}

rpc_wallet_debug() {
    docker exec "$CONTAINER" bitcoin-cli -regtest -rpcuser="$RPC_USER" -rpcpassword="$RPC_PASSWORD" -rpcwallet="$1" "${@:2}" 2>&1
}

wallet_exists() {
    rpc listwallets 2>/dev/null | grep -q "\"$1\""
}

get_miner_address() {
    if [ -z "$MINER_ADDR" ]; then
        MINER_ADDR=$(rpc_wallet "$WALLET_MINER" getnewaddress "miner_main" "bech32" 2>/dev/null)
        log_info "Miner address: $MINER_ADDR"
    fi
    echo "$MINER_ADDR"
}

# Get miner balance as a number (no colors, no extra text)
get_miner_balance() {
    rpc_wallet "$WALLET_MINER" getbalance 2>/dev/null | tr -d '\n\r'
}

# Get miner UTXO count
get_miner_utxo_count() {
    rpc_wallet "$WALLET_MINER" listunspent 2>/dev/null | python3 -c "import sys, json; print(len(json.load(sys.stdin)))" 2>/dev/null || echo "0"
}

check_miner_balance() {
    local balance=$(get_miner_balance)
    local utxo_count=$(get_miner_utxo_count)
    log_debug "Miner balance: $balance BTC, UTXOs: $utxo_count"
    echo "$balance"
}

refill_miner() {
    log_info "Refilling miner wallet..."
    local miner_addr=$(get_miner_address)

    log_info "Generating 101 blocks to refill miner..."
    rpc_wallet "$WALLET_MINER" generatetoaddress 101 "$miner_addr" > /dev/null 2>&1

    local new_balance=$(get_miner_balance)
    log_info "Miner refilled. New balance: $new_balance BTC"
}

is_initialized() {
    if [ -f "$FLAG_FILE" ]; then
        return 0
    fi

    local utxo_count=$(rpc_wallet "$WALLET_STUDENT" listunspent 2>/dev/null | python3 -c "import sys, json; print(len(json.load(sys.stdin)))" 2>/dev/null || echo "0")
    if [ "$utxo_count" -gt 0 ]; then
        log_info "Student wallet already has $utxo_count UTXOs, marking as initialized"
        touch "$FLAG_FILE"
        return 0
    fi

    return 1
}

ensure_wallet() {
    if ! wallet_exists "$1"; then
        log_info "Creating wallet: $1"
        rpc createwallet "$1" > /dev/null 2>&1
        sleep 2
    else
        log_info "Wallet already exists: $1"
    fi
}

import_descriptors() {
    local wallet=$1
    local file=$2

    log_info "Importing descriptors from $file to wallet $wallet"

    python3 << EOF
import json
import subprocess
import sys
import time

with open('$file', 'r') as f:
    descriptors = json.load(f)

for i, desc in enumerate(descriptors):
    import_json = json.dumps([{
        "desc": desc["desc"],
        "timestamp": desc["timestamp"],
        "active": desc.get("active", True),
        "internal": desc.get("internal", False)
    }])

    cmd = ["docker", "exec", "$CONTAINER", "bitcoin-cli", "-regtest",
           "-rpcuser=$RPC_USER", "-rpcpassword=$RPC_PASSWORD",
           "-rpcwallet=$wallet", "importdescriptors", import_json]

    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0:
        print(f"Warning: Failed to import descriptor {i}: {result.stderr}", file=sys.stderr)
    time.sleep(0.5)
EOF

    log_info "Descriptor import completed"
}

generate_addresses_from_descriptors() {
    local wallet=$1
    local file=$2

    log_info "Generating addresses from descriptors..."

    python3 << EOF
import json
import subprocess
import time

def rpc_cmd(wallet, method, *args):
    cmd = ["docker", "exec", "$CONTAINER", "bitcoin-cli", "-regtest",
           "-rpcuser=$RPC_USER", "-rpcpassword=$RPC_PASSWORD",
           "-rpcwallet=" + wallet, method] + list(args)
    result = subprocess.run(cmd, capture_output=True, text=True)
    return result.stdout.strip()

with open('$file', 'r') as f:
    descriptors = json.load(f)

for desc in descriptors:
    if desc.get("internal"):
        continue

    desc_str = desc["desc"]
    range_max = desc["range"][1] if desc.get("range") else 0

    if "pkh(" in desc_str:
        addr_type = "legacy"
    elif "sh(wpkh(" in desc_str:
        addr_type = "p2sh-segwit"
    elif "tr(" in desc_str:
        addr_type = "bech32m"
    elif "wpkh(" in desc_str:
        addr_type = "bech32"
    else:
        continue

    print(f"Generating {range_max + 1} addresses of type {addr_type}")

    for i in range(range_max + 1):
        label = f"{addr_type}_{i}"
        rpc_cmd("$wallet", "getnewaddress", label, addr_type)
        if i % 50 == 0 and i > 0:
            time.sleep(0.1)

    time.sleep(1)

print("Address generation complete")
EOF

    log_info "Address generation completed"
}

get_student_addresses() {
    rpc_wallet "$WALLET_STUDENT" listreceivedbyaddress 0 true 2>/dev/null | \
        python3 -c "import sys, json; data = json.load(sys.stdin); [print(a['address']) for a in data]"
}

get_student_address_count() {
    get_student_addresses | wc -l
}

fund_addresses() {
    local total_amount=$1
    local addresses=()

    log_info "Collecting student addresses..."

    while IFS= read -r addr; do
        [ -n "$addr" ] && addresses+=("$addr")
    done < <(get_student_addresses)

    local total=${#addresses[@]}
    log_info "Found $total addresses"

    if [ $total -eq 0 ]; then
        log_error "No addresses to fund!"
        return 1
    fi

    # Use Python for precise decimal calculation
    local per_address=$(python3 -c "print(f'{(float($total_amount) / float($total)):.8f}')")
    local min_dust=0.00001

    # Ensure minimum amount to avoid dust
    if (( $(echo "$per_address < $min_dust" | python3 -c "import sys; print(float(sys.stdin.read()))" 2>/dev/null || echo "0") )); then
        per_address=$min_dust
    fi

    log_info "Sending $per_address BTC to each of $total addresses"

    local total_to_send=$(python3 -c "print(f'{(float($per_address) * float($total)):.8f}')")
    log_info "Total to send: $total_to_send BTC"

    local success_count=0
    local fail_count=0
    local batch=1
    local batch_size=50

    for i in "${!addresses[@]}"; do
        local addr="${addresses[$i]}"
        local current=$((i + 1))

        # Check balance every 100 sends (simplified)
        if [ $((current % 100)) -eq 0 ]; then
            local miner_balance=$(get_miner_balance)
            log_debug "Current miner balance: $miner_balance BTC"
        fi

        log_info "Sending to address $current/$total: ${addr:0:25}..."

        local send_output=$(rpc_wallet_debug "$WALLET_MINER" sendtoaddress "$addr" $per_address 2>&1)
        local send_exit_code=$?

        if [ $send_exit_code -eq 0 ]; then
            success_count=$((success_count + 1))
            log_debug "✓ Success"
        else
            fail_count=$((fail_count + 1))
            log_warn "✗ Failed: $send_output"

            # If error is about insufficient funds, refill immediately
            if echo "$send_output" | grep -q "Insufficient funds"; then
                log_warn "Insufficient funds! Refilling miner..."
                refill_miner
                # Retry this address
                log_info "Retrying address $current/$total..."
                if rpc_wallet "$WALLET_MINER" sendtoaddress "$addr" $per_address > /dev/null 2>&1; then
                    success_count=$((success_count + 1))
                    fail_count=$((fail_count - 1))
                    log_debug "✓ Success on retry"
                fi
            fi
        fi

        # Generate block every batch_size transactions
        if [ $((current % batch_size)) -eq 0 ]; then
            log_info "Generating block for batch $batch..."
            local miner_addr=$(get_miner_address)
            rpc_wallet "$WALLET_MINER" generatetoaddress 1 "$miner_addr" > /dev/null 2>&1
            batch=$((batch + 1))
            sleep 1
        fi

        sleep 0.05
    done

    log_info "Generating final blocks..."
    local miner_addr=$(get_miner_address)
    rpc_wallet "$WALLET_MINER" generatetoaddress 1 "$miner_addr" > /dev/null 2>&1

    log_info "Funding completed: $success_count successful, $fail_count failed"

    if [ $success_count -gt 0 ] && [ $fail_count -eq 0 ]; then
        touch "$FLAG_FILE"
        log_info "Created initialization flag file"
    elif [ $success_count -gt 0 ]; then
        log_warn "Partial success: $success_count/$total addresses funded"
    fi
}

generate_blocks() {
    local count=$1
    log_info "Generating $count blocks..."
    local miner_addr=$(get_miner_address)
    rpc_wallet "$WALLET_MINER" generatetoaddress $count "$miner_addr" > /dev/null 2>&1
    log_info "Generated $count blocks"
}

check_status() {
    log_info "=== Final Status ==="

    local balance=$(rpc_wallet "$WALLET_STUDENT" getbalance 2>/dev/null || echo "0")
    local utxos=$(rpc_wallet "$WALLET_STUDENT" listunspent 2>/dev/null | python3 -c "import sys, json; print(len(json.load(sys.stdin)))" 2>/dev/null || echo "0")
    log_info "Student wallet: $utxos UTXOs, balance: $balance BTC"

    local miner_balance=$(get_miner_balance)
    log_info "Miner wallet balance: $miner_balance BTC"
}

main() {
    log_info "=== Starting wallet initialization ==="

    if is_initialized; then
        log_info "System already initialized, skipping funding. Remove $FLAG_FILE to force re-initialization."
        check_status
        log_info "=== Initialization already complete, exiting ==="
        exit 0
    fi

    log_info "First time setup - starting full initialization..."
``
    log_info "Waiting for bitcoind to be ready..."
    sleep 5

    ensure_wallet "$WALLET_MINER"
    ensure_wallet "$WALLET_STUDENT"

    get_miner_address

    if [ -f "$DESCRIPTORS_FILE" ]; then
        import_descriptors "$WALLET_STUDENT" "$DESCRIPTORS_FILE"
    else
        log_error "Descriptors file not found: $DESCRIPTORS_FILE"
        exit 1
    fi

    generate_blocks 101
    generate_addresses_from_descriptors "$WALLET_STUDENT" "$DESCRIPTORS_FILE"

    local addr_count=$(get_student_address_count)
    log_info "Total student addresses: $addr_count"

    fund_addresses 10

    check_status

    log_info "=== Initialization complete ==="
}

main