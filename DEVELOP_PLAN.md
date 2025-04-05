**ZK Gacha Game - 포괄적 개발 가이드**

이 문서는 `Nextjs-Rust-WASM-monorepo` 템플릿을 기반으로 Arkworks 라이브러리를 사용하여 ZK-SNARK 기반의 검증 가능한 웹 가챠 게임 데모를 구축하는 전체 과정을 안내합니다.

**단계 요약:**

1.  **템플릿 기반 새 저장소 생성 및 초기 설정:** GitHub 템플릿 사용, 프로젝트 이름 변경, 기본 설정 완료.
2.  **새 프로젝트 README 작성:** ZK 가챠 데모의 목적, 기능, 기술 스택, **작동 방식(암호화 흐름 포함)**, **중요 보안 한계점** 설명.
3.  **Rust 코드 로직 구현 (`packages/zk-circuits`):** ZK 회로, 증명/검증 로직, WASM 인터페이스 상세 구현 계획 (Arkworks 가젯 사용, 파라미터 관리, 타입 정의, 에러 처리 포함).
4.  **웹 애플리케이션 개발 (`apps/web`):** 가챠 게임 UI, 상태 관리, WASM 연동, 데이터 처리, 초기화 로직 등 프론트엔드 구현 상세 가이드.
5.  **오프라인 스크립트 개발 (`scripts/`):** CRS 생성 및 관리자 데이터 준비 스크립트 구현 계획.

---

**1단계: 템플릿 기반 새 저장소 생성 및 초기 설정**

1.  **새 저장소 생성:**
    *   `https://github.com/glycogen94/Nextjs-Rust-WASM-monorepo` 로 이동합니다.
    *   "Use this template" > "Create a new repository" 클릭.
    *   저장소 이름 지정: `zk-gacha-game`.
    *   저장소를 로컬에 클론: `git clone git@github.com:YOUR_USERNAME/zk-gacha-game.git`
    *   `cd zk-gacha-game`

2.  **프로젝트 이름 변경 및 설정 (매우 중요):**
    *   `GETTING_STARTED.md` 파일의 "Step 2: Rename Placeholders" 섹션을 **정확히** 따릅니다.
    *   **결정된 이름:**
        *   Project Scope/Name: `zk-gacha-game`
        *   Web App Package Name: `web`
        *   WASM Library Package Name: `zk-circuits`
        *   WASM Crate Name: `zk_circuits`
    *   **주요 수정 대상 파일:**
        *   `/package.json` (`name`)
        *   `packages/wasm-lib` 디렉토리 -> `packages/zk-circuits`로 이름 변경 (`git mv` 사용 권장).
        *   `packages/zk-circuits/package.json` (`name: "zk-circuits"`, `main: "./pkg/zk_circuits.js"`, `types: "./pkg/zk_circuits.d.ts"`)
        *   `packages/zk-circuits/Cargo.toml` (`[package].name = "zk_circuits"`)
        *   `apps/web/lib/wasmLoader.ts` (`import ... from 'zk-circuits';`)
        *   `apps/web/next.config.mjs` (`transpilePackages: ["zk-circuits"]`)
        *   기타 문서 내 참조 이름 수정.

3.  **의존성 재설치:** 루트 디렉토리에서 `pnpm install` 실행.

4.  **기본 검증:** `pnpm run format`, `pnpm run lint`, `pnpm run build` 등을 실행하여 기본 설정이 올바른지 확인합니다. (`pnpm run test:wasm`은 추후 수정 필요).

---

**2단계: 새 프로젝트 README 작성 (`/README.md`)**

```markdown
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
```

---

**3단계: Rust 코드 로직 구현 (`packages/zk-circuits`)**

**A. `Cargo.toml`:**

```toml
# packages/zk-circuits/Cargo.toml

[package]
name = "zk_circuits" # WASM Crate Name
version = "0.1.0"
edition = "2021"
authors = ["Your Name <you@example.com>"]
description = "ZK circuits and WASM bindings for the Verifiable Gacha Game Demo"
license = "MIT"
repository = "https://github.com/YOUR_USERNAME/zk-gacha-game"

[lib]
crate-type = ["cdylib", "rlib"] # cdylib for WASM, rlib for integration tests/scripts

[dependencies]
# WASM Interaction
wasm-bindgen = "0.2.89"
serde = { version = "1.0", features = ["derive"] }
serde_bytes = "0.11"
serde_wasm_bindgen = "0.6"
console_error_panic_hook = { version = "0.1.7", optional = true }
getrandom = { version = "0.2", features = ["js"] } # WASM random source

# Arkworks Core & Algebra
ark-ff = { version = "^0.4.0", default-features = false }
ark-ec = { version = "^0.4.0", default-features = false }
ark-bls12-381 = { version = "^0.4.0", default-features = false, features = ["curve"] }
ark-std = { version = "^0.4.0", default-features = false, features = ["print-trace"] } # Add "parallel" if needed

# Arkworks ZK-SNARKs & Constraints
ark-relations = { version = "^0.4.0", default-features = false }
ark-r1cs-std = { version = "^0.4.0", default-features = false } # Standard Gadgets (Boolean, FpVar, etc.)
ark-snark = "^0.4.0"
ark-groth16 = { version = "^0.4.0", default-features = false, features = ["r1cs"] }

# Arkworks Crypto Primitives (Assuming this local/git dep contains Poseidon, Merkle Tree)
# Adjust path or git details as needed
ark-crypto-primitives = { version = "0.4", default-features = false, features = ["crh", "merkle_tree", "sponge", "r1cs"] }

# Utilities
hex = "0.4"
once_cell = "1.19"      # For global PK/VK/Params storage
parking_lot = "0.12"    # Mutex for thread-safe access to globals
rand = "0.8"            # For proof generation randomness
thiserror = "1.0"       # For error handling
bincode = "1.3"         # Alternative serialization (optional)
ark-serialize = { version = "^0.4.0", features = ["derive"] } # Arkworks serialization

[dev-dependencies]
wasm-bindgen-test = "0.3.39" # For WASM testing

[features]
default = ["console_error_panic_hook"]
# parallel = ["ark-std/parallel", "ark-crypto-primitives/parallel"] # Enable parallel features if needed

[profile.release]
opt-level = 3 # Optimize for speed and size
lto = true      # Link-Time Optimization
codegen-units = 1 # Better optimization potential
panic = 'abort'   # Smaller code size on panic
```

