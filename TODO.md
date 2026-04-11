# Bitcoin Indexer Storage & Cache Architecture

## 🚀 Part 1: High-Performance UTXO Cache in Rust (In-Memory)

* Implement Sharded Flat-Hashmap
* Use the hashbrown or fxhash crate instead of std::collections::HashMap to minimize memory overhead.
    * Implement an active lock sharding (multiple RwLock segments) to prevent thread contention during massive parallel block processing.
* Define Compact Cache Types
* Key: ([u8; 32], u32) $\rightarrow$ (Txid, Vout) (Total: 36 bytes).
    * Value: UtxoValue { amount: u64, script_type: u8 } (Total: 9 bytes).
* Hook Cache into Block Processing Pipeline
* For Outputs: Automatically add every newly generated transaction output directly into the memory cache.
    * For Inputs: Attempt to pop the corresponding output from the cache using cache.spend().
    * Fallback: If the target output is missing from the cache, batch the IDs and fetch them directly from Postgres in a single optimized query.

## 🗄️ Part 2: Lightweight Persistent Tables (The Archive)

* Build Raw Transaction Archive Table
* Create a transaction_archive table with fields: txid BYTEA PRIMARY KEY, block_height BIGINT, and raw_tx BYTEA.
    * Apply native Postgres HASH Partitioning on txid (modulus 32 or 64) to prevent reaching database size bottlenecks and index depth issues.
* Eliminate Complex DB Joins
* Do not store transaction inputs and outputs as separate relational tables.
    * Treat the raw hex/binary transaction array (raw_tx) as the ultimate source of truth.
* Adopt On-Demand Parsing for API Details
* When a user queries GET /tx/<id>, fetch the single raw_tx array.
    * Use the Rust bitcoin crate to decode and parse the transaction details directly in the CPU memory on the fly.
* Deploy Light Address-Mapping Index
* Create a thin mapping table: Address_ID <-> TxID <-> Height purely to identify which raw transactions need to be fetched for any given wallet.

# CI/CD Section

## 🔄 Enabling Compile-Time Macros in GitHub Actions

* Generate SQLx Offline Cache Locally [1]
* Once the local database is running and all migrations are applied, run the following command to parse your project's SQL queries and save their expected types [1]:

  cargo sqlx prepare

  * Note: If you are operating in a Cargo workspace, use cargo sqlx prepare --workspace instead [1].
* Commit the Query Metadata to Git
* Stage and commit the newly generated .sqlx folder or sqlx-data.json file [1]:

  git add .sqlx/
  git commit -m "chore: update sqlx query metadata for offline mode"

  * This allows the compiler on GitHub to analyze query safety without needing a live connection to a Postgres database! [1, 2]
* Update GitHub Actions Workflow File
* Open your .github/workflows/ci.yml file and inject the SQLX_OFFLINE=true environment variable into your testing or building steps [1, 2]:

  - name: Run Tests
    run: cargo test --verbose
    env:
    SQLX_OFFLINE: true

  * Refactor Queries Back to Macros
* Once the CI is prepared, safely convert non-macro functions like sqlx::query_as::<_, BlockInfo> back into strictly typed macros like sqlx::query_as!(BlockInfo, ...) to reclaim zero-cost compile-time database checks [1].



