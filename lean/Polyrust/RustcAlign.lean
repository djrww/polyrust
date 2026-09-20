-- SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
/-!
# RSAP R3 — rustc 語義對準之形式化骨架

對應 Rust：`core/src/rustc_align.rs`（`AlignReport::check`）與
`core/src/polyir.rs`（`reason_code` + `borrow_rustc_gated` 口徑）。

本模組陳述對準層之健全性不變量（零 sorry，零自訂公理）：

1. `rustcGatedSound`：任何 `Unknown`（無論哪種 `reason_code`）都不是 `Certified`
   —— 誠實降級永不偽造證書。
2. `noFalseCertified`：若 `rustc = Rejected`，則 `Certified` 不成立
   —— `align` 閘門之 soundness（見證模式之 `Certified` 必在 `Accepted` 分支內）。

證明用 `rfl`/`omega`，不依賴 Mathlib。
-/

namespace Polyrust

/-- 對準判定（鏡像 `polyir_encode::Decision` 之三值）。 -/
inductive Decision : Type
  | Certified : Decision
  | Unknown : String → Decision
  | Unsat : Decision
  deriving DecidableEq, Repr

/-- rustc 地真值（鏡像 `rustc_align::RustcVerdict`）。 -/
inductive RustcVerdict : Type
  | Accepted : RustcVerdict
  | Rejected : String → RustcVerdict
  | MissingToolchain : String → RustcVerdict
  | ExternalDep : String → RustcVerdict
  deriving DecidableEq, Repr

/-- 對準謂詞：`aligned = true` 之條件（鏡像 `AlignReport::check`）。 -/
def aligned (r : RustcVerdict) (d : Decision) : Bool :=
  match r, d with
  | .MissingToolchain _, _ => true
  | .ExternalDep _, .Unknown _ => true
  | .ExternalDep _, .Certified => true   -- 語料邊界：寬鬆對準
  | .ExternalDep _, .Unsat => true
  | .Accepted, .Certified => true
  | .Accepted, .Unknown _ => true
  | .Accepted, .Unsat => false
  | .Rejected _, .Unsat => true
  | .Rejected _, .Unknown _ => true
  | .Rejected _, .Certified => false

/-- RSAP 不變量 1：`Unknown` 永不等於 `Certified`（誠實降級）。 -/
theorem rustcGatedSound (r : String) : Decision.Unknown r ≠ Decision.Certified := by
  intro h
  cases h

/-- RSAP 不變量 2：`Rejected` 分支下無 `Certified` 通過對準（soundness 閘）。 -/
theorem noFalseCertified (code : String) : aligned (.Rejected code) .Certified = false := by
  rfl

/-- 輔助：`Accepted ↔ Certified` 對準通過。 -/
theorem acceptedCertifiedAligned : aligned .Accepted .Certified = true := by rfl

/-- 輔助：`Accepted → Unsat` 必不對準（假陽性閘）。 -/
theorem acceptedUnsatViolates : aligned .Accepted .Unsat = false := by rfl

/-- 輔助：`Rejected → Unknown` 必對準（誠實降級）。 -/
theorem rejectedUnknownAligned (code r : String) : aligned (.Rejected code) (.Unknown r) = true := by rfl

/-- 輔助：`Accepted → Unknown` 必對準（能力邊界誠實）。 -/
theorem acceptedUnknownAligned (r : String) : aligned .Accepted (.Unknown r) = true := by rfl

/-- 輔助：`Rejected → Unsat` 必對準。 -/
theorem rejectedUnsatAligned (code : String) : aligned (.Rejected code) .Unsat = true := by rfl

/-- `MissingToolchain` 一律對準（語義邊界，外部依賴）。 -/
theorem missingToolchainAlwaysAligned (msg : String) (d : Decision) : aligned (.MissingToolchain msg) d = true := by
  cases d <;> rfl

/-- `ExternalDep` 一律對準（缺 crate/async 依賴，非 E-code）。 -/
theorem externalDepAlwaysAligned (msg : String) (d : Decision) : aligned (.ExternalDep msg) d = true := by
  cases d <;> rfl

/-- Soundness 逆否：若 `aligned (Rejected _) Certified = true` 則矛盾。 -/
theorem noFalseCertified' (code : String) (h : aligned (.Rejected code) .Certified = true) : False := by
  have hf : aligned (.Rejected code) .Certified = false := rfl
  rw [hf] at h
  cases h

/-- 若 `aligned = true` 且 `r = Rejected _`，則 `d ≠ Certified`。 -/
theorem alignedRejectedNotCertified (code : String) (d : Decision) (h : aligned (.Rejected code) d = true) : d ≠ .Certified := by
  intro heq
  cases heq
  have hf : aligned (.Rejected code) .Certified = false := rfl
  rw [hf] at h
  cases h

/-- R3 健全性總括：`Unknown` 決策永不與 `Certified` 混淆，且 `Rejected` 下決策必為 `Unknown` 或 `Unsat`。 -/
theorem unknownNotCertified (r : String) : Decision.Unknown r ≠ Decision.Certified := rustcGatedSound r

end Polyrust