**B. 에러 타입 정의 (`src/error.rs`):**

```rust
// packages/zk-circuits/src/error.rs
use thiserror::Error;
use wasm_bindgen::JsValue;
use ark_serialize::SerializationError;

#[derive(Error, Debug)]
pub enum GachaCircuitError {
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Arkworks Serialization error: {0}")]
    ArkSerialization(#[from] SerializationError),
    #[error("Deserialization error: {0}")]
    Deserialization(String),
    #[error("Arkworks synthesis error: {0}")]
    Synthesis(#[from] ark_relations::r1cs::SynthesisError),
    #[error("Proof generation failed: {0}")]
    ProofGeneration(String),
    #[error("Proof verification failed: {0}")]
    ProofVerification(String),
    #[error("Keys or Parameters not initialized")]
    NotInitialized,
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Data conversion error: {0}")]
    ConversionError(String),
    #[error("Hex decoding error: {0}")]
    HexError(#[from] hex::FromHexError),
    #[error("Setup error: {0}")]
    SetupError(String),
    #[error("Underlying crypto primitive error: {0}")]
    PrimitiveError(String), // Wrap errors from ark-crypto-primitives
}

// Convert GachaCircuitError to JsValue for WASM boundary
impl From<GachaCircuitError> for JsValue {
    fn from(error: GachaCircuitError) -> Self {
        JsValue::from_str(&error.to_string())
    }
}

// Allow '?' operator to convert ark_crypto_primitives::Error
impl From<ark_crypto_primitives::Error> for GachaCircuitError {
    fn from(err: ark_crypto_primitives::Error) -> Self {
        GachaCircuitError::PrimitiveError(err.to_string())
    }
}
```

**C. 타입 정의 및 변환 유틸리티 (`src/types.rs`):**

```rust
// packages/zk-circuits/src/types.rs
use ark_bls12_381::Fr as BlsFr;
use ark_ff::{PrimeField, BigInteger}; // Added BigInteger for byte conversion
use ark_std::vec::Vec;
use serde::{Deserialize, Serialize};
use crate::error::GachaCircuitError;
use ark_serialize::{CanonicalSerialize, CanonicalDeserialize};

// --- Import concrete types from ark-crypto-primitives (Verify these paths!) ---
use ark_crypto_primitives::{
    crh::{
        poseidon::{PoseidonConfig, PoseidonCRH},
        CRHScheme, TwoToOneCRHScheme,
    },
    merkle_tree::{Config as MerkleConfig, Path as MerklePath, IdentityDigestConverter},
};

// --- Native Merkle Tree Configuration ---
#[derive(Debug, Clone)]
pub struct GachaMerkleConfig;

impl MerkleConfig for GachaMerkleConfig {
    // Leaf data is conceptually (secret_key, item_id), but hash takes &[BlsFr]
    type Leaf = [BlsFr];
    // Both leaf and inner digests are Poseidon outputs (field elements)
    type LeafDigest = BlsFr;
    type InnerDigest = BlsFr;

    // Use Poseidon for hashing leaves (assuming it takes a slice)
    // Replace 'PoseidonCRH<BlsFr>' if the actual type is different
    type LeafHash = PoseidonCRH<BlsFr>;
    // Use Poseidon for hashing inner nodes (two children digests)
    type TwoToOneHash = PoseidonCRH<BlsFr>;

    // Since digests are BlsFr and hash inputs are BlsFr, no conversion needed.
    type LeafInnerDigestConverter = IdentityDigestConverter<BlsFr>;
}

// --- Core Types ---
pub type PoseidonParam = PoseidonConfig<BlsFr>; // Poseidon parameters type
pub type GachaHashOutput = BlsFr;              // Hash output type
pub type GachaMerklePathNodes = Vec<GachaHashOutput>; // Sibling nodes in the path
pub type GachaPathIndices = Vec<bool>;         // Left/right indicators for the path

// --- WASM Data Transfer Object (matches expected JSON from S3) ---
#[derive(Serialize, Deserialize, Debug)]
pub struct WasmGachaCircuitInputs {
    #[serde(rename = "merkleRoot")] // Ensure JSON field names match
    pub merkle_root: String, // Public input (Hex)
    #[serde(rename = "itemIdHex")]
    pub item_id: String,       // Private witness (Hex)
    #[serde(rename = "secretKeyHex")]
    pub secret_key: String,    // Private witness (Hex)
    #[serde(rename = "merklePathHex")]
    pub merkle_path: Vec<String>, // Private witness (Vec<Hex>)
    #[serde(rename = "pathIndices")]
    pub path_indices: Vec<bool>,  // Private witness (Vec<bool>)
}

// --- Native Rust Input Structure (for circuit construction) ---
#[derive(Clone, Debug)]
pub struct NativeGachaCircuitInputs {
    pub merkle_root: GachaHashOutput,       // Public
    pub item_id: BlsFr,                     // Witness
    pub secret_key: BlsFr,                  // Witness
    pub merkle_path_nodes: GachaMerklePathNodes, // Witness (sibling hashes)
    pub path_indices: GachaPathIndices,       // Witness
}

// --- Conversion Functions ---

/// Converts a hex string (with or without "0x") to BlsFr.
pub fn fr_from_hex(hex_str: &str) -> Result<BlsFr, GachaCircuitError> {
    let stripped = hex_str.trim_start_matches("0x");
    let bytes = hex::decode(stripped)?;
    // Ensure bytes are interpreted correctly (Little Endian for Arkworks)
    BlsFr::from_le_bytes_mod_order(&bytes)
    // Add length validation if necessary:
    // if bytes.len() > BlsFr::size_in_bytes() {
    //     return Err(GachaCircuitError::ConversionError("Hex string too long for field element".to_string()));
    // }
}

/// Converts a BlsFr to a hex string ("0x..." prefix).
pub fn fr_to_hex(fr: &BlsFr) -> Result<String, GachaCircuitError> {
    let mut bytes = Vec::new();
    // Use compressed serialization if possible and desired
    fr.serialize_compressed(&mut bytes)?;
    Ok(format!("0x{}", hex::encode(&bytes)))
}

/// Tries to convert WASM DTO to Native Rust structure.
impl TryFrom<WasmGachaCircuitInputs> for NativeGachaCircuitInputs {
    type Error = GachaCircuitError;

    fn try_from(wasm_inputs: WasmGachaCircuitInputs) -> Result<Self, Self::Error> {
        let merkle_root = fr_from_hex(&wasm_inputs.merkle_root)?;
        let item_id = fr_from_hex(&wasm_inputs.item_id)?;
        let secret_key = fr_from_hex(&wasm_inputs.secret_key)?;

        let merkle_path_nodes = wasm_inputs
            .merkle_path
            .iter()
            .map(|hex_node| fr_from_hex(hex_node))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| GachaCircuitError::ConversionError(format!("Failed to parse merkle path node: {}", e)))?;

        // Basic validation: path nodes count must match indices count
        if merkle_path_nodes.len() != wasm_inputs.path_indices.len() {
             return Err(GachaCircuitError::InvalidInput(
                 format!("Merkle path nodes count ({}) does not match indices count ({})",
                 merkle_path_nodes.len(), wasm_inputs.path_indices.len())
             ));
        }

        Ok(NativeGachaCircuitInputs {
            merkle_root,
            item_id,
            secret_key,
            merkle_path_nodes,
            path_indices: wasm_inputs.path_indices,
        })
    }
}

/// Prepares the public inputs vector for Groth16 verification.
/// The order MUST match the `new_input()` calls in the circuit.
pub fn prepare_groth16_public_inputs(merkle_root: GachaHashOutput) -> Vec<BlsFr> {
    vec![merkle_root]
}
```

