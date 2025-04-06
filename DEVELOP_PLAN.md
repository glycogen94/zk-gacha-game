알겠습니다. 3단계까지의 코드 구현 및 테스트를 반영하여 개발 계획을 다듬고, 향후 진행해야 할 작업 단계를 명확히 제시하겠습니다.

---

**ZK Gacha Game - 개발 가이드 (수정본)**

이 문서는 `Nextjs-Rust-WASM-monorepo` 템플릿을 기반으로 Arkworks 라이브러리를 사용하여 ZK-SNARK 기반의 검증 가능한 웹 가챠 게임 데모를 구축하는 전체 과정을 안내합니다. 현재 3단계까지의 핵심 Rust 코드 구현 및 테스트가 완료되었음을 반영합니다.

**단계 요약:**

1.  **템플릿 기반 새 저장소 생성 및 초기 설정:** 완료.
2.  **새 프로젝트 README 작성:** 완료. (중요 보안 한계점 포함)
3.  **Rust 코드 로직 구현 (`packages/zk-circuits`):** **완료.** (회로 정의, 타입, WASM 바인딩, 네이티브 테스트 포함)
4.  **오프라인 스크립트 구현 (`scripts/`):** CRS 생성 및 데이터 준비 스크립트 완료 및 실행.
5.  **웹 애플리케이션 개발 (`apps/web`):** 가챠 게임 UI, 상태 관리, WASM 연동, 데이터 처리, 초기화 로직 등 프론트엔드 구현.
6.  **테스트 및 배포:** 프론트엔드 테스트, 데이터 배포, 웹 애플리케이션 배포.

---

**1단계: 템플릿 기반 새 저장소 생성 및 초기 설정**

*   **완료됨:** `zk-gacha-game` 저장소가 `Nextjs-Rust-WASM-monorepo` 템플릿으로부터 생성되었고, `GETTING_STARTED.md` 가이드에 따라 프로젝트 이름(`zk-gacha-game`), 웹 앱 이름(`web`), WASM 라이브러리 이름(`zk-circuits`)이 성공적으로 변경 및 설정되었습니다. `pnpm install`을 통해 의존성 설치가 완료되었습니다.

---

**2단계: 새 프로젝트 README 작성 (`/README.md`)**

*   **완료됨:** 프로젝트의 목적, 주요 기능, 기술 스택, **작동 방식(암호화 흐름 상세 포함)**, 설치 및 실행 방법, 그리고 **중요한 보안상의 한계점**(클라이언트 측 키 관리, 확률 미적용 등)을 명시한 포괄적인 README 파일이 작성되었습니다.

---

**3단계: Rust 코드 로직 구현 (`packages/zk-circuits`)**

*   **완료됨:** ZK 가챠 게임의 핵심 암호화 로직 및 회로가 Rust로 구현되었습니다.
    *   **곡선 및 필드:** BLS12-381 곡선 (`ark-bls12-381`) 및 관련 스칼라 필드 (`Fr`)를 제약 조건 필드로 사용하여 회로를 정의했습니다.
    *   **암호화 프리미티브:** `ark-crypto-primitives` 라이브러리를 사용하여 Poseidon 해시 함수 (`PoseidonCRH`, `PoseidonTwoToOneCRH`) 및 Merkle Tree (`MerkleTree`, `MerklePath`)를 올바르게 활용했습니다.
    *   **타입 정의 (`src/types.rs`):** `ConstraintField`, `GachaMerkleConfig`, `NativeMerklePath`, `NativePoseidonConfig`, `NativeGachaCircuitInputs`, `WasmGachaCircuitInputs` 등 필요한 네이티브 및 WASM DTO 타입을 명확하게 정의하고, `fr_from_hex`/`fr_to_hex` 변환 유틸리티를 구현했습니다.
    *   **회로 설계 (`src/circuit.rs`):** `UserPullCircuit`는 `ConstraintSynthesizer<ConstraintField>`를 구현합니다. Poseidon을 사용하여 `leaf_hash = H(secret_key, item_id)`를 계산하고, `ark-crypto-primitives`의 `MerklePathVar` 가젯과 관련 해시 가젯 (`CRHGadget`, `TwoToOneCRHGadget`)을 사용하여 Merkle 경로 멤버십 증명을 제약 조건으로 올바르게 표현했습니다.
    *   **에러 처리 (`src/error.rs`):** `GachaCircuitError` 열거형을 정의하여 다양한 오류 상황(직렬화, 증명 생성/검증 실패, 초기화 오류 등)을 처리하고, `JsValue`로 변환 가능하도록 구현했습니다.
    *   **WASM 바인딩 (`src/lib.rs`):**
        *   `init_gacha_keys`: PK, VK, Poseidon 파라미터 바이트를 받아 전역 정적 변수 (`OnceCell<Mutex<T>>`)에 안전하게 초기화합니다.
        *   `generate_gacha_proof`: `WasmGachaCircuitInputs`를 받아 네이티브 타입으로 변환 후, `UserPullCircuit` 인스턴스를 생성하고 `Groth16::prove`를 호출하여 증명 바이트를 반환합니다.
        *   `verify_gacha_proof`: 공개 입력(Merkle Root)과 증명 바이트를 받아 `Groth16::verify`를 호출하고 검증 결과(`bool`)를 반환합니다.
    *   **네이티브 테스트 (`tests/integration_tests.rs`):**
        *   회로 제약 조건 만족/불만족 테스트 구현 완료.
        *   Groth16 증명 생성 및 (올바른/잘못된 공개 입력에 대한) 검증 테스트 구현 완료.
        *   테스트용 Poseidon 파라미터 및 Merkle Tree 데이터 생성 헬퍼 함수 구현 완료.

