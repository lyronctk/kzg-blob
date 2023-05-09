# KZG Blob Reads
Prove batch multi-opens of KZG blobs described in EIP-4844.

## Motivation
Data availability is a critical aspect to Ethereum’s rollup-centric roadmap. Current production systems for storing rollup transactions make compromises in cost (when stored in calldata) or in trust (when stored off-chain).

Danksharding, along with the intermediate upgrade of proto-danksharding, presents a promising solution. It is an approach for storing aribitrary blobs of data cheaply on the main Ethereum network. How? Rather than storing blobs directly in the calldata at the execution layer, proto-danksharding has raw data stored temporarily at the consensus layer, with only the commitments stored at the execution layer.

These blob commitments are KZG vector commitments. To make use of them in the EVM, contracts must check the validity of batches of KZG openings. Given the lack of a BLS precompile on top of an already compute intensive procedure, this is only feasible on-chain when done within a SNARK.

## Usage
```
cargo run --example driver
```