**D. 회로 설계 (`src/circuit.rs`):**

```rust
// packages/zk-circuits/src/circuit.rs
use ark_ff::PrimeField;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_std::{marker::PhantomData, vec::Vec};
use ark_r1cs_std::{
    prelude::*, // AllocVar, EqGadget, Boolean, FieldVar etc.
    fields::fp::FpVar,
};

// --- Import necessary types and gadgets (Verify paths!) ---
use ark_crypto_primitives::{
    crh::{
        poseidon::constraints::{PoseidonGadget, PoseidonParametersVar},
        CRHGadget, // If using generic CRH methods
    },
    merkle_tree::{
        constraints::{PathVar, PathMerkleTreeVerificationGadget},
        Config as MerkleConfig, // Native Config trait
    },
};
use crate::types::{
    GachaMerkleConfig, // Our defined native Config
    PoseidonParam,     // Native Poseidon parameters
    GachaHashOutput, GachaMerklePathNodes, GachaPathIndices, NativeGachaCircuitInputs
};

/// ZK-SNARK circuit for verifying a gacha pull.
#[derive(Clone)]
pub struct UserPullCircuit<F: PrimeField> {
    // --- Public Inputs ---
    pub merkle_root: GachaHashOutput,

    // --- Private Inputs (Witness) ---
    pub item_id: F,
    pub secret_key: F,
    pub merkle_path_nodes: GachaMerklePathNodes, // Sibling node hashes
    pub path_indices: GachaPathIndices,       // Left/right booleans

    // --- Parameters (Passed during construction) ---
    pub poseidon_params: PoseidonParam,

    _phantom_f: PhantomData<F>,
}

impl<F: PrimeField> UserPullCircuit<F> {
    /// Creates a new circuit instance.
    pub fn new(inputs: NativeGachaCircuitInputs, poseidon_params: PoseidonParam) -> Self {
        Self {
            merkle_root: inputs.merkle_root,
            item_id: inputs.item_id,
            secret_key: inputs.secret_key,
            merkle_path_nodes: inputs.merkle_path_nodes,
            path_indices: inputs.path_indices,
            poseidon_params,
            _phantom_f: PhantomData,
        }
    }
}

impl<F: PrimeField> ConstraintSynthesizer<F> for UserPullCircuit<F> {
    /// Generates the R1CS constraints for the circuit.
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        // 1. Allocate Public Input Variables
        // This value is known by both prover and verifier.
        let merkle_root_var = FpVar::<F>::new_input(cs.clone(), || Ok(self.merkle_root))?;

        // 2. Allocate Private Witness Variables
        // These values are known only by the prover.
        let item_id_var = FpVar::<F>::new_witness(cs.clone(), || Ok(self.item_id))?;
        let secret_key_var = FpVar::<F>::new_witness(cs.clone(), || Ok(self.secret_key))?;
        // Allocate sibling node hashes as witnesses
        let merkle_path_vars = Vec::<FpVar<F>>::new_witness(cs.clone(), || Ok(self.merkle_path_nodes))?;
        // Allocate path indices (booleans) as witnesses
        let path_indices_vars = Vec::<Boolean<F>>::new_witness(cs.clone(), || Ok(self.path_indices))?;

        // 3. Allocate Parameters as Constants
        // Poseidon parameters are fixed for the circuit.
        let params_var = PoseidonParametersVar::<F>::new_constant(cs.clone(), self.poseidon_params)?;

        // --- Define Constraints ---

        // Constraint 1: Calculate the leaf hash from private inputs
        // leaf_hash = Poseidon(secret_key, item_id)
        let leaf_inputs = &[secret_key_var, item_id_var];
        // Use the PoseidonGadget's hashing method (verify API).
        // Assuming `hash` takes params and a slice of inputs.
        let computed_leaf_hash_var = PoseidonGadget::<F>::hash(cs.clone(), &params_var, leaf_inputs)?;

        // Constraint 2: Verify the Merkle path
        // We need to prove that `computed_leaf_hash_var` exists in the tree defined by `merkle_root_var`.

        // 2a. Create the Path Variable Gadget
        // This gadget wraps the path indices and sibling node hashes.
        // It needs the native MerkleConfig type (`GachaMerkleConfig`) as a generic parameter.
        let path_var = PathVar::<GachaMerkleConfig, FpVar<F>, Boolean<F>>::new_witness(
            cs.clone(),
            // The closure provides the native path data for witness assignment.
            // The actual values inside this native Path might not be crucial if already provided
            // via path_indices_vars and merkle_path_vars, but it's good practice to provide them.
            || {
                // Find the leaf index from path_indices - this is tricky, might need simpler approach
                // For now, just provide the path nodes. leaf_index calculation might be complex.
                // Or, PathVar might not strictly need the native path if bools/siblings are allocated.
                Ok(ark_crypto_primitives::merkle_tree::Path::<GachaMerkleConfig> {
                    auth_path: self.merkle_path_nodes, // Use the witness directly
                    // These might be derivable or not needed by PathVar if indices/siblings are provided
                    leaf_index: 0, // Placeholder - check if PathVar uses this
                    leaf_sibling_hash: Default::default(), // Placeholder
                })
            },
            &path_indices_vars, // The allocated boolean witnesses
            &merkle_path_vars   // The allocated sibling hash witnesses
        )?;

        // 2b. Perform the Merkle Path Verification using the Gadget
        // This gadget enforces that the calculated leaf hash, combined with the path variable,
        // results in the expected Merkle root.
        PathMerkleTreeVerificationGadget::<GachaMerkleConfig>::conditionally_check_membership(
            cs, // Constraint System reference
            &params_var, // Poseidon parameters needed for internal hashing
            &Boolean::TRUE, // Condition: always verify in this case
            &computed_leaf_hash_var, // The leaf hash we calculated earlier
            &path_var, // The path information gadget
            &merkle_root_var, // The public Merkle root to check against
        )?;

        Ok(())
    }
}
```