---

**4단계: 오프라인 스크립트 구현 (`scripts/`)**

*   **완료됨:** CRS 생성 및 데이터 준비를 위한 오프라인 Rust 스크립트가 구현되었습니다.
    *   **`scripts/src/bin/generate_crs.rs`:**
        *   `ark-bls12-381` 곡선 사용.
        *   Poseidon 파라미터 (`NativePoseidonConfig`) 생성 로직 구현.
        *   더미 `UserPullCircuit` 인스턴스 생성 로직 구현.
        *   `Groth16::<Bls12_381>::circuit_specific_setup`을 사용하여 Groth16 PK, VK 생성.
        *   생성된 파라미터(`params.bin`), PK(`gacha_pk.bin`), VK(`gacha_vk.bin`)를 파일로 직렬화하여 저장.
    *   **`scripts/src/bin/prepare_gacha_data.rs`:**
        *   아이템 마스터 데이터 정의.
        *   `params.bin` 로드.
        *   각 아이템(및 패딩 아이템)에 대해 `item_id` (`ConstraintField`) 생성 및 `secret_key` (`ConstraintField`) 랜덤 생성.
        *   `leaf_hash = PoseidonCRH::<ConstraintField>::evaluate(&params, &[secret_key, item_id])` 계산.
        *   `MerkleTree::<GachaMerkleConfig>::new_with_leaf_digest`를 사용하여 Merkle Tree 구축.
        *   Merkle Root 계산 및 `merkle_root.hex` 파일로 저장.
        *   각 리프 인덱스 `i`에 대해 `tree.generate_proof(i)`를 호출하여 `NativeMerklePath` 획득.
        *   `path_indices`, `merkle_path_nodes_hex`, `leaf_sibling_hash_hex`, `item_id_hex`, `secret_key_hex`를 포함하는 `ItemProofData` 구조체를 생성하여 `items/item_{i}.json` 파일로 저장.
        *   생성된 JSON 파일 목록을 `key_list.txt`로 저장.
        *   아이템 마스터 데이터를 `item_master.json`으로 저장.

---

**5단계: 웹 애플리케이션 개발 (`apps/web`) - 다음 단계**

이제 Rust 백엔드 로직과 데이터 준비가 완료되었으므로, 프론트엔드 웹 애플리케이션 개발에 집중해야 합니다.

*   **5.1. WASM 모듈 로딩 및 초기화:**
    *   Next.js 애플리케이션 (`apps/web`)에서 `zk-circuits` WASM 모듈을 안정적으로 로드하는 메커니즘 구현 (예: `useEffect` 내 동적 `import()`).
    *   애플리케이션 시작 시 `/public` 또는 S3에서 `gacha_pk.bin`, `gacha_vk.bin`, `params.bin` 파일을 비동기적으로 가져옵니다.
    *   가져온 바이트 데이터를 사용하여 WASM `init_gacha_keys` 함수를 **한 번만** 호출하여 초기화합니다. 로딩 및 초기화 상태를 UI에 표시합니다.
*   **5.2. 상태 관리 설정 (Zustand 등):**
    *   `useGachaStore`와 같은 상태 저장소를 설정하여 다음 상태를 관리합니다:
        *   WASM 및 키 초기화 상태 (`isLoadingAssets`, `isKeysInitialized`, `initializationError`).
        *   로드된 공개 데이터 (`merkleRoot`, `availableKeyUrls`, `itemMasterData`).
        *   가챠 진행 상태 (`isPulling`, `pullResult`, `pullError`).
        *   증명 생성 상태 (`isGeneratingProof`, `proof`, `proofGenerationError`).
        *   증명 검증 상태 (`isVerifyingProof`, `verificationResult`, `verificationError`).
        *   사용자 인벤토리 (`inventory`).
    *   `loadInitialAssets`, `performPull`, `generateProofForLastPull`, `verifyProofForLastPull` 등의 액션 함수를 정의합니다.
