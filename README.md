# ZK Gacha Game Demo

[![CI Checks](https://github.com/YOUR_USERNAME/zk-gacha-game/actions/workflows/ci.yml/badge.svg)](https://github.com/YOUR_USERNAME/zk-gacha-game/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**A demonstration of a verifiable gacha (loot box) game using Zero-Knowledge Proofs (ZK-SNARKs).**

This project showcases how ZK-SNARKs (specifically using the [Arkworks](https://arkworks.rs/) Rust libraries) can be used to build a web-based gacha game where users can cryptographically verify the fairness and validity of their pulls without relying on a trusted backend server for the verification step itself.

The demo utilizes a simplified backend-less architecture for demonstration purposes, relying on **Rust compiled to WebAssembly (WASM)** for client-side proof generation/verification, **Amazon S3** (or similar) for storing public data (Merkle root, CRS, Poseidon Params) and **individual item data files** (containing secret key and Merkle path), and a **ZK-SNARK circuit** defining the validity rules.

Built upon the `Nextjs-Rust-WASM-monorepo` template.

## Features

*   🎲 **Verifiable Gacha Pulls:** Users receive cryptographic proof that their pulled item's corresponding public key `HASH(secret_key, item_id)` is a valid member of the pre-committed Merkle Tree.
*   🔒 **Client-Side Verification:** ZK proof generation and verification happens directly in the user's browser using WASM (Groth16).
*   🌳 **Merkle Tree Commitment:** The public keys of the entire gacha item pool are committed into a Merkle Tree (using Poseidon hashing). The **Merkle Root** is published transparently.
*   🦀 **Rust & WASM Core:** ZK circuits and cryptographic logic (Poseidon Hash, Merkle Tree) implemented in Rust (Arkworks `ark-crypto-primitives`, `ark-bls12-381`, `ark-groth16`) and compiled to WASM.
*   ✨ **Engaging UI (Planned):** Includes visually appealing animations for the gacha pulling sequence.
*   📦 **Local Inventory:** Pulled items are stored and displayed using browser local storage.
*   ⚡ **Modern Web Frontend:** Built with Next.js (App Router), TypeScript, Tailwind CSS, and Shadcn UI.
*   🚀 **Monorepo:** Managed with Turborepo and pnpm.
*   ✨ **Code Quality:** Linting and formatting enforced by Biome.
*   🧪 **Testing:** Comprehensive testing setup (Vitest, Playwright, Rust tests, wasm-bindgen-test).

## How it Works (Simplified Demo Flow & Cryptography)

1.  **Admin Prep (Offline Script - typically run locally, e.g., `scripts/generate_crs.rs`, `scripts/prepare_gacha_data.rs`):**
    *   Define item pool (ID, name, image, etc.).
    *   **(CRS Gen)** Generate ZK-SNARK **CRS (Proving Key - PK, Verifying Key - VK)** for the `UserPullCircuit` using `ark-groth16::circuit_specific_setup`. Save `gacha_pk.bin`, `gacha_vk.bin`.
    *   **(Params Gen)** Generate **Poseidon hash parameters** using `ark_crypto_primitives::crh::poseidon::PoseidonCRH::setup`. Save `params.bin`.
    *   **(Data Prep)** For each potential gacha item instance:
        *   Generate a unique `secret_key` (random `BlsFr`).
        *   Calculate the `item_id` (`BlsFr` representation).
        *   Load Poseidon parameters (`params.bin`).
        *   Calculate the **public key (leaf hash):** `leaf_hash = PoseidonCRH::evaluate(&params, &[secret_key, item_id])`.
    *   **(Data Prep)** Build a **Merkle Tree** (`ark_crypto_primitives::merkle_tree::MerkleTree`) using all calculated `leaf_hash` values. The tree config should use `PoseidonCRH` (with the *same* loaded parameters) for both leaf and inner node hashing, and `IdentityDigestConverter<BlsFr>`.
    *   **(Data Prep)** Compute and save the **Merkle Root** (`merkle_root.hex`).
    *   **(Data Prep)** For each leaf (`leaf_hash`) at `leaf_index`:
        *   Generate its **Merkle proof (`Path` struct)**: `path = tree.generate_proof(leaf_index)`.
        *   Derive `path_indices: Vec<bool>` using `path.position_list().collect()`.
        *   Create a JSON file named uniquely (e.g., `item_{leaf_index}.json`) containing:
            ```json
            {
              "itemIdHex": "0x...", // item_id as hex
              "secretKeyHex": "0x...", // secret_key as hex
              "merklePathHex": ["0x...", ...], // path.auth_path as hex strings
              "pathIndices": [false, true, ...] // derived path_indices
            }
            ```
2.  **Deployment:**
    *   Deploy `gacha_pk.bin`, `gacha_vk.bin`, `params.bin`, `merkle_root.hex` to a public location (e.g., S3 bucket, or `/public` in web app).
    *   Deploy all generated item JSON files (`item_*.json`) to the same S3 bucket (or another known location).
    *   Deploy item master data (ID -> name, image) if needed.
    *   Create a file listing all deployed item JSON file URLs (e.g., `key_list.txt`) and deploy it.
3.  **User Experience (Web App):**
    *   App fetches PK, VK, Params, Merkle Root, Item Master Data, and the `key_list.txt`.
    *   WASM module is loaded and initialized: `init_gacha_keys(pkBytes, vkBytes, paramsBytes)`.
    *   User clicks "Pull Gacha".
    *   **(Demo Simplification)** App randomly selects a URL from `key_list.txt`.
    *   App fetches the selected item JSON file.
    *   Gacha animation plays, revealing the item (`itemIdHex` mapped to Item Master Data).
    *   User clicks "Verify My Pull".
    *   App prepares `WasmGachaCircuitInputs` (JS object) using fetched JSON data and the loaded Merkle Root.
    *   App serializes the inputs using `serde_wasm_bindgen::to_bytes`.
    *   Browser calls WASM `generate_gacha_proof(serializedBytes)`. The circuit verifies the cryptographic relations.
    *   User clicks "Verify Proof".
    *   Browser calls WASM `verify_gacha_proof(merkleRootHex, proofBytes)`.
    *   Verification result (Success/Fail) is displayed.

## Tech Stack

*   **Framework:** Next.js 14+ (App Router)
*   **Language:** TypeScript, Rust (Stable)
*   **ZK Library:** Arkworks (`ark-bls12-381`, `ark-groth16`, `ark-crypto-primitives`)
*   **WASM:** `wasm-bindgen`, `wasm-pack`
*   **Monorepo:** Turborepo
*   **Package Manager:** pnpm (v10+)
*   **Styling:** Tailwind CSS
*   **UI Components:** Shadcn UI, Lucide React, Framer Motion (for animations)
*   **Linting/Formatting:** Biome
*   **Testing:** Vitest, React Testing Library, Playwright, `cargo test`, `wasm-bindgen-test`
*   **Deployment (Data):** AWS S3 (or compatible)
*   **CI:** GitHub Actions

## Project Structure

```
/
├── .github/          # GitHub Actions workflows (CI)
├── apps/
│   └── web/          # Next.js application (Gacha UI)
├── packages/
│   ├── zk-circuits/  # Rust WASM library (ZK Circuits, Arkworks logic) <--- Renamed
│   ├── tsconfig/     # Shared TypeScript configs
│   └── config/       # Shared configurations (e.g., Tailwind preset)
├── scripts/          # Offline Rust scripts (CRS gen, Admin data prep) <--- New (Run Separately)
├── biome.json        # Biome (Lint/Format) config
├── package.json      # Root dependencies & scripts
├── pnpm-workspace.yaml # pnpm workspace config
├── turbo.json        # Turborepo pipeline config
└── tsconfig.json     # Root TS config
```

## Getting Started (Development)

1.  **Initial Setup:** Follow Step 1 above (clone, rename, install deps).
2.  **Run Offline Scripts (One-time or when circuits/params change):**
    *   `cd scripts && cargo run --bin generate_crs --release` (adjust path/command) - Generates `*.bin` files.
    *   `cd scripts && cargo run --bin prepare_gacha_data --release` (adjust path/command) - Generates `merkle_root.hex` and item JSON files.
    *   **Note:** You'll need to configure these scripts (e.g., item definitions, output paths). Place generated files (`*.bin`, `merkle_root.hex`) into `apps/web/public/` for local development, and upload item JSON files somewhere accessible (or also put in `/public` for simple testing). Create a `key_list.txt` in `/public` listing the local paths to the JSON files.
3.  **Run Development Server:**
    ```bash
    pnpm run dev
    ```
4.  **Access App:** Open `http://localhost:3000` (or your configured port) in your browser. Navigate to the gacha page.

## Disclaimer & Security Limitations (IMPORTANT)

This project is a **demonstration** of ZK-SNARKs for client-side verification. **It is NOT a secure, production-ready gacha system.** Key limitations include:

*   **🚫 Insecure Key Distribution:** The client fetches the secret key and Merkle path directly from public storage (S3 or `/public`). **In a real system, this is a critical vulnerability**, as anyone could download all files and steal all secret keys. A secure backend is required to manage secrets and distribute only the *user's specific pulled item data* directly to that user after a pull event.
*   **🚫 No Probability Enforcement:** The client randomly selecting a file URL does not guarantee the admin's advertised probabilities are respected. A trusted backend or a more complex on-chain mechanism is needed for fair probability distribution and to prevent users from selecting specific desired outcomes.
*   **State:** Inventory relies on browser local storage, which is not persistent or shared.

This demo focuses solely on the **cryptographic verification aspect** assuming the user somehow legitimately received the secret key and path for their item.

## License

This project is licensed under the MIT License.