**E. WASM 인터페이스 (`src/lib.rs`):**

```rust
// packages/zk-circuits/src/lib.rs
use wasm_bindgen::prelude::*;
use ark_std::UniformRand; // Might not be needed if using getrandom::js
use ark_groth16::{Groth16, Proof, ProvingKey, VerifyingKey};
use ark_bls12_381::{Bls12_381, Fr as BlsFr};
use ark_snark::SNARK;
use std::sync::Mutex; // For thread-safe access to shared keys/params
use once_cell::sync::OnceCell; // For one-time initialization of globals
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};

// Import local modules
use crate::circuit::UserPullCircuit;
use crate::types::{WasmGachaCircuitInputs, NativeGachaCircuitInputs, prepare_groth16_public_inputs, PoseidonParam};
use crate::error::GachaCircuitError;

// --- Global Static Variables (Initialized once) ---
// Use Mutex for thread safety, although JS is single-threaded, Rust/WASM might evolve.
static GACHA_PK: OnceCell<Mutex<ProvingKey<Bls12_381>>> = OnceCell::new();
static GACHA_VK: OnceCell<Mutex<VerifyingKey<Bls12_381>>> = OnceCell::new();
// Poseidon parameters are read-only after init, so Mutex might be optional, but included for consistency.
static POSEIDON_PARAMS: OnceCell<PoseidonParam> = OnceCell::new();

/// Sets up a panic hook to forward Rust panics to the browser console.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
    Ok(())
}

/// Initializes the WASM module with necessary cryptographic keys and parameters.
/// Should be called once after fetching the key/param bytes.
///
/// # Arguments
/// * `pk_bytes`: Serialized Groth16 Proving Key bytes.
/// * `vk_bytes`: Serialized Groth16 Verifying Key bytes.
/// * `params_bytes`: Serialized Poseidon Parameter bytes.
#[wasm_bindgen]
pub fn init_gacha_keys(pk_bytes: &[u8], vk_bytes: &[u8], params_bytes: &[u8]) -> Result<(), JsValue> {
    // Prevent double initialization
    if GACHA_PK.get().is_some() || GACHA_VK.get().is_some() || POSEIDON_PARAMS.get().is_some() {
        // Use map_err to convert the specific error into JsValue
        return Err(GachaCircuitError::SetupError("Already initialized".to_string()).into());
    }

    // Deserialize parameters and keys. Use unchecked versions for faster loading.
    // Errors are converted to GachaCircuitError, then to JsValue via From impl.
    let params = PoseidonParam::deserialize_compressed_unchecked(params_bytes)
        .map_err(|e| GachaCircuitError::Deserialization(format!("Params: {}", e)))?;
    let pk = ProvingKey::<Bls12_381>::deserialize_compressed_unchecked(pk_bytes)
        .map_err(|e| GachaCircuitError::Deserialization(format!("PK: {}", e)))?;
    let vk = VerifyingKey::<Bls12_381>::deserialize_compressed_unchecked(vk_bytes)
        .map_err(|e| GachaCircuitError::Deserialization(format!("VK: {}", e)))?;

    // Atomically set the global variables. Returns error if already set.
    POSEIDON_PARAMS.set(params)
        .map_err(|_| GachaCircuitError::SetupError("Failed to set Params".to_string()))?;
    GACHA_PK.set(Mutex::new(pk))
        .map_err(|_| GachaCircuitError::SetupError("Failed to set PK".to_string()))?;
    GACHA_VK.set(Mutex::new(vk))
        .map_err(|_| GachaCircuitError::SetupError("Failed to set VK".to_string()))?;

    // Optional: Log success to console
    // web_sys::console::log_1(&"Gacha keys and params initialized successfully!".into());

    Ok(())
}

/// Generates a ZK-SNARK proof for the given gacha pull inputs.
///
/// # Arguments
/// * `inputs_bytes`: Serialized `WasmGachaCircuitInputs` (likely using `serde_wasm_bindgen`).
///
/// # Returns
/// * `Vec<u8>`: The generated proof, serialized (compressed).
#[wasm_bindgen]
pub fn generate_gacha_proof(inputs_bytes: &[u8]) -> Result<Vec<u8>, JsValue> {
    // 1. Deserialize inputs from WASM call
    let wasm_inputs: WasmGachaCircuitInputs = serde_wasm_bindgen::from_bytes(inputs_bytes)
        .map_err(|e| GachaCircuitError::Deserialization(format!("WASM inputs: {}", e)))?;
    // Convert DTO to native Rust structure
    let native_inputs = NativeGachaCircuitInputs::try_from(wasm_inputs)?;

    // 2. Retrieve Proving Key and Parameters safely
    let pk_lock = GACHA_PK.get().ok_or(GachaCircuitError::NotInitialized)?;
    let params = POSEIDON_PARAMS.get().ok_or(GachaCircuitError::NotInitialized)?.clone(); // Clone params
    // Lock the mutex to access the Proving Key
    let pk = pk_lock.lock().map_err(|_| GachaCircuitError::SetupError("Failed to lock PK mutex".to_string()))?;

    // 3. Create the circuit instance
    let circuit = UserPullCircuit::<BlsFr>::new(native_inputs, params);

    // 4. Generate the proof
    // Use a CSPRNG suitable for WASM environments
    let mut rng = rand::thread_rng(); // getrandom::js feature should make this work
    let proof = Groth16::<Bls12_381>::prove(&pk, circuit, &mut rng)
        .map_err(|e| GachaCircuitError::ProofGeneration(e.to_string()))?; // Convert Groth16 error

    // 5. Serialize the proof (compressed)
    let mut proof_bytes = Vec::new();
    // This can return ark_serialize::SerializationError, which converts to GachaCircuitError
    proof.serialize_compressed(&mut proof_bytes)?;

    Ok(proof_bytes)
}

/// Verifies a ZK-SNARK proof against the public inputs.
///
/// # Arguments
/// * `merkle_root_hex`: The public Merkle root (hex string) used as input to the circuit.
/// * `proof_bytes`: The serialized proof generated by `generate_gacha_proof`.
///
/// # Returns
/// * `bool`: `true` if the proof is valid, `false` otherwise.
#[wasm_bindgen]
pub fn verify_gacha_proof(
    merkle_root_hex: String,
    proof_bytes: &[u8]
) -> Result<bool, JsValue> {
    // 1. Retrieve Verifying Key safely
    let vk_lock = GACHA_VK.get().ok_or(GachaCircuitError::NotInitialized)?;
    let vk = vk_lock.lock().map_err(|_| GachaCircuitError::SetupError("Failed to lock VK mutex".to_string()))?;

    // 2. Prepare public inputs
    // Convert hex Merkle root to native field element
    let merkle_root = crate::types::fr_from_hex(&merkle_root_hex)?;
    // Create the public input vector in the correct order
    let public_inputs = prepare_groth16_public_inputs(merkle_root);

    // 3. Deserialize the proof
    let proof = Proof::<Bls12_381>::deserialize_compressed_unchecked(proof_bytes)
         .map_err(|e| GachaCircuitError::Deserialization(format!("Proof: {}", e)))?;

    // 4. Verify the proof
    let is_valid = Groth16::<Bls12_381>::verify(&vk, &public_inputs, &proof)
        .map_err(|e| GachaCircuitError::ProofVerification(e.to_string()))?; // Convert Groth16 error

    Ok(is_valid)
}
```