*   **5.3. UI 컴포넌트 구현 (Shadcn UI 활용):**
    *   **GachaDisplay:** 메인 가챠 인터페이스. 'Pull Gacha' 버튼, 아이템 결과 표시 영역, 'Generate Proof' 버튼, 'Verify Proof' 버튼, 로딩/에러/성공/실패 메시지 표시 로직 포함.
    *   **GachaAnimation:** (선택 사항) 가챠 뽑기 시 시각적 효과를 위한 애니메이션 컴포넌트.
    *   **InventoryDisplay:** 사용자 인벤토리를 표시하는 컴포넌트 (별도 페이지 또는 모달).
    *   **LoadingSpinner / ErrorMessage:** 재사용 가능한 로딩 및 에러 표시 컴포넌트.
*   **5.4. 핵심 로직 연결:**
    *   `loadInitialAssets` 액션: `/public` 또는 S3에서 PK, VK, Params, Merkle Root, `key_list.txt`, `item_master.json`을 fetch하고 `init_gacha_keys` 호출.
    *   `performPull` 액션: (데모) `availableKeyUrls`에서 무작위 URL 선택 -> 해당 `item_{i}.json` fetch -> 결과 아이템 및 증명 데이터(`pullResult`)를 상태에 저장. (실제 시스템에서는 백엔드 호출).
    *   `generateProofForLastPull` 액션: `pullResult`와 `merkleRoot`를 사용하여 `WasmGachaCircuitInputs` 구성 -> `serde_wasm_bindgen` 등으로 직렬화 -> WASM `generate_gacha_proof` 호출 -> 반환된 `proof` 바이트를 상태에 저장.
    *   `verifyProofForLastPull` 액션: 상태에 저장된 `proof`와 `merkleRoot`를 WASM `verify_gacha_proof`에 전달 -> `verificationResult` 상태 업데이트.
*   **5.5. 인벤토리 관리:**
    *   `performPull` 또는 검증 성공 시 `inventory` 상태 업데이트.
    *   Zustand의 `persist` 미들웨어를 사용하여 인벤토리를 `localStorage`에 저장/로드.

---

**6단계: 테스트 및 배포**

*   **6.1. 프론트엔드 테스트:**
    *   Vitest/RTL을 사용하여 컴포넌트 및 상태 관리 로직 단위/통합 테스트 작성.
    *   Playwright를 사용하여 가챠 뽑기, 증명 생성, 검증, 인벤토리 확인 등 E2E 시나리오 테스트 작성.
*   **6.2. 데이터 배포:**
    *   `scripts/` 실행 결과물 (`params.bin`, `gacha_pk.bin`, `gacha_vk.bin`, `merkle_root.hex`, `item_master.json`, `key_list.txt`, `items/*.json`)을 정적 파일 호스팅(Vercel의 `/public` 또는 S3 등)에 배포합니다.
*   **6.3. 웹 애플리케이션 배포:**
    *   `apps/web` 애플리케이션을 Vercel, Netlify 등 정적 호스팅 서비스에 배포합니다.

---

**요약: 다음 단계**

1.  **스크립트 실행 및 데이터 생성:** `generate_crs.rs` 및 `prepare_gacha_data.rs`를 실행하여 실제 `.bin`, `.hex`, `.json` 파일들을 생성하고 `apps/web/public/` 디렉토리에 배치합니다.
2.  **웹 앱 기본 설정:** `apps/web`에서 WASM 로더, 상태 관리 (Zustand), 기본 UI 레이아웃을 설정합니다.
3.  **초기 데이터 로딩 구현:** 웹 앱이 시작될 때 `/public`에서 CRS, Params, Root, Key List, Item Master를 로드하고 WASM `init_gacha_keys`를 호출하는 로직을 구현합니다.
4.  **가챠 및 증명/검증 플로우 구현:** UI 버튼과 WASM 함수 호출을 연결하고, 상태 관리 로직을 통해 전체 사용자 경험을 구현합니다.
5.  **UI/UX 개선:** 스타일링, 애니메이션, 오류 처리 등을 통해 사용자 경험을 향상시킵니다.
6.  **프론트엔드 테스트 작성.**
7.  **(선택) 배포 준비:** 데이터 파일을 S3 등으로 옮기고, 웹 앱 배포 설정을 구성합니다.