**F. 오프라인 스크립트 개발 (`scripts/`)**

이 스크립트들은 별도의 Rust 바이너리로 작성되어 로컬 환경에서 실행됩니다. `packages/zk-circuits`의 코드를 라이브러리(`rlib`)로 참조하여 타입을 공유할 수 있습니다.

*   **`scripts/src/bin/generate_crs.rs`:**
    *   `ark_crypto_primitives`, `ark_groth16`, `ark_bls12_381`, `rand` 등을 `use`.
    *   `zk_circuits::{circuit::UserPullCircuit, types::*}` 등을 `use`.
    *   난수 생성기 초기화 (`rand::thread_rng`).
    *   **Poseidon 파라미터 생성:** `PoseidonCRH::<BlsFr>::setup(&mut rng)` 호출 (파라미터 설정 확인 필요 - width 등).
    *   생성된 `PoseidonParam`을 `params.bin` 파일로 직렬화 저장 (`serialize_compressed`).
    *   **더미 회로 인스턴스 생성:** `UserPullCircuit::new(dummy_inputs, loaded_params)` 호출. `dummy_inputs`는 `NativeGachaCircuitInputs` 타입의 기본값 또는 임의 값 사용.
    *   **CRS 생성:** `Groth16::<Bls12_381>::circuit_specific_setup(dummy_circuit, &mut rng)` 호출.
    *   반환된 `ProvingKey`와 `VerifyingKey`를 각각 `gacha_pk.bin`, `gacha_vk.bin` 파일로 직렬화 저장 (`serialize_compressed`).
    *   성공/실패 메시지 출력.
*   **`scripts/src/bin/prepare_gacha_data.rs`:**
    *   `ark_crypto_primitives`, `ark_bls12_381`, `rand`, `serde_json`, `hex` 등을 `use`.
    *   `zk_circuits::types::*` 등을 `use`.
    *   아이템 목록 정의 (ID, 이름 등).
    *   **파라미터 로드:** `params.bin` 파일 역직렬화 (`deserialize_compressed`).
    *   난수 생성기 초기화.
    *   빈 `leaf_hashes: Vec<BlsFr>` 및 `item_data_to_save: Vec<serde_json::Value>` 생성.
    *   **아이템별 데이터 생성 루프:**
        *   `item_id: BlsFr` 생성.
        *   `secret_key: BlsFr = BlsFr::rand(&mut rng)`.
        *   `leaf_hash = PoseidonCRH::evaluate(&loaded_params, &[secret_key, item_id])?`.
        *   `leaf_hashes.push(leaf_hash)`.
        *   나중에 경로 생성 위해 `(leaf_index, item_id, secret_key)` 저장.
    *   **Merkle Tree 구축:**
        *   `MerkleTree::<GachaMerkleConfig>::new_with_leaf_digest(&loaded_params, &loaded_params, &leaf_hashes)` 호출 (첫번째는 LeafHash용, 두번째는 TwoToOneHash용 파라미터).
    *   **Merkle Root 저장:** `root = tree.root()`. `fr_to_hex(&root)?` 결과를 `merkle_root.hex` 파일로 저장.
    *   **경로 및 JSON 생성 루프 (저장된 `(leaf_index, item_id, secret_key)` 사용):**
        *   `path_struct: MerklePath<GachaMerkleConfig> = tree.generate_proof(leaf_index)?`.
        *   `path_indices: Vec<bool> = path_struct.position_list().collect()`.
        *   `merkle_path_hex: Vec<String> = path_struct.auth_path.iter().map(|h| fr_to_hex(h)).collect::<Result<_,_>>()?`.
        *   JSON 객체 생성:
            ```json
            {
              "itemIdHex": fr_to_hex(&item_id)?,
              "secretKeyHex": fr_to_hex(&secret_key)?,
              "merklePathHex": merkle_path_hex,
              "pathIndices": path_indices
            }
            ```
        *   JSON 객체를 `item_{leaf_index}.json` 파일로 저장.
        *   (선택) 저장된 JSON 파일 경로 목록을 `key_list.txt`에 추가.
    *   성공/실패 메시지 출력.

---

**4단계: 웹 애플리케이션 개발 (`apps/web`)**

**A. 라우팅 (`app/gacha/page.tsx` 등):**

*   메인 가챠 페이지 (`/gacha`) 생성.
*   (선택) 인벤토리 페이지 (`/gacha/inventory`) 생성.

**B. 상태 관리 (Zustand 사용 예시 - `stores/gachaStore.ts`):**

```typescript
import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware'; // For inventory persistence

// Define types for clarity
interface ItemDetails {
  id: string; // Original item ID (e.g., "sword_001")
  name: string;
  imageUrl: string;
  rarity?: string;
}

interface PullResultData {
  itemIdHex: string;
  secretKeyHex: string;
  merklePathHex: string[];
  pathIndices: boolean[];
  itemDetails: ItemDetails; // Resolved item info
}

interface GachaState {
  // Static Assets (Loaded once)
  merkleRoot: string | null;
  provingKey: Uint8Array | null;
  verifyingKey: Uint8Array | null;
  poseidonParams: Uint8Array | null;
  itemMasterData: Record<string, ItemDetails> | null; // Map from itemIdHex to details
  availableKeyUrls: string[]; // List of URLs for item_{index}.json files

  // Initialization Status
  isWasmInitialized: boolean;
  isKeysInitialized: boolean; // Includes params
  isLoadingAssets: boolean;
  initializationError: string | null;

  // Gacha Process State
  isPulling: boolean;
  pullResult: PullResultData | null; // Data for the latest pull
  pullError: string | null;

  // Proof State
  isGeneratingProof: boolean;
  proof: Uint8Array | null;
  proofGenerationError: string | null;

  // Verification State
  isVerifyingProof: boolean;
  verificationResult: boolean | null; // true = success, false = fail
  verificationError: string | null;

  // User Inventory (Persisted)
  inventory: ItemDetails[];

  // Actions
  loadInitialAssets: () => Promise<void>;
  performPull: () => Promise<void>;
  generateProofForLastPull: () => Promise<void>;
  verifyProofForLastPull: () => Promise<void>;
  resetPullState: () => void;
  // Inventory actions managed by persist middleware implicitly
}

export const useGachaStore = create<GachaState>()(
  persist(
    (set, get) => ({
      // Initial state...
      merkleRoot: null,
      provingKey: null,
      verifyingKey: null,
      poseidonParams: null,
      itemMasterData: null,
      availableKeyUrls: [],
      isWasmInitialized: false,
      isKeysInitialized: false,
      isLoadingAssets: false,
      initializationError: null,
      isPulling: false,
      pullResult: null,
      pullError: null,
      isGeneratingProof: false,
      proof: null,
      proofGenerationError: null,
      isVerifyingProof: false,
      verificationResult: null,
      verificationError: null,
      inventory: [],

      // --- Actions ---
      loadInitialAssets: async () => {
        set({ isLoadingAssets: true, initializationError: null });
        try {
          const { loadWasm, init_gacha_keys } = await import('zk-circuits'); // Dynamic import

          // Fetch assets in parallel (adjust paths as needed)
          const [
            pkRes, vkRes, paramsRes, rootRes, keyListRes, masterDataRes, _ // Wasm loaded via import
          ] = await Promise.all([
            fetch('/gacha_pk.bin'),
            fetch('/gacha_vk.bin'),
            fetch('/params.bin'),
            fetch('/merkle_root.hex'), // Fetch the hex string
            fetch('/key_list.txt'),   // Fetch the list of JSON file URLs
            fetch('/item_master.json') // Fetch item details map
            // loadWasm() is implicitly handled by dynamic import above
          ]);

          if (!pkRes.ok || !vkRes.ok || !paramsRes.ok || !rootRes.ok || !keyListRes.ok || !masterDataRes.ok) {
            throw new Error('Failed to fetch one or more initial assets');
          }

          const pkBytes = new Uint8Array(await pkRes.arrayBuffer());
          const vkBytes = new Uint8Array(await vkRes.arrayBuffer());
          const paramsBytes = new Uint8Array(await paramsRes.arrayBuffer());
          const merkleRootHex = (await rootRes.text()).trim(); // Get root as string
          const keyListText = await keyListRes.text();
          const itemMaster = await masterDataRes.json();

          const availableKeyUrls = keyListText.split('\n').filter(line => line.trim() !== '');

          // Initialize WASM keys/params AFTER fetching is complete
          await init_gacha_keys(pkBytes, vkBytes, paramsBytes);

          set({
            provingKey: pkBytes, // Store if needed later? Maybe not.
            verifyingKey: vkBytes, // Store if needed later? Maybe not.
            poseidonParams: paramsBytes, // Store if needed later? Maybe not.
            merkleRoot: merkleRootHex,
            availableKeyUrls: availableKeyUrls,
            itemMasterData: itemMaster, // Assuming JSON maps itemIdHex to ItemDetails
            isWasmInitialized: true, // Assuming dynamic import handles this
            isKeysInitialized: true,
            isLoadingAssets: false,
          });

        } catch (error) {
          console.error("Initialization failed:", error);
          set({
            initializationError: error instanceof Error ? error.message : String(error),
            isLoadingAssets: false,
          });
        }
      },

      performPull: async () => {
        const { availableKeyUrls, itemMasterData } = get();
        if (availableKeyUrls.length === 0 || !itemMasterData) {
          set({ pullError: "Assets not loaded or no keys available." });
          return;
        }
        set({ isPulling: true, pullError: null, pullResult: null, proof: null, verificationResult: null }); // Reset previous pull state

        try {
          // **DEMO ONLY**: Randomly select a key URL. In production, a backend determines this.
          const randomIndex = Math.floor(Math.random() * availableKeyUrls.length);
          const selectedUrl = availableKeyUrls[randomIndex]; // Use relative path if in /public

          const response = await fetch(selectedUrl);
          if (!response.ok) {
            throw new Error(`Failed to fetch item data from ${selectedUrl}`);
          }
          const itemData = await response.json(); // Parses the JSON file

          // Validate fetched data structure (basic check)
          if (!itemData.itemIdHex || !itemData.secretKeyHex || !itemData.merklePathHex || !itemData.pathIndices) {
              throw new Error(`Invalid item data structure received from ${selectedUrl}`);
          }

          // Find item details from master data
          const itemDetails = itemMasterData[itemData.itemIdHex];
          if (!itemDetails) {
              throw new Error(`Item details not found for ID: ${itemData.itemIdHex}`);
          }

          // Add to inventory (or do this after verification)
           set((state) => ({ inventory: [...state.inventory, itemDetails] }));

          set({
            pullResult: { ...itemData, itemDetails }, // Combine fetched data and details
            isPulling: false,
          });

        } catch (error) {
          console.error("Pull failed:", error);
          set({
            pullError: error instanceof Error ? error.message : String(error),
            isPulling: false,
          });
        }
      },

      generateProofForLastPull: async () => {
        const { pullResult, merkleRoot } = get();
        if (!pullResult || !merkleRoot) {
          set({ proofGenerationError: "No pull data or Merkle root available." });
          return;
        }
        set({ isGeneratingProof: true, proofGenerationError: null, proof: null, verificationResult: null });

        try {
          const { generate_gacha_proof } = await import('zk-circuits');
          const { serde_wasm_bindgen } = await import('./serdeHelper'); // Helper for serialization

          // Prepare inputs for WASM function
          const wasmInputs: import('zk-circuits').WasmGachaCircuitInputs = { // Use imported type
            merkle_root: merkleRoot, // Pass the public root
            item_id: pullResult.itemIdHex,
            secret_key: pullResult.secretKeyHex,
            merkle_path: pullResult.merklePathHex,
            path_indices: pullResult.pathIndices,
          };

          // Serialize inputs using serde_wasm_bindgen
          const inputsBytes = serde_wasm_bindgen.to_bytes(wasmInputs);

          // Call WASM function
          const proofBytes = await generate_gacha_proof(inputsBytes);

          set({
            proof: proofBytes, // Store the generated proof bytes
            isGeneratingProof: false,
          });

        } catch (error) {
          console.error("Proof generation failed:", error);
          set({
            proofGenerationError: error instanceof Error ? error.message : String(error),
            isGeneratingProof: false,
          });
        }
      },

      verifyProofForLastPull: async () => {
        const { proof, merkleRoot } = get();
         if (!proof || !merkleRoot) {
          set({ verificationError: "No proof or Merkle root available." });
          return;
        }
        set({ isVerifyingProof: true, verificationError: null, verificationResult: null });

        try {
           const { verify_gacha_proof } = await import('zk-circuits');

           // Call WASM function with public inputs and proof
           const isValid = await verify_gacha_proof(merkleRoot, proof);

           set({
             verificationResult: isValid,
             isVerifyingProof: false,
           });

        } catch (error) {
           console.error("Proof verification failed:", error);
           set({
             verificationError: error instanceof Error ? error.message : String(error),
             isVerifyingProof: false,
           });
        }
      },

      resetPullState: () => {
          set({
              isPulling: false,
              pullResult: null,
              pullError: null,
              isGeneratingProof: false,
              proof: null,
              proofGenerationError: null,
              isVerifyingProof: false,
              verificationResult: null,
              verificationError: null,
          });
      },

    }),
    {
      name: 'gacha-game-storage', // Unique name for local storage
      storage: createJSONStorage(() => localStorage), // Use local storage
      partialize: (state) => ({ inventory: state.inventory }), // Only persist inventory
    }
  )
);

// Helper for serde_wasm_bindgen (e.g., stores/serdeHelper.ts)
// This avoids direct dependency in the store file if preferred.
// export const serde_wasm_bindgen = {
//   to_bytes: (value: any): Uint8Array => {
//     // Dynamically import or ensure it's loaded globally
//     // return serde_wasm_bindgen_instance.to_bytes(value);
//     throw new Error("serde_wasm_bindgen not loaded"); // Implement actual loading
//   }
// }
```

**C. 초기 로딩 로직 (`app/gacha/page.tsx` 또는 layout):**

```tsx
'use client';

import { useEffect } from 'react';
import { useGachaStore } from '@/stores/gachaStore';
import GachaDisplay from '@/components/gacha/GachaDisplay'; // Your main component

export default function GachaPage() {
  const { loadInitialAssets, isLoadingAssets, initializationError } = useGachaStore((state) => ({
    loadInitialAssets: state.loadInitialAssets,
    isLoadingAssets: state.isLoadingAssets,
    initializationError: state.initializationError,
  }));

  // Load assets once on component mount
  useEffect(() => {
    loadInitialAssets();
  }, [loadInitialAssets]); // Dependency array ensures it runs once

  if (isLoadingAssets) {
    return <div>Loading Gacha Assets... Please Wait...</div>;
  }

  if (initializationError) {
    return <div style={{ color: 'red' }}>Initialization Failed: {initializationError}</div>;
  }

  // Render the main gacha component once assets are loaded
  return (
    <div>
      <h1>ZK Verifiable Gacha</h1>
      <GachaDisplay />
      {/* Add Inventory Link/Component */}
    </div>
  );
}
```

**D. 핵심 UI 컴포넌트 (`components/gacha/GachaDisplay.tsx` 예시):**

```tsx
'use client';

import { useGachaStore } from '@/stores/gachaStore';
import { Button } from '@/components/ui/button'; // Assuming Shadcn UI
// Import animation and result display components

export default function GachaDisplay() {
  const {
    isKeysInitialized,
    performPull, isPulling, pullResult, pullError,
    generateProofForLastPull, isGeneratingProof, proof, proofGenerationError,
    verifyProofForLastPull, isVerifyingProof, verificationResult, verificationError,
    resetPullState
  } = useGachaStore();

  const handlePull = () => {
    resetPullState(); // Reset previous states before new pull
    performPull();
  };

  const handleGenerateProof = () => {
      generateProofForLastPull();
  };

  const handleVerifyProof = () => {
      verifyProofForLastPull();
  };

  if (!isKeysInitialized) {
      return <p>Initializing cryptographic components...</p>; // Or handle in parent
  }

  return (
    <div>
      {/* Pull Button */}
      <Button onClick={handlePull} disabled={isPulling || isGeneratingProof || isVerifyingProof}>
        {isPulling ? 'Pulling...' : 'Pull Gacha (1x)'}
      </Button>
      {pullError && <p style={{ color: 'red' }}>Pull Error: {pullError}</p>}

      {/* Gacha Animation Trigger (conditionally render based on pullResult) */}
      {/* <GachaAnimation pullData={pullResult} /> */}

      {/* Result Display Area */}
      {pullResult && !isPulling && (
        <div style={{ border: '1px solid #ccc', padding: '10px', marginTop: '10px' }}>
          <h2>You Pulled:</h2>
          <p>
            <img src={pullResult.itemDetails.imageUrl} alt={pullResult.itemDetails.name} width={50} />
            {pullResult.itemDetails.name} ({pullResult.itemDetails.rarity})
          </p>
          <p>Item ID (Hex): {pullResult.itemIdHex}</p>

          {/* Proof Generation Button */}
          {!proof && !isGeneratingProof && (
            <Button onClick={handleGenerateProof} disabled={isGeneratingProof || isVerifyingProof}>
              Generate Verification Proof
            </Button>
          )}
          {isGeneratingProof && <p>Generating Proof...</p>}
          {proofGenerationError && <p style={{ color: 'red' }}>Proof Gen Error: {proofGenerationError}</p>}

          {/* Proof Verification Button */}
          {proof && !isVerifyingProof && verificationResult === null && (
             <Button onClick={handleVerifyProof} disabled={isVerifyingProof}>
               Verify Proof
             </Button>
          )}
          {isVerifyingProof && <p>Verifying Proof...</p>}
          {verificationError && <p style={{ color: 'red' }}>Verification Error: {verificationError}</p>}

          {/* Verification Result */}
          {verificationResult !== null && (
            <p style={{ color: verificationResult ? 'green' : 'red', fontWeight: 'bold' }}>
              Verification {verificationResult ? 'Successful!' : 'Failed!'}
            </p>
          )}
        </div>
      )}
    </div>
  );
}
```

**E. 인벤토리 (`app/gacha/inventory/page.tsx`):**

```tsx
'use client';
import { useGachaStore } from '@/stores/gachaStore';

export default function InventoryPage() {
    const inventory = useGachaStore((state) => state.inventory);

    return (
        <div>
            <h1>My Gacha Inventory</h1>
            {inventory.length === 0 ? (
                <p>Your inventory is empty. Pull some items!</p>
            ) : (
                <ul>
                    {inventory.map((item, index) => (
                        <li key={`${item.id}-${index}`}> {/* Use a more stable key if possible */}
                            <img src={item.imageUrl} alt={item.name} width={30} />
                            {item.name} ({item.rarity})
                        </li>
                    ))}
                </ul>
            )}
        </div>
    );
}
```

---

이 가이드는 프로젝트의 전체 수명 주기를 다루며, 각 단계에서 필요한 코드 구조와 로직을 상세히 설명합니다. 실제 구현 시에는 `ark-crypto-primitives` 라이브러리의 정확한 API 문서를 참조하고, 프론트엔드에서는 상태 관리와 사용자 경험을 더욱 다듬어야 합니다.